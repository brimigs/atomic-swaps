use crate::error::ContractError;
use crate::error::ContractError::{
    ExternalInvocation, InaccurateFunds, InvalidTaker, NoOfferFound,
};
use crate::msg::{ExecuteMsg, Offer};
use crate::state::{FULFILLED_OFFERS, MATCH_OFFER_TEMP_STORAGE, OFFERS, OFFER_ID_COUNTER};
use cosmwasm_std::{to_binary, BankMsg, Binary, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Reply, Response, SubMsg, SubMsgResult, WasmMsg};
use osmosis_std::types::cosmos::authz::v1beta1::{Grant, MsgExec, MsgGrant};
use osmosis_std::types::cosmos::bank::v1beta1::{MsgSend, SendAuthorization};
use osmosis_std::types::cosmos::base::v1beta1::Coin as Coin2;
use osmosis_std::types::cosmwasm::wasm::v1::{ContractExecutionAuthorization, ContractGrant};
use prost::Message;

fn grant_authorization(info: MessageInfo, env: Env) -> Result<Response, ContractError> {
    // Populate limit and filter as needed in your use case. For now they are set to NONE.
    // Limit defines execution limits that are enforced and updated when the grant
    let limit = None;
    // Filter define more fine-grained control on the message payload passed
    let filter = None;
    // For simplicity, expiration is set to None but can be updated based on risk requirements, user input, etc.

    let grant = ContractGrant {
        contract: env.contract.address.to_string(),
        limit,
        filter,
    };

    let authz = ContractExecutionAuthorization {
        grants: vec![grant],
    };

    let grant = Grant {
        authorization: Option::from(authz.to_any()),
        expiration: None,
    };

    let grant_msg = MsgGrant {
        granter: info.sender.to_string(),
        grantee: env.contract.address.to_string(),
        grant: Some(grant),
    };

    Ok(Response::new().add_message(grant_msg))
}

// pub fn test_make_offer(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     maker_coin: Coin2,
//     taker_coin: Coin,
// ) -> Result<Response, ContractError> {
//
// }

pub fn make_offer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    maker_coin: Coin2,
    taker_coin: Coin,
) -> Result<Response, ContractError> {
    // Validate that no funds are being sent since contract will take the funds from the account in the future
    if !info.funds.is_empty()  {
        return Err(InaccurateFunds {});
    }

    // Initialize or load the offer_id counter
    let offer_id = match OFFER_ID_COUNTER.may_load(deps.storage)? {
        Some(counter) => counter + 1,
        None => {
            // If the counter doesn't exist in storage, set its initial value to 1
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
            taker_coin: taker_coin.clone(),
        },
    )?;

    // grant_authorization(info.clone(), env.clone())?;

    let send_auth = SendAuthorization {
        spend_limit: vec![maker_coin.clone()],
    };

    let grant = Grant {
        authorization: Option::from(send_auth.to_any()),
        expiration: None,
    };

    let grant_msg = MsgGrant {
        granter: info.sender.to_string(),
        grantee: env.contract.address.to_string(),
        grant: Some(grant),
    };

    let msg = CosmosMsg::Stargate {
        type_url: "/cosmos.authz.v1beta1.MsgGrant".to_string(),
        value: grant_msg.encode_to_vec().into(),
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("offer_id", offer_id.to_string())
        .add_attribute("authorization_granted", info.sender.to_string())
        .add_attribute("maker_coin", maker_coin.denom)
        .add_attribute("taker_coin", taker_coin.denom.clone()))
}

pub fn provide_taker(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    offer_id: String,
) -> Result<Response, ContractError> {
    let offer = OFFERS.load(deps.storage, &offer_id)?;

    // Validate maker address from storage
    deps.api.addr_validate(&offer.maker)?;

    // Update offer to include new taker
    OFFERS.update(deps.storage, &offer_id, |offer| match offer {
        None => Err(NoOfferFound {}),
        Some(mut offer) => {
            offer.taker = Some(info.sender.to_string());
            Ok(offer)
        }
    })?;

    // Validate that the correct funds are being sent and ONLY the correct funds are being sent
    // Note: This could be optimized by using one_coin and payable from cw-utils
    if info.funds.len() != 1 || info.funds[0] != offer.taker_coin {
        return Err(InaccurateFunds {});
    }

    // Send funds to the contract as a submessage
    // This allows you to verify accurate funds were went to match the offer
    // and only fulfill the offer upon success
    let sub_msg = SubMsg::reply_always(
        CosmosMsg::Bank(BankMsg::Send {
            to_address: env.contract.address.to_string(),
            amount: vec![offer.taker_coin.clone()],
        }),
        TAKER_MATCH_REPLY_ID,
    );

    Ok(Response::new()
        .add_submessage(sub_msg)
        .add_attribute("taker", info.sender.to_string())
        .add_attribute("offer_id", offer_id.clone()))
}

pub fn fulfill_offer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    offer_id: String,
) -> Result<Response, ContractError> {
    if info.sender != env.contract.address {
        return Err(ExternalInvocation {});
    }

    let offer = OFFERS.load(deps.storage, &offer_id)?;

    // Handle Option<String> type
    let taker = offer.taker.clone().ok_or(InvalidTaker {});

    // Send funds on behalf of maker to taker
    let send_msg = MsgSend {
        from_address: offer.maker.clone(),
        to_address: taker?,
        amount: vec![offer.maker_coin.clone()],
    };

    let exec_msg = MsgExec {
        grantee: env.contract.address.to_string(),
        msgs: vec![send_msg.to_any()],
    };

    let msg = CosmosMsg::Stargate {
        type_url: "/cosmos.authz.v1beta1.MsgExec".to_string(),
        value: Binary::from(exec_msg),
    };

    // Send funds from contract to maker
    let bank_message: CosmosMsg = BankMsg::Send {
        to_address: offer.maker.clone(),
        amount: vec![offer.taker_coin.clone()],
    }
    .into();

    // Mark offer as fulfilled in case it needs to be referenced later
    FULFILLED_OFFERS.save(
        deps.storage,
        &offer_id.to_string(),
        &Offer {
            maker: offer.maker.clone(),
            taker: offer.taker.clone(),
            maker_coin: offer.maker_coin.clone(),
            taker_coin: offer.taker_coin.clone(),
        },
    )?;

    // Delete offer from active offers
    OFFERS.remove(deps.storage, &offer_id);

    // FIXME:: ADD attribute msg
    Ok(Response::new().add_message(msg).add_message(bank_message))
}

// Handle Reply Message
pub const TAKER_MATCH_REPLY_ID: u64 = 1;
pub fn handle_taker_match_offer_request(
    deps: DepsMut,
    env: Env,
    msg: Reply,
) -> Result<Response, ContractError> {
    match msg.result {
        SubMsgResult::Ok(_response) => {
            let offer_id = MATCH_OFFER_TEMP_STORAGE.load(deps.storage)?;

            // If correct funds are successfully sent to the contract, then initiate fulfilment
            let fulfill_msg = WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::FulfillOffer {
                    offer_id,
                })?,
                funds: vec![],
            };

            MATCH_OFFER_TEMP_STORAGE.remove(deps.storage);

            let res = Response::new()
                .add_message(fulfill_msg)
                .add_attribute("successful_match", "fulfilment_initiated");

            Ok(res)
        }
        SubMsgResult::Err(e) => {
            MATCH_OFFER_TEMP_STORAGE.remove(deps.storage);

            let res = Response::new().add_attribute("unsuccessful_match", "fulfilment_canceled");

            Ok(res)
        }
    }
}
