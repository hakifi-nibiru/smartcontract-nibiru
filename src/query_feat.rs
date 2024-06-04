use crate::state::INSURANCE_INFOR;
use cosmwasm_std::to_json_binary;
use cosmwasm_std::{Binary, Deps, StdResult};

pub mod query {
    use super::*;
    pub fn get_insurance_info(deps: Deps, id_insurance: String) -> StdResult<Binary> {
        let insurance_info = INSURANCE_INFOR.load(deps.storage, id_insurance)?;
        to_json_binary(&insurance_info)
    }
}

