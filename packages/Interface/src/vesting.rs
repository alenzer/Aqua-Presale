use cosmwasm_std::{Addr, Uint128, Uint64};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: Option<String>,
    pub token_addr: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    StartRelease {
        start_time: Uint128,
    },
    SetConfig {
        admin: String,
        treasury: String,
        token_addr: String,
        start_time: Uint128,
    },
    SetPrice {
        usdc_price: Uint128,
        juno_price: Uint128,
    },
    SetVestingParameters {
        params: VestingParameter,
    },
    AddUser {},
    ClaimPendingTokens {
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetConfig { },
    GetPendingTokens { wallet: Addr },
    GetUserInfo { wallet: Addr },
    GetBalance { wallet: Addr },
}

//------------Config---------------------------------------
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub treasury: String,
    pub token_addr: String,
    pub start_time: Uint128,
}

//------------Vesting parameter---------------------------------------
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Copy)]
pub struct VestingParameter {
    pub soon: Uint128,
    pub after: Uint128,
    pub period: Uint128,
}

//-------------Token holder-------------------------------------------
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserInfo {
    pub total_amount: Uint128,    //WFD token total amount that the investor buys.
    pub released_amount: Uint128, //released WFD token amount of totalAmount
}