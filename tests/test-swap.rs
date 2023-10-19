use crate::helpers::{assert_err, instantiate_contract, query_balance};
use atomic_swaps_contract::error::ContractError::{InaccurateFunds, Unauthorized};
use atomic_swaps_contract::msg::{ExecuteMsg, Offer, QueryMsg};
use cosmwasm_std::{coin, CosmosMsg};
use osmosis_std::shim::Timestamp;
use osmosis_std::types::cosmos::base::v1beta1::Coin;
use osmosis_test_tube::cosmrs::proto::cosmos::authz::v1beta1::{Grant, MsgGrant};
use osmosis_test_tube::{Account, Bank, Module, OsmosisTestApp, Runner, Wasm};

use osmosis_std::types::cosmos::authz::v1beta1::MsgGrantResponse;
use osmosis_std::types::cosmos::bank::v1beta1::SendAuthorization;
use osmosis_std::types::cosmwasm::wasm::v1::{
    AllowAllMessagesFilter, ContractExecutionAuthorization, ContractGrant, MaxFundsLimit,
};
use osmosis_test_tube::cosmrs::Any;
use prost::Message;

pub mod helpers;

#[test]
fn maker_attempts_to_send_funds_before_accepted_match() {
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
                maker_coin: Coin::from(coin(1_000_000_000, "uatom")),
                taker_coin: Coin::from(coin(1_000_000_000, "uosmo")),
            },
            &[coin(1_000_000_000, "uatom")],
            &maker,
        )
        .unwrap_err();

    assert_err(res_err, InaccurateFunds {})
}

#[test]
fn account_that_isnt_contract_attempts_to_directly_execute_fulfill_offer() {
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

    // This section on granting authorization is repeated in several tests, ideally it should be extracted to utils
    // Since this was a significant part of the task, I kept the logic directly in the tests
    let grant = ContractGrant {
        contract: contract_addr.clone(),
        limit: Some(
            MaxFundsLimit {
                amounts: vec![Coin::from(coin(1_000_000_000_000_000, "uatom"))],
            }
            .to_any(),
        ),
        filter: Some(AllowAllMessagesFilter {}.to_any()),
    };

    let authz = ContractExecutionAuthorization {
        grants: vec![grant],
    };

    let grant = osmosis_std::types::cosmos::authz::v1beta1::Grant {
        authorization: Option::from(authz.to_any()),
        expiration: Some(Timestamp {
            seconds: 3150000000,
            nanos: 0,
        }),
    };

    let grant_msg = osmosis_std::types::cosmos::authz::v1beta1::MsgGrant {
        granter: maker.address().to_string(),
        grantee: contract_addr.clone().to_string(),
        grant: Some(grant),
    };

    let grant_msg2 = MsgGrant {
        granter: maker.address().to_string(),
        grantee: contract_addr.to_string(),
        grant: Some(Grant {
            authorization: Some(Any::from(
                SendAuthorization {
                    spend_limit: vec![Coin::from(coin(1_000_000_000_000_000, "uatom"))],
                }
                .to_any(),
            )),
            expiration: Some(prost_types::Timestamp {
                seconds: 3150000000,
                nanos: 0,
            }),
        }),
    };

    let msg = CosmosMsg::Stargate {
        type_url: "/cosmos.authz.v1beta1.MsgGrant".to_string(),
        value: grant_msg.encode_to_vec().into(),
    };

    let msg2 = CosmosMsg::Stargate {
        type_url: "/cosmos.authz.v1beta1.MsgGrant".to_string(),
        value: grant_msg2.encode_to_vec().into(),
    };

    app.execute_cosmos_msgs::<MsgGrantResponse>(&[msg, msg2], maker)
        .unwrap();

    wasm.execute(
        &contract_addr,
        &ExecuteMsg::MakeOffer {
            maker_coin: Coin::from(coin(1_000_000_000, "uatom")),
            taker_coin: Coin::from(coin(1_000_000_000, "uosmo")),
        },
        &[],
        &maker,
    )
    .unwrap();

    let number: i32 = 1;
    let offer_id: String = number.to_string();

    let err_res = wasm
        .execute(
            &contract_addr,
            &ExecuteMsg::FulfillOffer { offer_id },
            &[coin(1_000_000_000, "uosmo")],
            &(taker),
        )
        .unwrap_err();

    assert_err(err_res, Unauthorized {})
}

#[test]
fn invalid_offer_id_is_passed() {
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

    let grant = ContractGrant {
        contract: contract_addr.clone(),
        limit: Some(
            MaxFundsLimit {
                amounts: vec![Coin::from(coin(1_000_000_000_000_000, "uatom"))],
            }
            .to_any(),
        ),
        filter: Some(AllowAllMessagesFilter {}.to_any()),
    };

    let authz = ContractExecutionAuthorization {
        grants: vec![grant],
    };

    let grant = osmosis_std::types::cosmos::authz::v1beta1::Grant {
        authorization: Option::from(authz.to_any()),
        expiration: Some(Timestamp {
            seconds: 3150000000,
            nanos: 0,
        }),
    };

    let grant_msg = osmosis_std::types::cosmos::authz::v1beta1::MsgGrant {
        granter: maker.address().to_string(),
        grantee: contract_addr.clone().to_string(),
        grant: Some(grant),
    };

    let grant_msg2 = MsgGrant {
        granter: maker.address().to_string(),
        grantee: contract_addr.to_string(),
        grant: Some(Grant {
            authorization: Some(Any::from(
                SendAuthorization {
                    spend_limit: vec![Coin::from(coin(1_000_000_000_000_000, "uatom"))],
                }
                .to_any(),
            )),
            expiration: Some(prost_types::Timestamp {
                seconds: 3150000000,
                nanos: 0,
            }),
        }),
    };

    let msg = CosmosMsg::Stargate {
        type_url: "/cosmos.authz.v1beta1.MsgGrant".to_string(),
        value: grant_msg.encode_to_vec().into(),
    };

    let msg2 = CosmosMsg::Stargate {
        type_url: "/cosmos.authz.v1beta1.MsgGrant".to_string(),
        value: grant_msg2.encode_to_vec().into(),
    };

    app.execute_cosmos_msgs::<MsgGrantResponse>(&[msg, msg2], maker)
        .unwrap();

    wasm.execute(
        &contract_addr,
        &ExecuteMsg::MakeOffer {
            maker_coin: Coin::from(coin(1_000_000_000, "uatom")),
            taker_coin: Coin::from(coin(1_000_000_000, "uosmo")),
        },
        &[],
        &maker,
    )
    .unwrap();

    // offer id should be 1 but we are passing 2 instead
    let number: u64 = 2;
    let offer_id: String = number.to_string();

    let err_res = wasm
        .execute(
            &contract_addr,
            &ExecuteMsg::OfferTaker { offer_id },
            &[coin(1_000_000_000, "uatom")],
            &(taker),
        )
        .unwrap_err();

    assert_err(err_res, "Offer not found")
}

#[test]
fn contract_never_authorized_by_maker() {
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
            taker_coin: Coin::from(coin(1_000_000_000, "uosmo")),
        },
        &[],
        &maker,
    )
    .unwrap();

    // Since this is the only offer in storage, the offer ID will be one. To optimize this in the future, add in additional queries to check for specific maker offers.
    let number: u64 = 1;
    let offer_id: String = number.to_string();

    let res_err = wasm
        .execute(
            &contract_addr,
            &ExecuteMsg::OfferTaker { offer_id },
            &[coin(1_000_000_000, "uosmo")],
            &(taker),
        )
        .unwrap_err();

    assert_err(res_err, "authorization not found: unauthorized")
}

#[test]
fn additional_funds_are_sent_by_taker() {
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

    let grant = ContractGrant {
        contract: contract_addr.clone(),
        limit: Some(
            MaxFundsLimit {
                amounts: vec![Coin::from(coin(1_000_000_000_000_000, "uatom"))],
            }
            .to_any(),
        ),
        filter: Some(AllowAllMessagesFilter {}.to_any()),
    };

    let authz = ContractExecutionAuthorization {
        grants: vec![grant],
    };

    let grant = osmosis_std::types::cosmos::authz::v1beta1::Grant {
        authorization: Option::from(authz.to_any()),
        expiration: Some(Timestamp {
            seconds: 3150000000,
            nanos: 0,
        }),
    };

    let grant_msg = osmosis_std::types::cosmos::authz::v1beta1::MsgGrant {
        granter: maker.address().to_string(),
        grantee: contract_addr.clone().to_string(),
        grant: Some(grant),
    };

    let grant_msg2 = MsgGrant {
        granter: maker.address().to_string(),
        grantee: contract_addr.to_string(),
        grant: Some(Grant {
            authorization: Some(Any::from(
                SendAuthorization {
                    spend_limit: vec![Coin::from(coin(1_000_000_000_000_000, "uatom"))],
                }
                .to_any(),
            )),
            expiration: Some(prost_types::Timestamp {
                seconds: 3150000000,
                nanos: 0,
            }),
        }),
    };

    let msg = CosmosMsg::Stargate {
        type_url: "/cosmos.authz.v1beta1.MsgGrant".to_string(),
        value: grant_msg.encode_to_vec().into(),
    };

    let msg2 = CosmosMsg::Stargate {
        type_url: "/cosmos.authz.v1beta1.MsgGrant".to_string(),
        value: grant_msg2.encode_to_vec().into(),
    };

    app.execute_cosmos_msgs::<MsgGrantResponse>(&[msg, msg2], maker)
        .unwrap();

    wasm.execute(
        &contract_addr,
        &ExecuteMsg::MakeOffer {
            maker_coin: Coin::from(coin(1_000_000_000, "uatom")),
            taker_coin: Coin::from(coin(1_000_000_000, "uosmo")),
        },
        &[],
        &maker,
    )
    .unwrap();

    // Since this is the only offer in storage, the offer ID will be one. To optimize this in the future, add in additional queries to check for specific maker offers.
    let number: u64 = 1;
    let offer_id: String = number.to_string();

    let err_res = wasm
        .execute(
            &contract_addr,
            &ExecuteMsg::OfferTaker { offer_id },
            &[coin(1_000_000_000, "uosmo"), coin(1_000_000_000, "uatom")],
            &(taker),
        )
        .unwrap_err();

    assert_err(err_res, "sentFunds: invalid coins")
}

#[test]
fn incorrect_funds_are_sent_by_taker() {
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

    let grant = ContractGrant {
        contract: contract_addr.clone(),
        limit: Some(
            MaxFundsLimit {
                amounts: vec![Coin::from(coin(1_000_000_000_000_000, "uatom"))],
            }
            .to_any(),
        ),
        filter: Some(AllowAllMessagesFilter {}.to_any()),
    };

    let authz = ContractExecutionAuthorization {
        grants: vec![grant],
    };

    let grant = osmosis_std::types::cosmos::authz::v1beta1::Grant {
        authorization: Option::from(authz.to_any()),
        expiration: Some(Timestamp {
            seconds: 3150000000,
            nanos: 0,
        }),
    };

    let grant_msg = osmosis_std::types::cosmos::authz::v1beta1::MsgGrant {
        granter: maker.address().to_string(),
        grantee: contract_addr.clone().to_string(),
        grant: Some(grant),
    };

    let grant_msg2 = MsgGrant {
        granter: maker.address().to_string(),
        grantee: contract_addr.to_string(),
        grant: Some(Grant {
            authorization: Some(Any::from(
                SendAuthorization {
                    spend_limit: vec![Coin::from(coin(1_000_000_000_000_000, "uatom"))],
                }
                .to_any(),
            )),
            expiration: Some(prost_types::Timestamp {
                seconds: 3150000000,
                nanos: 0,
            }),
        }),
    };

    let msg = CosmosMsg::Stargate {
        type_url: "/cosmos.authz.v1beta1.MsgGrant".to_string(),
        value: grant_msg.encode_to_vec().into(),
    };

    let msg2 = CosmosMsg::Stargate {
        type_url: "/cosmos.authz.v1beta1.MsgGrant".to_string(),
        value: grant_msg2.encode_to_vec().into(),
    };

    app.execute_cosmos_msgs::<MsgGrantResponse>(&[msg, msg2], maker)
        .unwrap();

    wasm.execute(
        &contract_addr,
        &ExecuteMsg::MakeOffer {
            maker_coin: Coin::from(coin(1_000_000_000, "uatom")),
            taker_coin: Coin::from(coin(1_000_000_000, "uosmo")),
        },
        &[],
        &maker,
    )
    .unwrap();

    // Since this is the only offer in storage, the offer ID will be one. To optimize this in the future, add in additional queries to check for specific maker offers.
    let number: u64 = 1;
    let offer_id: String = number.to_string();

    let err_res = wasm
        .execute(
            &contract_addr,
            &ExecuteMsg::OfferTaker { offer_id },
            &[coin(1_000_000_000, "uatom")],
            &(taker),
        )
        .unwrap_err();

    assert_err(err_res, InaccurateFunds {})
}

#[test]
fn successful_swap() {
    let app = OsmosisTestApp::new();
    let wasm = Wasm::new(&app);
    let bank = Bank::new(&app);

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

    let grant = ContractGrant {
        contract: contract_addr.clone(),
        limit: Some(
            MaxFundsLimit {
                amounts: vec![Coin::from(coin(1_000_000_000_000_000, "uatom"))],
            }
            .to_any(),
        ),
        filter: Some(AllowAllMessagesFilter {}.to_any()),
    };

    let authz = ContractExecutionAuthorization {
        grants: vec![grant],
    };

    let grant = osmosis_std::types::cosmos::authz::v1beta1::Grant {
        authorization: Option::from(authz.to_any()),
        expiration: Some(Timestamp {
            seconds: 3150000000,
            nanos: 0,
        }),
    };

    let grant_msg = osmosis_std::types::cosmos::authz::v1beta1::MsgGrant {
        granter: maker.address().to_string(),
        grantee: contract_addr.clone().to_string(),
        grant: Some(grant),
    };

    let grant_msg2 = MsgGrant {
        granter: maker.address().to_string(),
        grantee: contract_addr.to_string(),
        grant: Some(Grant {
            authorization: Some(Any::from(
                SendAuthorization {
                    spend_limit: vec![Coin::from(coin(1_000_000_000_000_000, "uatom"))],
                }
                .to_any(),
            )),
            expiration: Some(prost_types::Timestamp {
                seconds: 3150000000,
                nanos: 0,
            }),
        }),
    };

    let msg = CosmosMsg::Stargate {
        type_url: "/cosmos.authz.v1beta1.MsgGrant".to_string(),
        value: grant_msg.encode_to_vec().into(),
    };

    let msg2 = CosmosMsg::Stargate {
        type_url: "/cosmos.authz.v1beta1.MsgGrant".to_string(),
        value: grant_msg2.encode_to_vec().into(),
    };

    app.execute_cosmos_msgs::<MsgGrantResponse>(&[msg, msg2], maker)
        .unwrap();

    wasm.execute(
        &contract_addr,
        &ExecuteMsg::MakeOffer {
            maker_coin: Coin::from(coin(1_000_000_000, "uatom")),
            taker_coin: Coin::from(coin(1_000_000_000, "uosmo")),
        },
        &[],
        &maker,
    )
    .unwrap();

    // Assert offer is properly saved and test query for taker to be able see current offers
    let open_offers: Vec<Offer> = wasm
        .query(
            &contract_addr,
            &QueryMsg::AllOffers {
                start_after: None,
                limit: None,
            },
        )
        .unwrap();

    assert_eq!(
        open_offers[0].taker_coin,
        Coin::from(coin(1_000_000_000, "uosmo"))
    );
    assert_eq!(
        open_offers[0].maker_coin,
        Coin::from(coin(1_000_000_000, "uatom"))
    );
    assert_eq!(open_offers[0].taker, None);

    // Since this is the only offer in storage, the offer ID will be one. To optimize this in the future, add in additional queries to check for specific maker offers.
    let number: u64 = 1;
    let offer_id: String = number.to_string();

    wasm.execute(
        &contract_addr,
        &ExecuteMsg::OfferTaker { offer_id },
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

    let maker_atom_balance = query_balance(&bank, &maker.address(), "uatom");
    let taker_atom_balance = query_balance(&bank, &taker.address(), "uatom");

    let maker_osmo_balance = query_balance(&bank, &maker.address(), "uosmo");
    let taker_osmo_balance = query_balance(&bank, &taker.address(), "uosmo");

    // Validate atom swap
    assert_eq!(maker_atom_balance, 999000000000);
    assert_eq!(taker_atom_balance, 1001000000000);

    // Validate osmo swap
    assert_eq!(maker_osmo_balance, 1000314570000); // 1000314570000 - 1000000000 = 999314570000
    assert_eq!(taker_osmo_balance, 998127070000); // 998127070000 + 1000000000 = 999127070000

    // Validate the Fulfilled Offers storage was accurately updated
    assert_eq!(response.maker, maker.address().to_string());
    assert_eq!(response.taker, Some(taker.address().to_string()));
    assert_eq!(
        response.maker_coin,
        Coin::from(coin(1_000_000_000, "uatom"))
    );
    assert_eq!(
        response.taker_coin,
        Coin::from(coin(1_000_000_000, "uosmo"))
    );
}
