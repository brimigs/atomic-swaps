use cosmwasm_schema::{cw_serde, QueryResponses};
use osmosis_std::types::cosmos::base::v1beta1::Coin;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    MakeOffer { maker_coin: Coin, taker_coin: Coin },
    OfferTaker { offer_id: String },
    FulfillOffer { offer_id: String },
}

#[cw_serde]
pub struct Offer {
    pub maker: String,
    pub taker: Option<String>,
    pub maker_coin: Coin,
    pub taker_coin: Coin,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Vec<Offer>)]
    AllOffers {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(Offer)]
    FulfilledOffers { offer_id: u64 },
}
