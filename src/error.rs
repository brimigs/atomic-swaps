use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{reason:?}")]
    InaccurateFunds {
        reason: String,
    },

    #[error("{}")]
    AlreadyFulfilled {},

    #[error("Unauthorized")]
    Unauthorized {},
}
