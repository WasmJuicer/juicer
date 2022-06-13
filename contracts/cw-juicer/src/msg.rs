use juicer::msg::CircomProof;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
// use cosmwasm_std::{Addr, Api, Coin, StdResult};
use cosmwasm_std::Uint128 as U128;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub denom: String,
    pub amount: U128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositMsg {
    pub commitment: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WithdrawMsg {
    pub proof: CircomProof,

    pub root: String,
    pub nullifier_hash: String,
    pub recipient: String,
    pub relayer: String,
    pub fee: U128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Deposit(DepositMsg),
    Withdraw(WithdrawMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    IsKnownRoot(IsKnownRootMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IsKnownRootMsg {
    pub root: String,
}
