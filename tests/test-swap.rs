use crate::helpers::{assert_err, instantiate_contract};
use atomic_swaps_contract::error::ContractError::{InaccurateFunds, Unauthorized};
use atomic_swaps_contract::msg::{ExecuteMsg, Offer, QueryMsg};
use cosmwasm_std::coin;
use cw_utils::PaymentError;
use osmosis_std::types::cosmos::base::v1beta1::Coin;
use osmosis_test_tube::{Account, Module, OsmosisTestApp, Wasm};

pub mod helpers;

#[test]
fn no_funds_in_maker_account() {
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
    let admin = &accs[1];

    let contract_addr = instantiate_contract(&wasm, admin);
    let res_err = wasm
        .execute(
            &contract_addr,
            &ExecuteMsg::MakeOffer {
                maker_coin: Coin::from(coin(1000000, "umars")),
                taker_coin: coin(1000000, "uosmo"),
            },
            &[],
            &maker,
        )
        .unwrap_err();

    assert_err(res_err, InaccurateFunds {})
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
            3,
        )
        .unwrap();
    let maker = &accs[0];
    let taker = &accs[1];
    let admin = &accs[2];

    let contract_addr = instantiate_contract(&wasm, admin);

    wasm.execute(
        &contract_addr,
        &ExecuteMsg::MakeOffer {
            maker_coin: Coin::from(coin(1_000_000_000, "uatom")),
            taker_coin: coin(1_000_000_000, "uosmo"),
        },
        &[coin(1_000_000_000, "uatom")],
        &maker,
    )
    .unwrap();

    // Since this is the only offer in storage, the offer ID will be 1. To optimize this in the future, add in additional queries to check for a specific maker offers.
    let number: u64 = 1;
    let offer_id: String = number.to_string();

    let res_err = wasm
        .execute(
            &contract_addr,
            &ExecuteMsg::FulfillOffer { offer_id },
            &[coin(1_000, "uosmo")],
            &taker,
        )
        .unwrap_err();

    assert_err(res_err, InaccurateFunds {})
}

#[test]
fn additional_funds_are_sent() {
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
    let admin = &accs[1];

    let contract_addr = instantiate_contract(&wasm, admin);

    let err_res = wasm
        .execute(
            &contract_addr,
            &ExecuteMsg::MakeOffer {
                maker_coin: Coin::from(coin(1_000_000_000, "uatom")),
                taker_coin: coin(1_000_000_000, "uosmo"),
            },
            &[coin(1_000_000_000, "uatom"), coin(1_000_000_000, "uosmo")],
            &maker,
        )
        .unwrap_err();

    assert_err(err_res, InaccurateFunds {})
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
            3,
        )
        .unwrap();
    let maker = &accs[0];
    let taker = &accs[1];
    let admin = &accs[2];

    let taker_addr = taker.address();

    let contract_addr = instantiate_contract(&wasm, admin);

    wasm.execute(
        &contract_addr,
        &ExecuteMsg::MakeOffer {
            maker_coin: Coin::from(coin(1_000_000_000, "uatom")),
            taker_coin: coin(1_000_000_000, "uosmo"),
        },
        &[coin(1_000_000_000, "uatom")],
        &maker,
    )
    .unwrap();

    // Since this is the only offer in storage, the offer ID will be one. To optimize this in the future, add in additional queries to check for specific maker offers.
    let number: i32 = 2;
    let offer_id: String = number.to_string();

    // Generic error: Querier contract error: codespace: undefined, code: 1: execute wasm contract failed
    wasm.execute(
        &contract_addr,
        &ExecuteMsg::FulfillOffer { offer_id },
        &[coin(1_000_000_000_000, "uosmo")],
        &(taker),
    )
    .unwrap_err();
}

#[test]
fn unauthorized_taker() {
    let app = OsmosisTestApp::new();
    let wasm = Wasm::new(&app);

    let accs = app
        .init_accounts(
            &[
                coin(1_000_000_000_000, "uatom"),
                coin(1_000_000_000_000, "uosmo"),
            ],
            4,
        )
        .unwrap();
    let maker = &accs[0];
    let taker1 = &accs[1];
    let taker2 = &accs[2];
    let admin = &accs[3];

    let taker1_addr = taker1.address();

    let contract_addr = instantiate_contract(&wasm, admin);

    wasm.execute(
        &contract_addr,
        &ExecuteMsg::MakeOffer {
            maker_coin: Coin::from(coin(1_000_000_000, "uatom")),
            taker_coin: coin(1_000_000_000, "uosmo"),
        },
        &[coin(1_000_000_000, "uatom")],
        &maker,
    )
    .unwrap();

    // Since this is the only offer in storage, the offer ID will be one. To optimize this in the future, add in additional queries to check for specific maker offers.
    let number: i32 = 1;
    let offer_id: String = number.to_string();

    // Taker is executed on behalf of another user and fails
    let res_err = wasm
        .execute(
            &contract_addr,
            &ExecuteMsg::FulfillOffer {
                offer_id: offer_id.clone(),
            },
            &[coin(1_000_000_000, "uosmo")],
            &(taker2),
        )
        .unwrap_err();

    assert_err(res_err, "Unauthorized access");
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
            3,
        )
        .unwrap();
    let maker = &accs[0];
    let taker = &accs[1];
    let admin = &accs[2];

    let taker_addr = taker.address();

    let contract_addr = instantiate_contract(&wasm, admin);
    wasm.execute(
        &contract_addr,
        &ExecuteMsg::MakeOffer {
            maker_coin: Coin::from(coin(1_000_000_000, "uatom")),
            taker_coin: coin(1_000_000_000, "uosmo"),
        },
        &[coin(1_000_000_000, "uatom")],
        &maker,
    )
    .unwrap();

    // Since this is the only offer in storage, the offer ID will be one. To optimize this in the future, add in additional queries to check for specific maker offers.
    let number: u64 = 1;
    let offer_id: String = number.to_string();

    wasm.execute(
        &contract_addr,
        &ExecuteMsg::FulfillOffer { offer_id },
        &[coin(1_000_000_000, "uosmo")],
        &(taker),
    )
    .unwrap();

    // Query the fulfilled offer
    let response: Offer = wasm
        .query(
            &contract_addr,
            &QueryMsg::FulfilledOffers { offer_id: number },
        )
        .unwrap();

    // FIXME: Add in query balance of wallet to verify the coins were transferred

    // Validate the response
    assert_eq!(response.maker, maker.address().to_string());
    assert_eq!(response.taker, Some(taker.address().to_string()));
    assert_eq!(
        response.maker_coin,
        Coin::from(coin(1_000_000_000, "uatom"))
    );
    assert_eq!(response.taker_coin, coin(1_000_000_000, "uosmo"));
}
