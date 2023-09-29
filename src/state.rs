use cw_storage_plus::{Item, Map};
use crate::msg::Offer;

pub const OFFER_ID_COUNTER: Item<u64> = Item::new("offer_id_counter");
pub const OFFERS: Map<&str, Offer> = Map::new("offer");