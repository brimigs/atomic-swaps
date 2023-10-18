use crate::msg::Offer;
use cw_storage_plus::{Item, Map};

pub const OFFER_ID_COUNTER: Item<u64> = Item::new("offer_id_counter");
pub const OFFERS: Map<&str, Offer> = Map::new("offer");
pub const FULFILLED_OFFERS: Map<&str, Offer> = Map::new("fulfilled_offers");
