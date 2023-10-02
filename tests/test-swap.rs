use crate::helpers::{assert_err, instantiate_contract};
use atomic_swaps_contract::msg::ExecuteMsg;
use cosmwasm_std::{coin, StdError};
use osmosis_test_tube::{Account, Module, OsmosisTestApp, Wasm};

pub mod helpers;

#[test]
fn no_funds_in_maker_acccount() {
    let app = OsmosisTestApp::new();
    let wasm = Wasm::new(&app);

    let accs = app
        .init_accounts(&[coin(0, "uatom"), coin(1_000_000_000_000, "uosmo")], 1)
        .unwrap();
    let maker = &accs[0];

    let contract_addr = instantiate_contract(&wasm, maker);

    let res_err = wasm
        .execute(
            &contract_addr,
            &ExecuteMsg::MakeOffer(coin(1000000, "uatom"), coin(1000000, "uosmo"), None),
            &[],
            &maker,
        )
        .unwrap_err();
    assert_err(res_err, StdError::generic_err("Incorrect funds provided"));
}

#[test]
fn not_enough_funds_by_taker() {
    let app = OsmosisTestApp::new();
    let wasm = Wasm::new(&app);

    let accs = app
        .init_accounts(
            &[
                coin(1_000_000_000_000, "uatom"),
                coin(1_000_000_000_000, "uosmo"),
            ],
            2,
        )
        .unwrap();
    let maker = &accs[0];
    let taker = &accs[1];

    let taker_addr = taker.address();

    let contract_addr = instantiate_contract(&wasm, maker);

    wasm.execute(
        &contract_addr,
        &ExecuteMsg::MakeOffer(
            coin(1_000_000_000_000, "uatom"),
            coin(1_000_000_000_000, "uosmo"),
            None,
        ),
        &[],
        &maker,
    )
    .unwrap();

    // Since this is the only offer in storage, the offer ID will be one. To optimize this in the future, add in additional queries to check for specific maker offers.
    let number: i32 = 1;
    let offer_id: String = number.to_string();

    let res_err = wasm
        .execute(
            &contract_addr,
            &ExecuteMsg::FulfillOffer {
                offer_id,
                taker: taker_addr,
            },
            &[coin(1_000, "uosmo")],
            &(taker),
        )
        .unwrap_err();

    assert_err(res_err, StdError::generic_err("Accurate funds not sent"))
}

#[test]
fn invalid_offer_id() {
    let app = OsmosisTestApp::new();
    let wasm = Wasm::new(&app);

    let accs = app
        .init_accounts(
            &[
                coin(1_000_000_000_000, "uatom"),
                coin(1_000_000_000_000, "uosmo"),
            ],
            2,
        )
        .unwrap();
    let maker = &accs[0];
    let taker = &accs[1];

    let taker_addr = taker.address();

    let contract_addr = instantiate_contract(&wasm, maker);

    wasm.execute(
        &contract_addr,
        &ExecuteMsg::MakeOffer(
            coin(1_000_000_000_000, "uatom"),
            coin(1_000_000_000_000, "uosmo"),
            None,
        ),
        &[],
        &maker,
    )
    .unwrap();

    // Since this is the only offer in storage, the offer ID will be one. To optimize this in the future, add in additional queries to check for specific maker offers.
    let number: i32 = 2;
    let offer_id: String = number.to_string();

    // Generic error: Querier contract error: codespace: undefined, code: 1: execute wasm contract failed
    wasm.execute(
        &contract_addr,
        &ExecuteMsg::FulfillOffer {
            offer_id,
            taker: taker_addr,
        },
        &[coin(1_000_000_000_000, "uosmo")],
        &(taker),
    )
    .unwrap_err();
}

#[test]
fn successful_swap() {
    let app = OsmosisTestApp::new();
    let wasm = Wasm::new(&app);

    let accs = app
        .init_accounts(
            &[
                coin(1_000_000_000_000, "uatom"),
                coin(1_000_000_000_000, "uosmo"),
            ],
            2,
        )
        .unwrap();
    let maker = &accs[0];
    let taker = &accs[1];

    let taker_addr = taker.address();

    let contract_addr = instantiate_contract(&wasm, maker);

    wasm.execute(
        &contract_addr,
        &ExecuteMsg::MakeOffer(
            coin(1_000_000_000_000, "uatom"),
            coin(1_000_000_000_000, "uosmo"),
            None,
        ),
        &[],
        &maker,
    )
    .unwrap();

    // Since this is the only offer in storage, the offer ID will be one. To optimize this in the future, add in additional queries to check for specific maker offers.
    let number: i32 = 1;
    let offer_id: String = number.to_string();

    wasm.execute(
        &contract_addr,
        &ExecuteMsg::FulfillOffer {
            offer_id,
            taker: taker_addr,
        },
        &[coin(1_000_000_000_000, "uosmo")],
        &(taker),
    )
    .unwrap();
}
