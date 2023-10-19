use crate::error::ContractError;
use crate::execute::{fulfill_offer, make_offer, provide_taker};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::query::{query_all_offers, query_fulfilled_offers};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::MakeOffer {
            maker_coin,
            taker_coin,
        } => make_offer(deps, env, info, maker_coin, taker_coin),
        ExecuteMsg::OfferTaker { offer_id } => provide_taker(deps, env, info, offer_id),
        ExecuteMsg::FulfillOffer { offer_id } => fulfill_offer(deps, env, info, offer_id),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, StdError> {
    let res = match msg {
        QueryMsg::AllOffers { start_after, limit } => {
            to_binary(&query_all_offers(deps, start_after, limit)?)
        }
        QueryMsg::FulfilledOffers { offer_id } => {
            to_binary(&query_fulfilled_offers(deps, offer_id)?)
        }
    };
    res.map_err(Into::into)
}
