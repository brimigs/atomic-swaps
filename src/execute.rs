use crate::error::ContractError;
use crate::error::ContractError::{InaccurateFunds, InvalidTaker, NoOfferFound, Unauthorized};
use crate::msg::{ExecuteMsg, Offer};
use crate::state::{FULFILLED_OFFERS, OFFERS, OFFER_ID_COUNTER};
use cosmwasm_std::{BankMsg, Binary, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response};
use osmosis_std::types::cosmos::authz::v1beta1::MsgExec;
use osmosis_std::types::cosmos::bank::v1beta1::MsgSend;
use osmosis_std::types::cosmos::base::v1beta1::Coin as Coin2;
use osmosis_std::types::cosmwasm::wasm::v1::MsgExecuteContract;

pub fn make_offer(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    maker_coin: Coin2,
    taker_coin: Coin2,
) -> Result<Response, ContractError> {
    // Validate that no funds are being sent since contract will take the funds from the account in the future
    if !info.funds.is_empty() {
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

    Ok(Response::new()
        .add_attribute("offer_id", offer_id.to_string())
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

    // NOTE: could add in error handling to guard against already having a taker but not too urgent since offer is deleted when fulfilment msg is executed
    // Update offer to include new taker
    OFFERS.update(deps.storage, &offer_id, |offer| match offer {
        None => Err(NoOfferFound {}),
        Some(mut offer) => {
            offer.taker = Some(info.sender.to_string());
            Ok(offer)
        }
    })?;

    // Validate that the correct funds are being sent and ONLY the correct funds are being sent
    // Note: This could be optimized by using 'one_coin' and 'payable' from cw-utils but would need to update current result type
    if info.funds.len() != 1 || info.funds[0] != Coin::try_from(offer.taker_coin.clone())? {
        return Err(InaccurateFunds {});
    }

    // Now that the takers match is accepted by validating above funds,
    // the contract executes itself with the fulfilment message
    let msg = serde_json::to_vec(&ExecuteMsg::FulfillOffer {
        offer_id: offer_id.clone(),
    })
    .unwrap();

    let authz_wasm_msg = MsgExecuteContract {
        sender: env.contract.address.to_string().parse().unwrap(),
        contract: env.contract.address.to_string().parse().unwrap(),
        msg,
        funds: vec![offer.taker_coin.clone()],
    };

    Ok(Response::new()
        .add_message(authz_wasm_msg)
        .add_attribute("taker", info.sender.to_string())
        .add_attribute("offer_id", offer_id.clone()))
}

pub fn fulfill_offer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    offer_id: String,
) -> Result<Response, ContractError> {
    // fulfill offer should only be executed by the contract to ensure the offer was properly created
    // and the takers match for the offer is accurate and accepted
    if info.sender != env.contract.address {
        return Err(Unauthorized {});
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

    let coin = Coin::try_from(offer.taker_coin.clone())?;

    // Send funds from contract to maker
    let bank_message: CosmosMsg = BankMsg::Send {
        to_address: offer.maker.clone(),
        amount: vec![coin],
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

    Ok(Response::new().add_message(msg).add_message(bank_message).add_attribute("offer_fulfilled", offer_id.to_string()))
}
