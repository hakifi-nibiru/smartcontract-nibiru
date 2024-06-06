use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Vault, MODERATOR, VAULT};
use cosmwasm_std::{
    entry_point, DepsMut, Env, MessageInfo, Response, Uint128, Binary, Deps, StdResult,
};
use cw2::set_contract_version;
use crate::execute_feat::execute;
use crate::query_feat::query;

const CONTRACT_NAME: &str = "crates.io:hakifi";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let vault = Vault {
        contract_addr: msg.token_address,
        margin_pool: Uint128::zero(),
        claim_pool: Uint128::zero(),
        hakifi_fund: Uint128::zero(),
        third_party_fund: Uint128::zero(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    VAULT.save(deps.storage, &vault)?;

    let moderators = vec![info.sender.clone()];
    MODERATOR.save(deps.storage, &moderators)?;
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateInsurance { id_insurance, margin } => {
            execute::create_insurance(deps, env, info, id_insurance, margin)
        }
        ExecuteMsg::AddModerator { new_moderator } => {
            let addr = deps.api.addr_validate(&new_moderator)?;
            execute::add_moderator(deps, env, info, addr)
        }
        ExecuteMsg::DeleteModerator { moderator_to_remove } => {
            let addr = deps.api.addr_validate(&moderator_to_remove)?;
            execute::delete_moderator(deps, env, info, addr)
        }
        ExecuteMsg::UpdateAvailableInsurance { id_insurance, claim_amount, expired_time } => {
            execute::update_available_insurance(deps, info, id_insurance, claim_amount, expired_time)
        }
        ExecuteMsg::UpdateInvalidInsurance { id_insurance } => {
            execute::update_invalid_insurance(deps, info, id_insurance)
        }
        ExecuteMsg::ClaimInsurance { id_insurance } => {
            execute::claim_insurance(deps, info, id_insurance)
        }
        ExecuteMsg::RefundInsurance { id_insurance } => {
            execute::refund_insurance(deps, info, id_insurance)
        }
        ExecuteMsg::CancelInsurance { id_insurance } => {
            execute::cancel_insurance(deps, info, id_insurance)
        }
        ExecuteMsg::ExpireInsurance { id_insurance } => {
            execute::expire_insurance(deps, env, info, id_insurance)
        }
        ExecuteMsg::LiquidateInsurance { id_insurance } => {
            execute::liquidate_insurance(deps, env, info, id_insurance)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetInsuranceInfo { id_insurance } => {
            query::get_insurance_info(deps, id_insurance)
        }
        QueryMsg::GetVaultInfo {} => {
            query::get_vault_info(deps)
        }
    }
}
