use crate::error::ContractError;
use crate::error::ContractError::InaccurateFunds;
use crate::msg::Offer;
use crate::state::{FULFILLED_OFFERS, OFFERS, OFFER_ID_COUNTER};
use cosmwasm_std::{BankMsg, Binary, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response};
use osmosis_std::types::cosmos::authz::v1beta1::{Grant, MsgExec, MsgGrant};
use osmosis_std::types::cosmos::bank::v1beta1::MsgSend;
use osmosis_std::types::cosmos::base::v1beta1::Coin as Coin2;
use osmosis_std::types::cosmwasm::wasm::v1::{ContractExecutionAuthorization, ContractGrant};

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

pub fn make_offer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    maker_coin: Coin2,
    taker_coin: Coin,
) -> Result<Response, ContractError> {
    // FIXME: Can use payable -> refer to redbank repo
    // FIXME: Remove funds being send and grant authz should send later
    // FIXME: update to ONE_COIN()

    // // Get coin in from the message info, error if there is not exactly one coin sent
    // let coin_in = one_coin(&info)?;

    // Validate that the correct funds are being sent and ONLY the correct funds are being sent
    if info.funds.len() != 1 || info.funds[0] != Coin::try_from(maker_coin.clone())? {
        return Err(InaccurateFunds {});
    }

    grant_authorization(info.clone(), env)?;

    // Initialize or load the offer_id counter
    let offer_id = match OFFER_ID_COUNTER.may_load(deps.storage)? {
        Some(counter) => counter + 1,
        None => {
            // If the counter doesn't exist in storage, set its initial value to 1
            1
        }
    };

    OFFER_ID_COUNTER.save(deps.storage, &offer_id)?;

    // FIXME: Run cargo clippy and cargo fmt
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

    // FIXME:: ADD attribute msg (i.e. coins you transfer)
    Ok(Response::new().add_attribute("offer_id", offer_id.to_string()))
}

pub fn fulfill_offer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    offer_id: String,
) -> Result<Response, ContractError> {
    // Load offer
    let offer = OFFERS.load(deps.storage, &offer_id)?;

    // Only the sender can be the taker
    let taker = info.sender.to_string();

    // Validate maker address from storage
    deps.api.addr_validate(&offer.maker)?;

    // Note: This could be optimized by using one_coin and payable from cw-utils
    // Validate that the correct funds are being send and ONLY the correct funds are being sent
    if info.funds.len() != 1 || info.funds[0] != offer.taker_coin {
        return Err(InaccurateFunds {});
    }

    // Handle the Option<String> type for offer.taker
    let to_address = offer.taker.unwrap_or_else(|| String::from(info.sender));

    let send_msg = MsgSend {
        from_address: offer.maker.to_string(),
        to_address,
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

    // Transfer coins from taker to maker
    let bank_message: CosmosMsg = BankMsg::Send {
        to_address: offer.maker.to_string(),
        amount: vec![offer.taker_coin.clone()],
    }
    .into();

    // Mark offer as fulfilled
    FULFILLED_OFFERS.save(
        deps.storage,
        &offer_id.to_string(),
        &Offer {
            maker: offer.maker,
            taker: Some(taker.clone()),
            maker_coin: offer.maker_coin.clone(),
            taker_coin: offer.taker_coin.clone(),
        },
    )?;

    // Delete offer
    OFFERS.remove(deps.storage, &offer_id);

    // FIXME:: ADD attribute msg
    Ok(Response::new().add_message(msg).add_message(bank_message))
}
