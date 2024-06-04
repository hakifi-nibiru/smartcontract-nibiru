use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: Addr,
}

pub const STATE: Item<State> = Item::new("state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Vault {
    pub contract_addr: String,
    pub margin_pool: Uint128,
    pub claim_pool: Uint128,
    pub hakifi_fund: Uint128,
    pub third_party_fund: Uint128
}

pub const VAULT: Item<Vault> = Item::new("vault"); 

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct  InsuranceInfor {
    pub buyer: Addr,
    pub margin: Uint128 ,
    pub claim_amount: Uint128 ,
    pub expired_time: u64, 
    pub open_time: u64 ,
    pub state: InsuranceState ,
    pub valid: bool  
}

pub const INSURANCE_INFOR: Map<String, InsuranceInfor> = Map::new("insurance_infor");

pub const MODERATOR: Item<Vec<Addr>> = Item::new("admins");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum InsuranceState {
    PENDING,
    AVAILABLE,
    CLAIMED,
    REFUNDED,
    LIQUIDATED,
    EXPIRED,
    CANCELED,
    INVALID,
}

impl std::fmt::Display for InsuranceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state_str = match self {
            InsuranceState::PENDING => "PENDING",
            InsuranceState::AVAILABLE => "AVAILABLE",
            InsuranceState::CLAIMED => "CLAIMED",
            InsuranceState::REFUNDED => "REFUNDED",
            InsuranceState::LIQUIDATED => "LIQUIDATED",
            InsuranceState::EXPIRED => "EXPIRED",
            InsuranceState::CANCELED => "CANCELED",
            InsuranceState::INVALID => "INVALID",
        };
        write!(f, "{}", state_str)
    }
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum InsuranceType {
    CREATED,
    UPDATEAVAILABLE,
    UPDATEINVALID,
    REFUND,
    CANCEL,
    CLAIM,
    EXPIRED,
    LIQUIDATED,
}

impl std::fmt::Display for InsuranceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state_str = match self {
            InsuranceType::CREATED => "CREATED",
            InsuranceType::UPDATEAVAILABLE => "UPDATE_AVAILABLE",
            InsuranceType::UPDATEINVALID => "UPDATE_INVALID",
            InsuranceType::REFUND => "REFUND",
            InsuranceType::CANCEL => "CANCEL",
            InsuranceType::EXPIRED => "EXPIRED",
            InsuranceType::CLAIM => "CLAIM",
            InsuranceType::LIQUIDATED => "LIQUIDATED",
        };
        write!(f, "{}", state_str)
    }
}