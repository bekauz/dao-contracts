use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid nonce")]
    InvalidNonce {},

    #[error("Expired payload")]
    ExpiredPayload {},

    #[error("Invalid payload signature")]
    InvalidSignature {},
}
