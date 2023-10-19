use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Incorrect funds sent")]
    InaccurateFunds {},

    #[error("No offer found from provided offer id")]
    NoOfferFound {},

    #[error("Fulfillment messages cannot be invoked externally")]
    ExternalInvocation {},

    #[error("Invalid taker")]
    InvalidTaker {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Reply id: {0} not valid")]
    ReplyIdError(u64),
}
