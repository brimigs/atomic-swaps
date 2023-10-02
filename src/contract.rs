use crate::execute::{fulfill_offer, make_offer};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::query::query_all_offers;
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use crate::error::ContractError;

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    Ok(Response::default())
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::MakeOffer(maker_coin, taker_coin, future_time) => {
            make_offer(deps, env, info, maker_coin, taker_coin, future_time)
        }
        ExecuteMsg::FulfillOffer { offer_id, taker } => {
            fulfill_offer(deps, env, info, offer_id, taker)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, StdError> {
    let res = match msg {
        QueryMsg::AllOffers { start_after, limit } => {
            to_binary(&query_all_offers(deps, start_after, limit)?)
        }
    };
    res.map_err(Into::into)
}
