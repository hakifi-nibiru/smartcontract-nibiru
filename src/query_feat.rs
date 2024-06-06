use crate::state::INSURANCE_INFOR;
use crate::state::VAULT;
use cosmwasm_std::to_json_binary;
use cosmwasm_std::{Binary, Deps, StdResult};

pub mod query {

    use super::*;
    pub fn get_insurance_info(deps: Deps, id_insurance: String) -> StdResult<Binary> {
        let insurance_info = INSURANCE_INFOR.load(deps.storage, id_insurance)?;
        to_json_binary(&insurance_info)
    }

    pub fn get_vault_info(deps: Deps) -> StdResult<Binary> {
        let vault_info = VAULT.load(deps.storage)?;
        to_json_binary(&vault_info)
    }
}

