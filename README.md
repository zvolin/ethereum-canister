# Start a testnet

```bash
dfx start --clean
```

# Deploy

```bash
dfx deploy
```

# Useful links

- [Quick Start](https://internetcomputer.org/docs/quickstart/quickstart-intro)
- [SDK Developer Tools](https://internetcomputer.org/docs/developers-guide/sdk-guide)
- [Rust Canister Devlopment Guide](https://internetcomputer.org/docs/rust-guide/rust-intro)
- [ic-cdk](https://docs.rs/ic-cdk)
- [ic-cdk-macros](https://docs.rs/ic-cdk-macros)
- [Candid Introduction](https://internetcomputer.org/docs/candid-guide/candid-intro)


# Setup

❯ dfx canister status ethereum_canister
Memory Size: Nat(3118888)
Balance: 3_091_874_479_009 Cycles

❯ dfx canister call ethereum_canister setup 'record { network = variant { Mainnet }; execution_rpc_url = "https://ethereum.publicnode.com"; consensus_rpc_url = "https://www.lightclientdata.or
g" }'
()

❯ dfx canister status ethereum_canister
Memory Size: Nat(49518376)
Balance: 2_852_737_179_637 Cycles

[WARN  ethereum_canister] Setup instructions:     521_999_761
[WARN  ethereum_canister] Setup balance diff: 239_120_968_259
diff from status:                             239_137_299_372

diff of diffs:                                     16_331_113

# Setup 2

Memory Size: Nat(3118895)
Balance: 3_091_874_505_174 Cycles

❯ dfx canister call ethereum_canister setup 'record { network = variant { Mainnet }; execution_rpc_url = "https://ethereum.publicnode.com"; consensus_rpc_url = "https://www.lightclientdata.org" }' && dfx canister status ethereum_canister

Memory Size: Nat(57775919)
Balance: 2_851_961_568_938 Cycles

common::http::icp] GET https://www.lightclientdata.org/eth/v1/beacon/headers/finalized
common::http::icp] GET https://beacon-nd-995-871-887.p2pify.com:443/c9dce41bab3e120f541e4ffb748efa60/eth/v1/beacon/headers/finalized
common::http::icp] resp size: 653b
common::http::icp] POST https://ethereum.publicnode.com
common::http::icp] request size: 61
common::http::icp] resp size: 40b
common::http::icp] GET https://www.lightclientdata.org/eth/v1/config/spec
common::http::icp] GET https://beacon-nd-995-871-887.p2pify.com:443/c9dce41bab3e120f541e4ffb748efa60/eth/v1/config/spec
common::http::icp] resp size: 3874b
common::http::icp] GET https://www.lightclientdata.org/eth/v1/beacon/light_client/bootstrap/0x6eca17eb4352834c59af5821649985c013ab017ad956a58d1f5cd618e51a8931
common::http::icp] GET https://beacon-nd-995-871-887.p2pify.com:443/c9dce41bab3e120f541e4ffb748efa60/eth/v1/beacon/light_client/bootstrap/0x6eca17eb4352834c59af5821649985c013ab017ad956a58d1f5cd618e51a8931
common::http::icp] resp size: 54266b
common::http::icp] GET https://www.lightclientdata.org/eth/v1/beacon/light_client/updates?start_period=868&count=128
common::http::icp] GET https://beacon-nd-995-871-887.p2pify.com:443/c9dce41bab3e120f541e4ffb748efa60/eth/v1/beacon/light_client/updates?start_period=868&count=128
common::http::icp] resp size: 57122b
common::http::icp] GET https://www.lightclientdata.org/eth/v1/beacon/light_client/finality_update
common::http::icp] GET https://beacon-nd-995-871-887.p2pify.com:443/c9dce41bab3e120f541e4ffb748efa60/eth/v1/beacon/light_client/finality_update
common::http::icp] resp size: 4895b
common::http::icp] GET https://www.lightclientdata.org/eth/v1/beacon/light_client/optimistic_update
common::http::icp] GET https://beacon-nd-995-871-887.p2pify.com:443/c9dce41bab3e120f541e4ffb748efa60/eth/v1/beacon/light_client/optimistic_update
common::http::icp] resp size: 2446b
common::http::icp] GET https://www.lightclientdata.org/eth/v2/beacon/blocks/7113455
common::http::icp] GET https://beacon-nd-995-871-887.p2pify.com:443/c9dce41bab3e120f541e4ffb748efa60/eth/v2/beacon/blocks/7113455
common::http::icp] resp size: 215201b
common::http::icp] GET https://www.lightclientdata.org/eth/v2/beacon/blocks/7113376
common::http::icp] GET https://beacon-nd-995-871-887.p2pify.com:443/c9dce41bab3e120f541e4ffb748efa60/eth/v2/beacon/blocks/7113376
common::http::icp] resp size: 417564b
common::http::icp] GET https://www.lightclientdata.org/eth/v2/beacon/blocks/7113455
common::http::icp] GET https://beacon-nd-995-871-887.p2pify.com:443/c9dce41bab3e120f541e4ffb748efa60/eth/v2/beacon/blocks/7113455
common::http::icp] resp size: 215201b
common::http::icp] GET https://www.lightclientdata.org/eth/v2/beacon/blocks/7113454
... 62 others
common::http::icp] GET https://www.lightclientdata.org/eth/v2/beacon/blocks/7113391
common::http::icp] GET https://beacon-nd-995-871-887.p2pify.com:443/c9dce41bab3e120f541e4ffb748efa60/eth/v2/beacon/blocks/7113453
... 62 others
common::http::icp] GET https://beacon-nd-995-871-887.p2pify.com:443/c9dce41bab3e120f541e4ffb748efa60/eth/v2/beacon/blocks/7113446
common::http::icp] resp size: 1095012b # biggest
... 62 others
common::http::icp] resp size: 166836b # lowest
ethereum_canister] Setup instructions: 1643936212
ethereum_canister] Setup balance diff: 239862088584

# Setup 3

❯ dfx canister status ethereum_canister && dfx canister call ethereum_canister setup 'record { network = variant { Mainnet }; execution_rpc_url = "https://ethereum.publicnode.com"; consensus_rpc_url = "https://www.lightclientdata.org" }' && dfx canister status ethereum_canister

Memory Size: Nat(3118895)
Balance: 3_091_874_528_946 Cycles

Memory Size: Nat(49125167)
Balance: 2_852_620_627_135 Cycles

common::http::icp] GET https://www.lightclientdata.org/eth/v1/beacon/headers/finalized
common::http::icp] GET https://beacon-nd-995-871-887.p2pify.com:443/c9dce41bab3e120f541e4ffb748efa60/eth/v1/beacon/headers/finalized
common::http::icp] resp size: 653b
common::http::icp] POST https://ethereum.publicnode.com
common::http::icp] request size: 61
common::http::icp] resp size: 40b
common::http::icp] GET https://www.lightclientdata.org/eth/v1/config/spec
common::http::icp] GET https://beacon-nd-995-871-887.p2pify.com:443/c9dce41bab3e120f541e4ffb748efa60/eth/v1/config/spec
common::http::icp] resp size: 3874b
common::http::icp] GET https://www.lightclientdata.org/eth/v1/beacon/light_client/bootstrap/0x15ab3d4a1c4a9de6fa31147e513a0e36b1e319ee3353bd0b3992905bea17c6a7
common::http::icp] GET https://beacon-nd-995-871-887.p2pify.com:443/c9dce41bab3e120f541e4ffb748efa60/eth/v1/beacon/light_client/bootstrap/0x15ab3d4a1c4a9de6fa31147e513a0e36b1e319ee3353bd0b3992905bea17c6a7
common::http::icp] resp size: 54266b
common::http::icp] GET https://www.lightclientdata.org/eth/v1/beacon/light_client/updates?start_period=868&count=128
common::http::icp] GET https://beacon-nd-995-871-887.p2pify.com:443/c9dce41bab3e120f541e4ffb748efa60/eth/v1/beacon/light_client/updates?start_period=868&count=128
common::http::icp] resp size: 57122b
common::http::icp] GET https://www.lightclientdata.org/eth/v1/beacon/light_client/finality_update
common::http::icp] GET https://beacon-nd-995-871-887.p2pify.com:443/c9dce41bab3e120f541e4ffb748efa60/eth/v1/beacon/light_client/finality_update
common::http::icp] resp size: 4876b
common::http::icp] GET https://www.lightclientdata.org/eth/v1/beacon/light_client/optimistic_update
common::http::icp] GET https://beacon-nd-995-871-887.p2pify.com:443/c9dce41bab3e120f541e4ffb748efa60/eth/v1/beacon/light_client/optimistic_update
common::http::icp] resp size: 2446b
common::http::icp] GET https://www.lightclientdata.org/eth/v2/beacon/blocks/7113644
common::http::icp] GET https://beacon-nd-995-871-887.p2pify.com:443/c9dce41bab3e120f541e4ffb748efa60/eth/v2/beacon/blocks/7113644
common::http::icp] resp size: 783156b
common::http::icp] GET https://www.lightclientdata.org/eth/v2/beacon/blocks/7113568
common::http::icp] GET https://beacon-nd-995-871-887.p2pify.com:443/c9dce41bab3e120f541e4ffb748efa60/eth/v2/beacon/blocks/7113568
common::http::icp] resp size: 483338b
common::http::icp] GET https://www.lightclientdata.org/eth/v2/beacon/blocks/7113644
common::http::icp] GET https://beacon-nd-995-871-887.p2pify.com:443/c9dce41bab3e120f541e4ffb748efa60/eth/v2/beacon/blocks/7113644
common::http::icp] resp size: 783156b
common::http::icp] GET https://www.lightclientdata.org/eth/v2/beacon/blocks/7113643
... 62 more
common::http::icp] GET https://www.lightclientdata.org/eth/v2/beacon/blocks/7113580
common::http::icp] GET https://beacon-nd-995-871-887.p2pify.com:443/c9dce41bab3e120f541e4ffb748efa60/eth/v2/beacon/blocks/7113639
... 62 more
common::http::icp] GET https://beacon-nd-995-871-887.p2pify.com:443/c9dce41bab3e120f541e4ffb748efa60/eth/v2/beacon/blocks/7113632
common::http::icp] resp size: 136705b
... 62 more
common::http::icp] resp size: 806236b
ethereum_canister] Setup instructions: 719828620
ethereum_canister] Setup balance diff: 239231484040


# get block number
