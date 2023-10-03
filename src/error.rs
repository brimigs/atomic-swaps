use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Incorrect funds sent")]
    InaccurateFunds {},

    #[error("AlreadyFulfilled")]
    AlreadyFulfilled {},

    #[error("Unauthorized")]
    Unauthorized {},
}
