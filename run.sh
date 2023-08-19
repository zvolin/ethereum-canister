#!/bin/bash

set -euo pipefail

dfx start --background --clean --artificial-delay 100

dfx deploy
dfx canister call ethereum_canister setup \
  'record { network = variant { Mainnet }; execution_rpc_url = "https://ethereum.publicnode.com"; consensus_rpc_url = "https://www.lightclientdata.org" }'

for _ in $(seq 0 4); do
  dfx canister status ethereum_canister

  dfx canister call ethereum_canister erc721_owner_of \
    'record { contract = "0x5Af0D9827E0c53E4799BB226655A1de152A425a5"; token_id = 7773 }'

  dfx canister status ethereum_canister
done

dfx stop
