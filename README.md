# Ethereum Canister

Ethereum canister provides a trustless way to access Ethereum blockchain data in the ICP ecosystem.
Under the hood, it utilizes the [`helios`](https://github.com/a16z/helios) light Ethereum client which
is capable of verifying the authenticity of the data it fetches.

## Run and query the Ethereum canister

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
    account = "0xF977814e90dA44bFA03b6295A0616a897441aceC";
}'
(2_100_000_000_000_000 : nat) # canister's output
```

## Run end-to-end canister tests

```bash
dfx start --clean --background --artificial-delay 100

cargo test --target x86_64-unknown-linux-gnu
# or using nextest
cargo nextest run --target x86_64-unknown-linux-gnu
```

## Design

Ethereum canister is a thin wrapper around the `helios` light client.
The client is kept as a global state. After it is initialized, it runs the periodic synchronization
with the Ethereum consensus node in the background, while providing the trustless RPC API of the
execution node for other canisters.

### Canister API

The API definition for the Ethereum canister can be seen in the [`candid.did` file](src/ethereum_canister/candid.did).
In addition to that, for Rust canisters, all the input types are specified in the separate crate
[`interface`](src/interface/src/lib.rs) (will probably use a rename in the future)
that doesn't depend on `helios` directly and uses just a much lighter `ethers-core` for that matter.

The canister mostly just exposes the functions from the underlying helios client. Most of the users familiar with
Ethereum will likely be familiar with them too so there is no point in trying to be innovative there. Those usually just
have the same name, take the same arguments, and return the same types (or their `Candid` counterparts).

Depending on whether the function requires making an RPC call to the execution node or not, it's either marked as an
`update` or `query` function. Examples of queries can be `get_block_number` or `get_gas_price`, which just take information
from already synchronized blocks. Examples of updates are calling the function of a contract or estimating the gas for
a transaction as it requires fetching addresses and other data.

For the smart contract standards like the erc20 or erc721 the exposed API's are in form `${standard_name}_${function_name}`
eg. `erc20_balance_of`. The parameters to those functions are Candid's equivalents for the parameters from the contract's standard ABI.
It is on the ethereum canister to properly encode them before making a `call`.

As for now, the only exposed function that doesn't conform to the above is the `setup` function, which configures and starts
the helios client. It is required to be called before any other function, otherwise, the called function will return an error.
It takes urls to the consensus node and execution node the client will connect to, as well as the type of 
network it should operate on and an optional weak subjectivity checkpoint, a trusted hash of a block that nodes agree on. If not
provided, the last checkpoint from provided consensus node will be taken. Please note that providing a checkpoint that is too old
can result in much more https outcalls and computations needed to reach the synchronization,
in some cases exceeding the limits of an update call.

### Synchronization

The background loop is using `ic_cdk_timers::set_timer_interval` and runs in 12s intervals which is
equal to the Ethereum slot time. This is the only place where the running helios client is ever mutated
and the time of locking for updating was reduced to the required minimum which should be unnoticeable.

### Upgrades

The canister stores its configuration and the last checkpoint it has reached in stable memory. When upgrading the canister
it should restart the helios client itself with the previous configuration and the checkpoint that was already trusted. The last
64 blocks will be re-fetched.

### Error handling

Researching the approaches to error handling in inter-canister calls yields two results: returning the `Result`s
or just panicking when something goes wrong. The panicking way was chosen, as the `Result`s seemed less ergonomic for
potential developers and they feel a bit inconsistent as `CallResult` already has `CanisterError` and `CanisterReject`
variants. Also going with panics and having the state reverted felt more familiar with the smart contracts on other blockchains.

## Implementation notes

### Helios

The foundation of this canister. The ethereum canister depends on the `client`, `consensus`, `execution`, and `common` helios crates.
To be able to use a helios on the ICP it had [to be forked](https://github.com/eigerco/helios). The fork introduced many
changes to the internals of helios thus making itself not-upstreamable in its current form. Probably it is possible to upstream
some of those changes and maybe even make it compatible with the ICP ecosystem but this is considered to be not a trivial task
with many possible ways of achieving the goals that should be considered and brainstormed.
The changes were made in a manner that helios still works perfectly fine for native, but when targeting wasm it only supports
the ICP and no longer the browsers.

The changes include:
- updating the helios and making it compatible with the rust `stable` toolchain
- getting rid of any occurrences of wasm-bindgen, glue-timers, and other browser-related wasm dependencies
- getting rid of tokio-related stuff that is incompatible with wasm
- using ethers-core where possible and removing ethers-providers entirely
- replace `reqwest` with https outcalls when targeting wasm
- update the `revm` to v3 and introduce a way to fetch missing slots outside in an async manner
- change the updating logic to only lock for a short period and add proper shutdown and cleanup

For the complete list of changes see this [comparison](https://github.com/a16z/helios/compare/master...eigerco:helios:master).

### Ethers

The `ethers` crate is the building block of helios and the home for most of the types from the Ethereum ecosystem. It also
provides the utilities to generate types and encoding logic from the smart contract's ABI. It would be really useful to have this
crate working for the ICP ecosystem completely nonetheless this is considered a nontrivial task. The main problem is that it
has already good support for browser-side wasm and some of its subcrates utilize not supported crates heavily. The main pain
point we've encountered was the `ethers-providers` crate. The best way to approach this would be to support ICP in crates
like [`instant`](https://github.com/sebcrozet/instant) and [`futures-timer`](https://github.com/async-rs/futures-timer) or even
[`gloo-timers`](https://github.com/rustwasm/gloo), guard the http implementation behind the feature flag and add a way for a user
to add a custom provider.

There was one change made to the ethers, allowing the use of `ethers-contract` without the need to depend on `ethers-providers
and it was [already upstreamed](https://github.com/gakonst/ethers-rs/pull/2536).

## Cost analysis

The measurements below were taken using a local `dfx` network setup. They included additional logs and logic in the canister's code and
its dependencies. Those measurements are not precise and shouldn't be taken as truth, rather just to shed some
light on the potential costs.

|                            | call cost [cycles] | payments [cycles] |  instructions | https outcalls |
|:---------------------------|-------------------:|------------------:|--------------:|---------------:|
| setup                      |    122_579_275_431 |   122_553_254_989 | 1_149_129_243 |             74 |
| advance                    |     14_126_362_419 |    12_794_806_467 |   839_856_523 |              8 |
| get_gas_price              |            105_594 |                 0 |         5_605 |              0 |
| get_block_number           |            101_230 |                 0 |         2_336 |              0 |
| erc20_balance_of shiba-inu |     14_438_775_194 |    14_438_169_162 |    10_996_872 |              9 |
| erc20_balance_of bnb       |     14_439_107_417 |    14_438_470_723 |    12_031_945 |              9 |
| erc20_balance_of usdt      |     17_655_037_455 |    17_654_021_703 |    24_259_653 |             11 |
| erc721_owner_of  ens       |     17_654_474_853 |    17_653_498_044 |    23_011_079 |             11 |
| erc721_owner_of  dreadfulz |     17_656_442_523 |    17_655_209_030 |    31_469_631 |             11 |
| erc721_owner_of  milady    |     20_866_407_305 |    20_865_374_497 |    25_107_423 |             13 |

All the functions were measured with the advancing (sync loop) disabled except the advance itself.
All the functions were measured with 5 repetitions and the maximum acquired value was presented in the table.

The `setup` function was measured by fetching the latest checkpoint from the consensus node.

The 'call cost' was measured using the `dfx canister status` command before and after executing the case.

The 'payments' was measured using the difference between `ic_cdk::api::canister_balance` invoked at the beginning
and the end of the function. It should reflect the amount spent for https outcalls.

The 'instructions' were measured using the difference between `ic_cdk::api::performance_counter` invoked at the
beginning and the end of the function. It should reflect the number of wasm instructions executed during the call.

The 'https outcalls' was measured by counting the calls to the `http::get` and `http::post` functions.

## Next steps

### Optimization ideas

Https outcalls are the biggest factor of the costs and it should be the primary focus when optimizing.

The most pricey method is the `setup` call, as it has to initialize everything for the first time and fetch the last 64 blocks.
Unfortunately, it uses mostly the beacon API which is a restful http api, not a json-rpc like the execution API, so requests cannot
be batched easily. And even then, a single block can be bigger than 1MB so fetching two at the time could exceed the limit
[of 2MB](https://internetcomputer.org/docs/current/developer-docs/integrations/https-outcalls/https-outcalls-how-it-works) for
the https outcall.

However, it seems to be possible to heavily optimize all the execution RPC calls. Here is a sample log from the smart contract call.

<details>

<summary>erc721_owner_of milady log</summary>

```
common::http::icp] POST https://ethereum.publicnode.com {"id":0,"jsonrpc":"2.0","method":"eth_createAccessList","params":[{"type":"0x02","to":"0x5af0d9827e0c53e4799bb226655a1de152a425a5","gas":"0x5f5e100","data":"0x6352211e0000000000000000000000000000000000000000000000000000000000001e5d","accessList":[],"maxPriorityFeePerGas":"0x0","maxFeePerGas":"0x0"},"0x111d0e5"]}
common::http::icp] POST https://ethereum.publicnode.com {"id":1,"jsonrpc":"2.0","method":"eth_getProof","params":["0x5af0d9827e0c53e4799bb226655a1de152a425a5",["0x54a8fc6a2a9e7e91706be865e93ec51a73727c47639b740f76ecd87ef67aaffa","0x0000000000000000000000000000000000000000000000000000000000000002","0x405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb9789"],"0x111d0e5"]}
common::http::icp] POST https://ethereum.publicnode.com {"id":2,"jsonrpc":"2.0","method":"eth_getProof","params":["0x0000000000000000000000000000000000000000",[],"0x111d0e5"]}
common::http::icp] POST https://ethereum.publicnode.com {"id":3,"jsonrpc":"2.0","method":"eth_getProof","params":["0x5af0d9827e0c53e4799bb226655a1de152a425a5",[],"0x111d0e5"]}
common::http::icp] POST https://ethereum.publicnode.com {"id":4,"jsonrpc":"2.0","method":"eth_getProof","params":["0x388c818ca8b9251b393131c08a736a67ccb19297",[],"0x111d0e5"]}
common::http::icp] POST https://ethereum.publicnode.com {"id":5,"jsonrpc":"2.0","method":"eth_getCode","params":["0x388c818ca8b9251b393131c08a736a67ccb19297","0x111d0e5"]}
common::http::icp] POST https://ethereum.publicnode.com {"id":6,"jsonrpc":"2.0","method":"eth_getCode","params":["0x5af0d9827e0c53e4799bb226655a1de152a425a5","0x111d0e5"]}
common::http::icp] POST https://ethereum.publicnode.com {"id":7,"jsonrpc":"2.0","method":"eth_getCode","params":["0x5af0d9827e0c53e4799bb226655a1de152a425a5","0x111d0e5"]}

execution::evm] fetch basic evm state for address=0x0000000000000000000000000000000000000000
execution::evm] fetch basic evm state for address=0x388c818ca8b9251b393131c08a736a67ccb19297
execution::evm] fetch basic evm state for address=0x5af0d9827e0c53e4799bb226655a1de152a425a5
execution::evm] fetch evm state for address=0x5af0d9827e0c53e4799bb226655a1de152a425a5, slot=38292851690199200197994960496712248457597506228771887340217700269531888005114

execution::evm] Missing slots: MissingSlots { address: 0x5af0d9827e0c53e4799bb226655a1de152a425a5, slots: [0x54a8fc6a2a9e7e91706be865e93ec51a73727c47639b740f76ecd87ef67aaffa] }
common::http::icp] POST https://ethereum.publicnode.com {"id":8,"jsonrpc":"2.0","method":"eth_getProof","params":["0x5af0d9827e0c53e4799bb226655a1de152a425a5",["0x54a8fc6a2a9e7e91706be865e93ec51a73727c47639b740f76ecd87ef67aaffa"],"0x111d0e5"]}
common::http::icp] POST https://ethereum.publicnode.com {"id":9,"jsonrpc":"2.0","method":"eth_getCode","params":["0x5af0d9827e0c53e4799bb226655a1de152a425a5","0x111d0e5"]}

execution::evm] fetch basic evm state for address=0x0000000000000000000000000000000000000000
execution::evm] fetch basic evm state for address=0x388c818ca8b9251b393131c08a736a67ccb19297
execution::evm] fetch basic evm state for address=0x5af0d9827e0c53e4799bb226655a1de152a425a5
execution::evm] fetch evm state for address=0x5af0d9827e0c53e4799bb226655a1de152a425a5, slot=38292851690199200197994960496712248457597506228771887340217700269531888005114
execution::evm] fetch evm state for address=0x5af0d9827e0c53e4799bb226655a1de152a425a5, slot=2

execution::evm] Missing slots: MissingSlots { address: 0x5af0d9827e0c53e4799bb226655a1de152a425a5, slots: [0x0000000000000000000000000000000000000000000000000000000000000002] }
common::http::icp] POST https://ethereum.publicnode.com {"id":10,"jsonrpc":"2.0","method":"eth_getProof","params":["0x5af0d9827e0c53e4799bb226655a1de152a425a5",["0x0000000000000000000000000000000000000000000000000000000000000002","0x54a8fc6a2a9e7e91706be865e93ec51a73727c47639b740f76ecd87ef67aaffa"],"0x111d0e5"]}
common::http::icp] POST https://ethereum.publicnode.com {"id":11,"jsonrpc":"2.0","method":"eth_getCode","params":["0x5af0d9827e0c53e4799bb226655a1de152a425a5","0x111d0e5"]}

execution::evm] fetch basic evm state for address=0x0000000000000000000000000000000000000000
execution::evm] fetch basic evm state for address=0x388c818ca8b9251b393131c08a736a67ccb19297
execution::evm] fetch basic evm state for address=0x5af0d9827e0c53e4799bb226655a1de152a425a5
execution::evm] fetch evm state for address=0x5af0d9827e0c53e4799bb226655a1de152a425a5, slot=38292851690199200197994960496712248457597506228771887340217700269531888005114
execution::evm] fetch evm state for address=0x5af0d9827e0c53e4799bb226655a1de152a425a5, slot=2
execution::evm] fetch evm state for address=0x5af0d9827e0c53e4799bb226655a1de152a425a5, slot=29102676481673041902632991033461445430619272659676223336789171408008386418569

execution::evm] Missing slots: MissingSlots { address: 0x5af0d9827e0c53e4799bb226655a1de152a425a5, slots: [0x405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb9789] }
common::http::icp] POST https://ethereum.publicnode.com {"id":12,"jsonrpc":"2.0","method":"eth_getProof","params":["0x5af0d9827e0c53e4799bb226655a1de152a425a5",["0x405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb9789","0x54a8fc6a2a9e7e91706be865e93ec51a73727c47639b740f76ecd87ef67aaffa","0x0000000000000000000000000000000000000000000000000000000000000002"],"0x111d0e5"]}
common::http::icp] POST https://ethereum.publicnode.com {"id":13,"jsonrpc":"2.0","method":"eth_getCode","params":["0x5af0d9827e0c53e4799bb226655a1de152a425a5","0x111d0e5"]}

execution::evm] fetch basic evm state for address=0x0000000000000000000000000000000000000000
execution::evm] fetch basic evm state for address=0x388c818ca8b9251b393131c08a736a67ccb19297
execution::evm] fetch basic evm state for address=0x5af0d9827e0c53e4799bb226655a1de152a425a5
execution::evm] fetch evm state for address=0x5af0d9827e0c53e4799bb226655a1de152a425a5, slot=38292851690199200197994960496712248457597506228771887340217700269531888005114
execution::evm] fetch evm state for address=0x5af0d9827e0c53e4799bb226655a1de152a425a5, slot=2
execution::evm] fetch evm state for address=0x5af0d9827e0c53e4799bb226655a1de152a425a5, slot=29102676481673041902632991033461445430619272659676223336789171408008386418569
```

</details>

The first thing is `Evm::batch_fetch_accounts` could be optimized to only make a single request for fetching codes and proofs after creating the access list.
Also `ExecutionClient::fetch_account` could fetch the code and proof in one go. That would reduce the amount of https outcalls used in this specific case
from 13 to 5.
Another thing worth trying is finding what additional slots will be needed for the evm execution to prefetch them too, or find a way to get the list of
all missing addresses from `revm` at once, without resimulating (but this may require changes in `revm` itself). Depending on the results this could further reduce
the amounts of calls from 5 to 3 or even 2.

### Canister 

The initial development was mainly focused on getting the `helios` and `ethers` to the point where they operate correctly in the ICP environment rather than on
aligning to the best practices and state of the art canisters implementation. A few other Rust canisters (eg. `bitcoin-canister`) and also examples from `ic_cdk`
were researched to see what is the common way of doing things, but there is much room for improvement. Things like controllers, metrics collection,
API design, handling payments, and guarding some methods from public usage should be discussed and researched further.

Also, it would be good to expose the whole `helios` API and start using this canister or showing it to the community for further input.
