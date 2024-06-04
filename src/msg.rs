use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;
#[cw_serde]
pub struct InstantiateMsg {
    pub token_address: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddModerator { new_moderator: String },
    DeleteModerator { moderator_to_remove: String },
    CreateInsurance {
        id_insurance: String,
        margin: Uint128,
    },
    UpdateAvailableInsurance {
        id_insurance: String,
        claim_amount: Uint128,
        expired_time: u64,
    },
    UpdateInvalidInsurance {
        id_insurance: String,
    },
    ClaimInsurance {
        id_insurance: String,
    },
    RefundInsurance {
        id_insurance: String,
    },
    CancelInsurance {
        id_insurance: String,
    },
    ExpireInsurance {
        id_insurance: String,
    },
    LiquidateInsurance {
        id_insurance: String,
    },
}

#[cw_serde]
pub enum QueryMsg {
    GetInsuranceInfo { id_insurance: String },
}
