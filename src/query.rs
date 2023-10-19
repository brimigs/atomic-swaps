use crate::msg::Offer;
use crate::state::{FULFILLED_OFFERS, OFFERS};
use cosmwasm_std::{Deps, Order, StdError, StdResult};
use cw_storage_plus::Bound;

pub const DEFAULT_LIMIT: u32 = 10;

// Query all current offers
pub fn query_all_offers(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<Offer>> {
    let start = start_after
        .as_ref()
        .map(|denom| Bound::exclusive(denom.as_str()));
    let limit = limit.unwrap_or(DEFAULT_LIMIT) as usize;
    OFFERS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|res| Ok(res?.1))
        .collect()
}

// Query specific offers that have already been fulfilled
pub fn query_fulfilled_offers(deps: Deps, offer_id: u64) -> Result<Option<Offer>, StdError> {
    let offer_id_str = offer_id.to_string();
    let offer = FULFILLED_OFFERS.may_load(deps.storage, &offer_id_str)?;
    Ok(offer)
}
