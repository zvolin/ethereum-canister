#!/bin/bash
#https://beacon-nd-995-871-887.p2pify.com:443/c9dce41bab3e120f541e4ffb748efa60

set -euo pipefail

# deploy_setup() {
#   sleep 5

#   dfx deploy

#   dfx canister call ethereum_canister setup \
#     'record { network = variant { Mainnet }; execution_rpc_url = "https://ethereum.publicnode.com"; consensus_rpc_url = "https://beacon-nd-995-871-887.p2pify.com:443/c9dce41bab3e120f541e4ffb748efa60" }'

#   dfx canister status ethereum_canister
# }

# deploy_setup&

# dfx start --clean --artificial-delay 100 |& {
#   count=10
#   while IFS= read -r line; do
#     echo "$line"
#     if [[ "$line" == *"Advance balance diff"* ]]; then
#       dfx canister status ethereum_canister
#       count=$((count - 1))
#       [ $count = 0 ] && break
#     fi
#   done
# }

# dfx stop

# exit


dfx start --background --clean --artificial-delay 100

dfx deploy

dfx canister call ethereum_canister setup \
  'record { network = variant { Mainnet }; execution_rpc_url = "https://ethereum.publicnode.com"; consensus_rpc_url = "https://beacon-nd-995-871-887.p2pify.com:443/c9dce41bab3e120f541e4ffb748efa60" }'

for _ in $(seq 0 4); do
  dfx canister status ethereum_canister

  dfx canister call ethereum_canister get_block_number

  dfx canister status ethereum_canister
done

dfx stop
