use crate::helpers::assert_err;
use atomic_swaps_contract::error::ContractError::InaccurateFunds;
use atomic_swaps_contract::msg::{ExecuteMsg, InstantiateMsg};
use cosmwasm_std::coin;
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

    let wasm_byte_code = std::fs::read("./artifacts/atomic_swaps_contract-aarch64.wasm").unwrap();
    let code_id = wasm
        .store_code(&wasm_byte_code, None, admin)
        .unwrap()
        .data
        .code_id;

    let contract_addr = wasm
        .instantiate(
            code_id,
            &InstantiateMsg {},
            None,
            Some("atomic-swaps-contract"),
            &[],
            admin,
        )
        .unwrap()
        .data
        .address;

    let res_err = wasm
        .execute(
            &contract_addr,
            &ExecuteMsg::MakeOffer(coin(1000000, "umars"), coin(1000000, "uosmo"), None),
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

    let taker_addr = taker.address();

    let wasm_byte_code = std::fs::read("./artifacts/atomic_swaps_contract-aarch64.wasm").unwrap();
    let code_id = wasm
        .store_code(&wasm_byte_code, None, admin)
        .unwrap()
        .data
        .code_id;

    let contract_addr = wasm
        .instantiate(
            code_id,
            &InstantiateMsg {},
            None,
            Some("atomic-swaps-contract"),
            &[],
            admin,
        )
        .unwrap()
        .data
        .address;

    wasm.execute(
        &contract_addr,
        &ExecuteMsg::MakeOffer(
            coin(1_000_000_000, "uatom"),
            coin(1_000_000_000, "uosmo"),
            None,
        ),
        &[coin(1_000_000_000, "uatom"), coin(1_000_000_000, "uosmo")],
        &maker,
    )
    .unwrap();

    // Since this is the only offer in storage, the offer ID will be 1. To optimize this in the future, add in additional queries to check for a specific maker offers.
    let number: u64 = 1;
    let offer_id: String = number.to_string();

    let res_err = wasm
        .execute(
            &contract_addr,
            &ExecuteMsg::FulfillOffer {
                offer_id,
                taker: taker_addr,
            },
            &[coin(1_000, "uosmo")],
            &taker,
        )
        .unwrap_err();

    assert_err(res_err, InaccurateFunds {})
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

    let wasm_byte_code = std::fs::read("./artifacts/atomic_swaps_contract-aarch64.wasm").unwrap();
    let code_id = wasm
        .store_code(&wasm_byte_code, None, admin)
        .unwrap()
        .data
        .code_id;

    let contract_addr = wasm
        .instantiate(
            code_id,
            &InstantiateMsg {},
            None,
            Some("atomic-swaps-contract"),
            &[],
            admin,
        )
        .unwrap()
        .data
        .address;

    wasm.execute(
        &contract_addr,
        &ExecuteMsg::MakeOffer(
            coin(1_000_000_000, "uatom"),
            coin(1_000_000_000, "uosmo"),
            None,
        ),
        &[coin(1_000_000_000, "uatom"), coin(1_000_000_000, "uosmo")],
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
            3,
        )
        .unwrap();
    let maker = &accs[0];
    let taker = &accs[1];
    let admin = &accs[2];

    let taker_addr = taker.address();

    let wasm_byte_code = std::fs::read("./artifacts/atomic_swaps_contract-aarch64.wasm").unwrap();
    let code_id = wasm
        .store_code(&wasm_byte_code, None, admin)
        .unwrap()
        .data
        .code_id;

    let contract_addr = wasm
        .instantiate(
            code_id,
            &InstantiateMsg {},
            None,
            Some("atomic-swaps-contract"),
            &[],
            admin,
        )
        .unwrap()
        .data
        .address;

    wasm.execute(
        &contract_addr,
        &ExecuteMsg::MakeOffer(
            coin(1_000_000_000, "uatom"),
            coin(1_000_000_000, "uosmo"),
            None,
        ),
        &[coin(1_000_000_000, "uatom"), coin(1_000_000_000, "uosmo")],
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
        &[coin(1_000_000_000, "uosmo")],
        &(taker),
    )
    .unwrap();
}
