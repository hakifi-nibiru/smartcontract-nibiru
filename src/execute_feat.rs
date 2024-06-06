use crate::error::ContractError;
use crate::state::{InsuranceInfor, INSURANCE_INFOR, MODERATOR, VAULT, Vault};
use cosmwasm_std::{
    to_json_binary, Addr, DepsMut, Env, MessageInfo, Response, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use crate::state::{InsuranceState, InsuranceType};
use cosmwasm_std::Event;

pub mod execute {
    use super::*;

    fn check_authorization(moderators: &[Addr], sender: &Addr) -> Result<(), ContractError> {
        if !moderators.contains(sender) {
            return Err(ContractError::Unauthorized {});
        }
        Ok(())
    }

    fn save_insurance_info(
        deps: DepsMut,
        id_insurance: String,
        insurance_info: InsuranceInfor,
        vault: Vault,
        event_type: InsuranceType,
    ) -> Result<Response, ContractError> {
        INSURANCE_INFOR.save(deps.storage, id_insurance.clone(), &insurance_info)?;
        VAULT.save(deps.storage, &vault)?;

        let event = Event::new("EInsurance")
            .add_attribute("id_insurance", id_insurance.clone())
            .add_attribute("buyer", insurance_info.buyer.to_string())
            .add_attribute("margin", insurance_info.margin.to_string())
            .add_attribute("claim_amount", insurance_info.claim_amount.to_string())
            .add_attribute("expired_time", insurance_info.expired_time.to_string())
            .add_attribute("open_time", insurance_info.open_time.to_string())
            .add_attribute("state", insurance_info.state.to_string())
            .add_attribute("event_type", event_type.to_string());

        Ok(Response::new().add_event(event))
    }

    pub fn add_moderator(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        new_moderator: Addr,
    ) -> Result<Response, ContractError> {
        let mut moderators = MODERATOR.load(deps.storage)?;
        check_authorization(&moderators, &info.sender)?;

        if !moderators.contains(&new_moderator) {
            moderators.push(new_moderator);
            MODERATOR.save(deps.storage, &moderators)?;
        }

        Ok(Response::new().add_attribute("method", "add_moderator"))
    }

    pub fn delete_moderator(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        moderator_to_remove: Addr,
    ) -> Result<Response, ContractError> {
        let mut moderators = MODERATOR.load(deps.storage)?;
        check_authorization(&moderators, &info.sender)?;

        if moderators.contains(&moderator_to_remove) {
            moderators.retain(|x| x != &moderator_to_remove);
            MODERATOR.save(deps.storage, &moderators)?;
        }

        Ok(Response::new().add_attribute("method", "delete_moderator"))
    }

    pub fn create_insurance(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        id_insurance: String,
        margin: Uint128,
    ) -> Result<Response, ContractError> {
        if INSURANCE_INFOR.may_load(deps.storage, id_insurance.clone())?.is_some() {
            return Err(ContractError::AlreadyExistsInsurance {});
        }

        let mut vault = VAULT.load(deps.storage)?;
        vault.margin_pool += margin;

        let new_insurance = InsuranceInfor {
            buyer: info.sender.clone(),
            margin,
            claim_amount: Uint128::zero(),
            expired_time: 0,
            open_time: env.block.time.seconds(),
            state: InsuranceState::PENDING,
            valid: true,
        };

        let transfer_msg = WasmMsg::Execute {
            contract_addr: vault.contract_addr.clone(),
            msg: to_json_binary(&Cw20ExecuteMsg::TransferFrom {
                owner: info.sender.to_string(),
                recipient: env.contract.address.to_string(),
                amount: margin,
            })?,
            funds: vec![],
        };

        save_insurance_info(deps, id_insurance, new_insurance, vault, InsuranceType::CREATED)
            .map(|resp| resp.add_message(transfer_msg))
    }

    pub fn update_available_insurance(
        deps: DepsMut,
        info: MessageInfo,
        id_insurance: String,
        claim_amount: Uint128,
        expired_time: u64,
    ) -> Result<Response, ContractError> {
        let moderators = MODERATOR.load(deps.storage)?;
        check_authorization(&moderators, &info.sender)?;

        let mut vault = VAULT.load(deps.storage)?;
        let mut insurance_info = INSURANCE_INFOR.load(deps.storage, id_insurance.clone())?;

        insurance_info.state = InsuranceState::AVAILABLE;
        insurance_info.claim_amount = claim_amount;
        insurance_info.expired_time = expired_time;
        vault.claim_pool += claim_amount;

        save_insurance_info(deps, id_insurance, insurance_info, vault, InsuranceType::UPDATEAVAILABLE)
    }

    pub fn update_invalid_insurance(
        deps: DepsMut,
        info: MessageInfo,
        id_insurance: String,
    ) -> Result<Response, ContractError> {
        let moderators = MODERATOR.load(deps.storage)?;
        check_authorization(&moderators, &info.sender)?;

        let mut vault = VAULT.load(deps.storage)?;
        let mut insurance_info = INSURANCE_INFOR.load(deps.storage, id_insurance.clone())?;

        insurance_info.state = InsuranceState::INVALID;
        let margin = insurance_info.margin;
        vault.margin_pool -= margin;

        insurance_info.margin = Uint128::zero();

        let transfer_msg = WasmMsg::Execute {
            contract_addr: vault.contract_addr.clone(),
            msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
                recipient: insurance_info.buyer.to_string(),
                amount: margin,
            })?,
            funds: vec![],
        };

        save_insurance_info(deps, id_insurance, insurance_info, vault, InsuranceType::UPDATEINVALID)
            .map(|resp| resp.add_message(transfer_msg))
    }

    pub fn claim_insurance(
        deps: DepsMut,
        info: MessageInfo,
        id_insurance: String,
    ) -> Result<Response, ContractError> {
        let moderators = MODERATOR.load(deps.storage)?;
        check_authorization(&moderators, &info.sender)?;

        let mut insurance_info = INSURANCE_INFOR.load(deps.storage, id_insurance.clone())?;
        let mut vault = VAULT.load(deps.storage)?;

        insurance_info.state = InsuranceState::CLAIMED;
        vault.claim_pool -= insurance_info.claim_amount;

        let claim_amount = insurance_info.claim_amount;
        insurance_info.claim_amount = Uint128::zero();

        let transfer_msg = WasmMsg::Execute {
            contract_addr: vault.contract_addr.clone(),
            msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
                recipient: insurance_info.buyer.to_string(),
                amount: claim_amount,
            })?,
            funds: vec![],
        };

        save_insurance_info(deps, id_insurance, insurance_info, vault, InsuranceType::CLAIM)
            .map(|resp| resp.add_message(transfer_msg))
    }

    pub fn refund_insurance(
        deps: DepsMut,
        info: MessageInfo,
        id_insurance: String,
    ) -> Result<Response, ContractError> {
        let moderators = MODERATOR.load(deps.storage)?;
        check_authorization(&moderators, &info.sender)?;

        let mut vault = VAULT.load(deps.storage)?;
        let mut insurance_info = INSURANCE_INFOR.load(deps.storage, id_insurance.clone())?;

        insurance_info.state = InsuranceState::REFUNDED;
        let margin = insurance_info.margin;

        vault.margin_pool -= margin;
        insurance_info.margin = Uint128::zero();

        let transfer_msg = WasmMsg::Execute {
            contract_addr: vault.contract_addr.clone(),
            msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
                recipient: insurance_info.buyer.to_string(),
                amount: margin,
            })?,
            funds: vec![],
        };
        
        save_insurance_info(deps, id_insurance, insurance_info, vault, InsuranceType::REFUND)
            .map(|resp| resp.add_message(transfer_msg))
    }

    pub fn cancel_insurance(
        deps: DepsMut,
        info: MessageInfo,
        id_insurance: String,
    ) -> Result<Response, ContractError> {
        let moderators = MODERATOR.load(deps.storage)?;
        check_authorization(&moderators, &info.sender)?;

        let mut vault = VAULT.load(deps.storage)?;
        let mut insurance_info = INSURANCE_INFOR.load(deps.storage, id_insurance.clone())?;

        insurance_info.state = InsuranceState::CANCELED;
        let margin = insurance_info.margin;

        vault.margin_pool -= margin;
        insurance_info.margin = Uint128::zero();

        let transfer_msg = WasmMsg::Execute {
            contract_addr: vault.contract_addr.clone(),
            msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
                recipient: insurance_info.buyer.to_string(),
                amount: margin,
            })?,
            funds: vec![],
        };

        save_insurance_info(deps, id_insurance, insurance_info, vault, InsuranceType::CANCEL)
            .map(|resp| resp.add_message(transfer_msg))
    }

    pub fn expire_insurance(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        id_insurance: String,
    ) -> Result<Response, ContractError> {
        let moderators = MODERATOR.load(deps.storage)?;
        if !moderators.contains(&info.sender) {
            return Err(ContractError::Unauthorized {});
        }
        let mut insurance_info = INSURANCE_INFOR.load(deps.storage, id_insurance.clone())?;

        insurance_info.state = InsuranceState::EXPIRED;
        INSURANCE_INFOR.save(deps.storage, id_insurance.clone(), &insurance_info)?;

        let event = Event::new("EInsurance")
            .add_attribute("id_insurance", id_insurance.clone())
            .add_attribute("buyer", insurance_info.buyer)
            .add_attribute("margin", insurance_info.margin)
            .add_attribute("claim_amount", insurance_info.claim_amount)
            .add_attribute("expired_time", insurance_info.expired_time.to_string())
            .add_attribute("open_time", insurance_info.expired_time.to_string())
            .add_attribute("state", InsuranceState::EXPIRED.to_string())
            .add_attribute("event_type", InsuranceType::EXPIRED.to_string());

        Ok(Response::new().add_event(event))
    }
    pub fn liquidate_insurance(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        id_insurance: String,
    ) -> Result<Response, ContractError> {
        let moderators = MODERATOR.load(deps.storage)?;
        if !moderators.contains(&info.sender) {
            return Err(ContractError::Unauthorized {});
        }
        let mut insurance_info = INSURANCE_INFOR.load(deps.storage, id_insurance.clone())?;

        insurance_info.state = InsuranceState::LIQUIDATED;
        INSURANCE_INFOR.save(deps.storage, id_insurance.clone(), &insurance_info)?;

        let event = Event::new("EInsurance")
            .add_attribute("id_insurance", id_insurance.clone())
            .add_attribute("buyer", insurance_info.buyer)
            .add_attribute("margin", insurance_info.margin)
            .add_attribute("claim_amount", insurance_info.claim_amount)
            .add_attribute("expired_time", insurance_info.expired_time.to_string())
            .add_attribute("open_time", insurance_info.expired_time.to_string())
            .add_attribute("state", InsuranceState::LIQUIDATED.to_string())
            .add_attribute("event_type", InsuranceType::LIQUIDATED.to_string());

        Ok(Response::new().add_event(event))
    }
}
