# Ethereum Canister

Ethereum canister provides a trustless way to access Ethereum blockchain data in the ICP ecosystem.
Under the hood, it utilises the [`helios`](https://github.com/a16z/helios) light ethereum client which
is capable of verifying authenticity of the data it fetches.

## Run and query ethereum canister

```bash
dfx start --clean --background --artificial-delay 100
# deploy
dfx deploy
# start it
dfx canister call ethereum_canister setup 'record {
    network = variant { Mainnet };
    execution_rpc_url = "https://ethereum.publicnode.com";
    consensus_rpc_url = "https://www.lightclientdata.org";
}'
# use it
dfx canister call ethereum_canister erc20_balance_of 'record {
    contract = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
    account = "0xF977814e90dA44bFA03b6295A0616a897441aceC"
}'
(2_100_000_000_000_000 : nat) # canister's output
```

## Run end to end canister tests

```bash
dfx start --clean --background --artificial-delay 100

cargo test --target x86_64-unknown-linux-gnu
# or using nextest
cargo nextest run --target x86_64-unknown-linux-gnu
```

## Design

Ethereum canister is in fact a thin wrapper around the `helios` light client.
The client is kept as a global state. After it is initialized, it runs the periodic synchronization
with the ethereum consensus node in a background, while providing the trustless RPC access to the
execution ethereum node using inter-canister calls API.

### Canister API

The api definition for the ethereum canister can be seen in the [`candid.did` file](src/ethereum_canister/candid.did).
In addition to that, for Rust canisters, all the input types are specified in the separate crate
[`interface`](src/interface/src/lib.rs) (will probably use a rename in a future)
that doesn't depend on `helios` directly and uses just a much lighter `ethers-core` for that matter.

The canister mostly just exposes the functions from the underlying helios client. Most of the users familiar with
ethereum will likely be familiar with them too so there is no point of trying to be innovative there. Those usually just
have the same name, take the same arguments and returns the same types (or their `Candid` counterparts).

Depending on whether the function requires making an RPC call to the execution node or not, it's either marked as an
`update` or `query` function. The example of queries can be `get_block_number` or `get_gas_price`, which just take information
from already synchronized blocks. The examples of updates are calling a contracts function or estimating the gas for
a transactions as it requires fetching addresses and other data.

For the smart contract standards like the erc20 or erc721 the exposed API's are in form `${standard_name}_${function_name}`
eg. `erc20_balance_of`. The parameters to those functions are Candid's equivalents for the parameters from contract's standard ABI.
It is on the ethereum canister to properly encode them before making a `call`.

As for now, the only exposed function that doesn't conform to the above is the `setup` function, which configures and starts
the helios client. It is required to be called before any other function, otherwise the called function will return an error.
It takes urls to the consensus node and execution node the client will connect to, as well as the type of the
network it should operate on and optional weak subjectivity checkpoint, a trusted hash of a block that nodes argee on. If not
provided, the last checkpoint from provided consensus node will be taken. Please note that providing a checkpoint that is too old
can result in much more https outcalls and computations needed to reach the synchronization,
in some cases exceeding limits of an update call.

### Synchronization

The background loop is using `ic_cdk_timers::set_timer_interval` and runs in 12s interval which is
equal to the ethereum slot time. This is the only place where the running helios client is ever mutated
and the time of locking for updating was reduced to the required minimum which should be unnoticeable.

### Upgrades

The canister stores it's configuration and the last checkpoint it has reached in stable memory. When upgrading the canister
it should restart helios client itself with the previous configuration and the checkpoint that was already trusted. The last
64 blocks will be re-fetched.

### Error handling

Researching the approaches to the error handling in inter-canister calls yields to results: returning the `Result`s
or just panicking when something goes wrong. The panicking way was chosen, as the `Result`s seemed less ergonomic for
potential developers and they feel a bit inconsistent as `CallResult` already has `CanisterError` and `CanisterReject`
variants. Also going with panics and having the state reverted felt more familiar with the smart contracts on other blockchains.

## Implementation notes

### Helios

The foundation of this canister. The ethereum canister depends on the `client`, `consensus`, `execution` and `common` helios crates.
In order to be able to use a helios on the ICP it had [to be forked](https://github.com/eigerco/helios). The fork introduced many
changes to the internals of helios thus making itself not-upstreamable in a current form. Probably it is possible to upstream
some of those changes and maybe even making it compatible with the icp ecosystem but this is considered to be not a trivial task
with many possible ways of achieving the goals that should be considered and brainstormed.
The changes were made in a manner that helios still works perfectly fine for native, but when targeting wasm it only supports
the ICP and no longer the browsers.

The changes include:
- updating the helios and making it compatible with the rust `stable` toolchain
- getting rid of any occurences of wasm-bindgen, glue-timers and other browser related wasm dependencies
- getting rid of tokio related stuff that is incompatible with wasm
- using ethers-core where possible and removing ethers-providers entirely
- replace `reqwest` with https outcalls when targeting wasm
- update the `revm` to v3 and introduce a way to fetch missing slots outside in async manner
- change the updating logic to only lock for the short time period and add proper shutdown and cleanup

For the complete list of changes see this [comparison](https://github.com/a16z/helios/compare/master...eigerco:helios:master).

### Ethers

The `ethers` crate is the building block of helios and the home for most of the types from the ethereum ecosystem. It also
provides the utilities to generate types and encoding logic from the smart contract's ABI. It would be really useful to have this
crate working for the ICP ecosystem completely nonetheless this is considered a non trivial task. The main problems is that it
has already a good support for browser-side wasm and some of it's subcrates utilizes not supported crates heavily. The main pain
point we've encountered was the `ethers-providers` crate. The best way to approach this would be to support ICP in crates
like [`instant`](https://github.com/sebcrozet/instant) and [`futures-timer`](https://github.com/async-rs/futures-timer) or even
[`gloo-timers`](https://github.com/rustwasm/gloo), guard the http implementation behind feature flag and add a way for a user
to add a custom provider.

There was one change made to the ethers, allowing the use of `ethers-contract` without the need to depend on `ethers-providers
and it was [already upstreamed](https://github.com/gakonst/ethers-rs/pull/2536).

## Cost analysis

The measurements below were taken using a local `dfx` network setup. They included additional logs and logic in canister's code and
it's dependencies. As main cost calculation tool was a `dfx canister status` command before and after the action. Those measurements
are definitely not precise and shouldn't be taken as a truth, rather just to shed some light on the potential costs.


