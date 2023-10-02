use crate::msg::Offer;
use crate::state::OFFERS;
use cosmwasm_std::{Deps, Order, StdResult};
use cw_storage_plus::Bound;

pub const DEFAULT_LIMIT: u32 = 10;

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
