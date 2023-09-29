use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;
use osmosis_std::shim::Timestamp;

#[cw_serde]
pub struct InstantiateMsg{}

#[cw_serde]
pub enum ExecuteMsg {
    MakeOffer(Coin, Coin, Option<Timestamp>),
    MatchOffer(String),
    FulfillOffer { offer_id: String, taker: String },
}

#[cw_serde]
pub struct Offer {
    pub offer_id: u64,
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
}