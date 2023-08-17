use std::cell::RefCell;

use candid::Nat;
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use ic_cdk_timers::set_timer;
use interface::{
    Address, Erc20BalanceOfRequest, Erc721OwnerOfRequest, EstimateGasRequest, Network,
    SetupRequest, U256,
};
use log::{debug, error};

use crate::stable_memory::{
    init_stable_cell_default, load_static_string, save_static_string, StableCell,
    LAST_CHECKPOINT_ID, LAST_CONSENSUS_RPC_URL_ID, LAST_EXECUTION_RPC_URL_ID, LAST_NETWORK_ID,
};
use crate::utils::IntoCallOpts;

mod erc20;
mod erc721;
mod helios;
mod random;
mod stable_memory;
mod utils;

thread_local! {
    static LAST_NETWORK: RefCell<StableCell<String>> = RefCell::new(init_stable_cell_default(LAST_NETWORK_ID));
    static LAST_CONSENSUS_RPC_URL: RefCell<StableCell<String>> = RefCell::new(init_stable_cell_default(LAST_CONSENSUS_RPC_URL_ID));
    static LAST_EXECUTION_RPC_URL: RefCell<StableCell<String>> = RefCell::new(init_stable_cell_default(LAST_EXECUTION_RPC_URL_ID));
    static LAST_CHECKPOINT: RefCell<StableCell<String>> = RefCell::new(init_stable_cell_default(LAST_CHECKPOINT_ID));
}

#[init]
async fn init() {
    ic_cdk::setup();
}

/// Setup the helios client with given node urls
///
/// Mainnet:
///   dfx canister call ethereum_canister setup \
///     'record { network = variant { Mainnet }; execution_rpc_url = "https://ethereum.publicnode.com"; consensus_rpc_url = "https://www.lightclientdata.org" }'
///
/// Goerli:
///   dfx canister call ethereum_canister setup \
///     'record { network = variant { Goerli }; execution_rpc_url = "https://ethereum-goerli.publicnode.com"; consensus_rpc_url = "TODO" }'
#[update]
async fn setup(request: SetupRequest) {
    let start = ic_cdk::api::instruction_counter();
    let start_b = ic_cdk::api::canister_balance();
    let _ = ic_logger::init_with_level(log::Level::Warn);

    helios::start_client(
        request.network,
        &request.consensus_rpc_url,
        &request.execution_rpc_url,
        None,
    )
    .await
    .expect("starting client failed");

    save_static_string(&LAST_NETWORK, request.network.to_string());
    save_static_string(&LAST_CONSENSUS_RPC_URL, request.consensus_rpc_url);
    save_static_string(&LAST_EXECUTION_RPC_URL, request.execution_rpc_url);

    let end = ic_cdk::api::instruction_counter();
    let end_b = ic_cdk::api::canister_balance();
    log::warn!("Setup instructions: {}", end - start);
    log::warn!("Setup balance diff: {}", start_b - end_b);
}

#[query]
async fn get_block_number() -> Nat {
    let start = ic_cdk::api::instruction_counter();
    let start_b = ic_cdk::api::canister_balance();

    let helios = helios::client();

    let head_block_num = helios
        .get_block_number()
        .await
        .expect("get_block_number failed");

    let res = head_block_num.into();

    let end = ic_cdk::api::instruction_counter();
    let end_b = ic_cdk::api::canister_balance();
    log::warn!("Get block number instructions: {}", end - start);
    log::warn!("Get block number balance diff: {}", start_b - end_b);

    res
}

#[query]
async fn get_gas_price() -> U256 {
    let helios = helios::client();

    let gas_price = helios.get_gas_price().await.expect("get_gas_price failed");

    gas_price.into()
}

#[update]
async fn estimate_gas(request: EstimateGasRequest) -> U256 {
    let helios = helios::client();

    let gas_cost_estimation = helios
        .estimate_gas(&request.into_call_opts())
        .await
        .expect("estimate_gas failed");

    gas_cost_estimation.into()
}

#[update]
async fn erc20_balance_of(request: Erc20BalanceOfRequest) -> U256 {
    let start = ic_cdk::api::instruction_counter();
    let start_b = ic_cdk::api::canister_balance();

    let res = erc20::balance_of(request.contract.into(), request.account.into())
        .await
        .expect("erc20::balance_of failed")
        .into();

    let end = ic_cdk::api::instruction_counter();
    let end_b = ic_cdk::api::canister_balance();
    log::warn!("Erc20 balance of instructions: {}", end - start);
    log::warn!("Erc20 balance of balance diff: {}", start_b - end_b);

    res
}

#[update]
async fn erc721_owner_of(request: Erc721OwnerOfRequest) -> Address {
    let start = ic_cdk::api::instruction_counter();
    let start_b = ic_cdk::api::canister_balance();

    let res = erc721::owner_of(request.contract.into(), request.token_id.into())
        .await
        .expect("erc721::owner_of failed")
        .into();

    let end = ic_cdk::api::instruction_counter();
    let end_b = ic_cdk::api::canister_balance();
    log::warn!("Erc721 owner of instructions: {}", end - start);
    log::warn!("Erc721 owner of balance diff: {}", start_b - end_b);

    res
}

#[pre_upgrade]
async fn pre_upgrade() {
    let start = ic_cdk::api::instruction_counter();
    let start_b = ic_cdk::api::canister_balance();

    debug!("Stopping client");

    let checkpoint = helios::get_last_checkpoint().await;
    save_static_string(&LAST_CHECKPOINT, checkpoint);

    helios::shutdown().await;

    debug!("Client stopped");

    let end = ic_cdk::api::instruction_counter();
    let end_b = ic_cdk::api::canister_balance();
    log::warn!("Pre upgrade instructions: {}", end - start);
    log::warn!("Pre upgrade balance diff: {}", start_b - end_b);
}

#[post_upgrade]
async fn post_upgrade() {
    let start = ic_cdk::api::instruction_counter();
    let start_b = ic_cdk::api::canister_balance();

    let _ = ic_logger::init_with_level(log::Level::Warn);

    // Workaround because cross-canister calls are not allowed in post_upgrade.
    // Client will be started from a timer in a second.
    set_timer(std::time::Duration::from_secs(1), move || {
        ic_cdk::spawn(async move {
            let Some(network) = load_static_string(&LAST_NETWORK) else {
                return
            };

            let Ok(network) = network.parse::<Network>() else {
                error!("Failed to parse network: {network}. Use `setup` to initalize canister.");
                return
            };

            let Some(consensus_rpc_url) = load_static_string(&LAST_CONSENSUS_RPC_URL) else {
                return
            };

            let Some(execution_rpc_url) = load_static_string(&LAST_EXECUTION_RPC_URL) else {
                return
            };

            let checkpoint = load_static_string(&LAST_CHECKPOINT);

            debug!(
                "Resuming client with: network = {}, execution_rpc_url = {}, consensus_rpc_url = {}, checkpoint: {}",
                network,
                &execution_rpc_url,
                &consensus_rpc_url,
                &checkpoint.as_deref().unwrap_or("None"),
            );

            helios::start_client(
                network,
                &consensus_rpc_url,
                &execution_rpc_url,
                checkpoint.as_deref(),
            )
            .await
            .expect("starting client failed");

            let end = ic_cdk::api::instruction_counter();
            let end_b = ic_cdk::api::canister_balance();
            log::warn!("Post upgrade instructions: {}", end - start);
            log::warn!("Post upgrade balance diff: {}", start_b - end_b);
        });
    });
}
