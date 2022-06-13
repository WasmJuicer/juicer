use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Only accepts tokens in the cw20_whitelist")]
    NotInWhitelist {},

    #[error("Escrow is expired")]
    Expired {},

    #[error("Send some coins to create an escrow")]
    EmptyBalance {},

    #[error("Escrow id already in use")]
    AlreadyInUse {},

    #[error("Parse error: {msg}")]
    ParseError { msg: String },

    #[error("Invalid coin (this contract authorize only {amount} {denom})")]
    InvalidCoin { amount: Uint128, denom: String },

    #[error("Invalid amount (this contract authorize only {amount} {denom})")]
    InvalidAmount { amount: Uint128, denom: String },

    #[error("fees to high: {msg}")]
    FeesTooHigh { msg: String },

    #[error("Not enough founds")]
    NotEnoughFounds {},

    #[error("Commitment has already been sent")]
    DuplicatedCommitment {},

    #[error("Unknown commitment")]
    UnknownCommitment {},

    #[error("Unknown root")]
    UnknownRoot {},

    #[error("Invalid Proof")]
    InvalidProof {},
}
