use crate::error::ContractError;
use crate::error::ContractError::{AlreadyFulfilled, InaccurateFunds, Unauthorized};
use crate::msg::Offer;
use crate::state::{OFFERS, OFFER_ID_COUNTER, FULFILLED_OFFERS};
use cosmwasm_std::{
    to_binary, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdResult,
};
use osmosis_std::shim::{Any, Timestamp};
use osmosis_std::types::cosmos::authz::v1beta1::GrantAuthorization;
use osmosis_std::types::cosmwasm::wasm::v1::{ContractExecutionAuthorization, ContractGrant};

fn grant_authorization(
    env: Env,
    info: MessageInfo,
    coin: &Coin,
    future_time: Option<Timestamp>,
) -> StdResult<()> {
    let contract_addr = env.contract.address.to_string();

    // Populate limit and filter as needed in your use case.
    let limit = None;
    let filter = None;

    let grant = ContractGrant {
        contract: format!("cosmos.bank.v1beta1.MsgSend({})", coin.denom),
        limit,
        filter,
    };

    let authorization = ContractExecutionAuthorization {
        grants: vec![grant],
    };

    let authorization_any: Option<Any> = Some(Any {
        type_url: "/osmosis.authz.v1.ContractExecutionAuthorization".to_string(),
        value: Vec::from(to_binary(&authorization)?),
    });

    GrantAuthorization {
        granter: info.sender.to_string(),
        grantee: contract_addr,
        authorization: authorization_any,
        expiration: future_time,
    };

    Ok(())
}

pub fn make_offer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    maker_coin: Coin,
    taker_coin: Coin,
    future_time: Option<Timestamp>,
) -> Result<Response, ContractError> {
    grant_authorization(env, info.clone(), &maker_coin, future_time)?;

    if !info.funds.iter().any(|coin| *coin == maker_coin) {
        return Err(InaccurateFunds {});
    }

    // Initialize or load the offer_id counter
    let offer_id: u64 = match OFFER_ID_COUNTER.may_load(deps.storage)? {
        Some(counter) => counter + 1,
        None => {
            // If the counter doesn't exist in storage, set its initial value to 1
            OFFER_ID_COUNTER.save(deps.storage, &1)?;
            1
        }
    };

    OFFER_ID_COUNTER.save(deps.storage, &offer_id)?;

    // Store the offer
    OFFERS.save(
        deps.storage,
        &offer_id.to_string(),
        &Offer {
            maker: info.sender.to_string(),
            taker: None,
            maker_coin: maker_coin.clone(),
            taker_coin,
        },
    )?;

    Ok(Response::new().add_attribute("offer_id", offer_id.to_string()))
}

pub fn fulfill_offer(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    offer_id: String,
    taker: String,
) -> Result<Response, ContractError> {
    // Load offer
    let offer = OFFERS.load(deps.storage, &offer_id)?;

    // Validate taker
    if offer.taker.is_some() {
        return Err(AlreadyFulfilled {});
    }
    if taker != info.sender {
        return Err(Unauthorized {});
    }

    // Validate takers funds
    let has_taker_coin = info.funds.iter().any(|coin| *coin == offer.taker_coin.clone());
    if !has_taker_coin {
        return Err(InaccurateFunds {});
    }

    // Transfer coins
    let messages: Vec<CosmosMsg> = vec![
        BankMsg::Send {
            to_address: offer.maker.to_string(),
            amount: vec![offer.taker_coin.clone()],
        }
        .into(),
        BankMsg::Send {
            to_address: taker,
            amount: vec![offer.maker_coin.clone()],
        }
        .into(),
    ];

    // Mark offer as fulfilled
    FULFILLED_OFFERS.save(
        deps.storage,
        &offer_id.to_string(),
        &Offer {
            maker: offer.maker,
            taker: Some(info.sender.to_string()),
            maker_coin: offer.maker_coin.clone(),
            taker_coin: offer.taker_coin.clone(),
        },
    )?;

    // Delete offer
    OFFERS.remove(deps.storage, &offer_id);

    // Return
    Ok(Response::new().add_messages(messages))
}
