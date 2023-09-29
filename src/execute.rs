use cosmwasm_std::{BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdError, StdResult, to_binary, WasmMsg};
use osmosis_std::types::cosmos::authz::v1beta1::{GenericAuthorization, GrantAuthorization};
use osmosis_std::shim::{Any, Timestamp};
use crate::msg::{ExecuteMsg, Offer};
use crate::state::{OFFER_ID_COUNTER, OFFERS};

fn grant_authorization(
    env: Env,
    info: MessageInfo,
    coin: &Coin,
    future_time: Option<Timestamp>,
) -> StdResult<()> {
    let contract_addr = env.contract.address.to_string();

    let authorization = GenericAuthorization {
        msg: format!("cosmos.bank.v1beta1.MsgSend({})", coin.denom),
    };

    let authorization_any: Option<Any> = Some(Any {
        type_url: "/cosmos.authz.v1beta1.GenericAuthorization".to_string(),
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
) -> StdResult<Response> {
    grant_authorization(env, info.clone(), &maker_coin, future_time)?;

    // Increment the offer_id counter and use its value as the new offer_id
    let offer_id = OFFER_ID_COUNTER.update(deps.storage, |counter: u64| Ok::<_, StdError>(counter + 1))?;

    // Store the offer
    OFFERS.save(
        deps.storage,
        &info.sender.to_string(),
        &Offer {
            offer_id,
            maker: info.sender.to_string(),
            taker: None,
            maker_coin: maker_coin.clone(),
            taker_coin,
        },
    )?;

    Ok(Response::new().add_attribute("offer_id", offer_id.to_string()))
}

pub fn match_offer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    offer_id: String,
) -> StdResult<Response> {
    // Load the offer
    let offer = OFFERS.load(deps.storage, &offer_id)?;

    // Send coins
    let fulfill = WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::FulfillOffer {
            offer_id: offer_id.clone(),
            taker: info.sender.to_string(),
        })?,
        funds: vec![offer.taker_coin],
    };

    Ok(Response::new()
        .add_message(fulfill)
        .add_attributes(vec![("offer_id", offer_id)]))
}

pub fn fulfill_offer(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    offer_id: String,
    taker: String,
) -> StdResult<Response> {
    // Load offer
    let offer = OFFERS.load(deps.storage, &offer_id)?;

    // Validate taker
    if offer.taker.is_some() {
        return Err(StdError::generic_err("Offer already fulfilled"));
    }
    if taker != info.sender {
        return Err(StdError::generic_err("Invalid taker"));
    }

    // Transfer coins
    let messages: Vec<CosmosMsg> = vec![
        BankMsg::Send {
            to_address: offer.maker.to_string(),
            amount: vec![offer.taker_coin],
        }
            .into(),
        BankMsg::Send {
            to_address: taker,
            amount: vec![offer.maker_coin],
        }
            .into(),
    ];

    // Delete offer
    OFFERS.remove(deps.storage, &offer_id);

    // Return
    Ok(Response::new().add_messages(messages))
}