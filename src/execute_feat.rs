use crate::error::ContractError;
use crate::state::{InsuranceInfor, INSURANCE_INFOR, MODERATOR, VAULT};
use cosmwasm_std::{
    to_json_binary, Addr, DepsMut, Env, MessageInfo, Response, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use crate::state::{InsuranceState, InsuranceType};
use cosmwasm_std::Event;

pub mod execute {
    use super::*;

    pub fn add_moderator(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        new_moderator: Addr,
    ) -> Result<Response, ContractError> {
        let mut moderators = MODERATOR.load(deps.storage)?;
        if !moderators.contains(&info.sender) {
            return Err(ContractError::Unauthorized {});
        }
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

        if !moderators.contains(&info.sender) {
            return Err(ContractError::Unauthorized {});
        }

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
        match INSURANCE_INFOR.may_load(deps.storage, id_insurance.clone())? {
            Some(_) => return Err(ContractError::AlreadyExistsInsurance {}),
            None => {},
        }
        let mut vault = VAULT.load(deps.storage)?;
        let new_insurance = InsuranceInfor {
            buyer: info.sender.clone(),
            margin,
            claim_amount: Uint128::zero(),
            expired_time: 0,
            open_time: env.block.time.seconds(),
            state: InsuranceState::PENDING,
            valid: true,
        };

        INSURANCE_INFOR.save(deps.storage, id_insurance.clone(), &new_insurance)?;

        vault.margin_pool += margin;
        VAULT.save(deps.storage, &vault)?;

        let transfer_msg = WasmMsg::Execute {
            contract_addr: vault.contract_addr.clone(),
            msg: to_json_binary(&Cw20ExecuteMsg::TransferFrom {
                owner: info.sender.to_string(),
                recipient: env.contract.address.to_string(),
                amount: margin,
            })?,
            funds: vec![],
        };
        let event = Event::new("EInsurance")
            .add_attribute("id_insurance", id_insurance.clone())
            .add_attribute("buyer", info.sender.to_string())
            .add_attribute("margin", margin)
            .add_attribute("claim_amount", Uint128::zero())
            .add_attribute("expired_time", Uint128::zero())
            .add_attribute("open_time", env.block.time.seconds().to_string())
            .add_attribute("state", InsuranceState::PENDING.to_string())
            .add_attribute("event_type", InsuranceType::CREATED.to_string());

        Ok(Response::new()
        .add_message(transfer_msg)
        .add_event(event))
    }

    pub fn update_available_insurance(
        deps: DepsMut,
        info: MessageInfo,
        id_insurance: String,
        claim_amount: Uint128,
        expired_time: u64,
    ) -> Result<Response, ContractError> {
        let admins = MODERATOR.load(deps.storage)?;
        let mut vault = VAULT.load(deps.storage)?;
        let mut insurance_info = INSURANCE_INFOR.load(deps.storage, id_insurance.clone())?;

        if !admins.contains(&info.sender) {
            return Err(ContractError::Unauthorized {});
        }

        insurance_info.state = InsuranceState::AVAILABLE;
        insurance_info.claim_amount = claim_amount;
        insurance_info.expired_time = expired_time;
        vault.claim_pool = vault.claim_pool + claim_amount;

        INSURANCE_INFOR.save(deps.storage, id_insurance.clone(), &insurance_info)?;
        VAULT.save(deps.storage, &vault)?;

        let event = Event::new("EInsurance")
            .add_attribute("id_insurance", id_insurance.clone())
            .add_attribute("buyer", insurance_info.buyer)
            .add_attribute("margin", insurance_info.margin)
            .add_attribute("claim_amount", Uint128::zero())
            .add_attribute("expired_time", expired_time.to_string())
            .add_attribute("open_time", insurance_info.open_time.to_string())
            .add_attribute("state", InsuranceState::AVAILABLE.to_string())
            .add_attribute("event_type", InsuranceType::UPDATEAVAILABLE.to_string());
        Ok(Response::new().add_event(event))
    }

    pub fn update_invalid_insurance(
        deps: DepsMut,
        info: MessageInfo,
        id_insurance: String,
    ) -> Result<Response, ContractError> {
        let admins = MODERATOR.load(deps.storage)?;
        let mut vault = VAULT.load(deps.storage)?;
        let mut insurance_info = INSURANCE_INFOR.load(deps.storage, id_insurance.clone())?;
        if !admins.contains(&info.sender) {
            return Err(ContractError::Unauthorized {});
        }

        insurance_info.state = InsuranceState::INVALID;

        let margin = insurance_info.margin;
        vault.margin_pool = vault.margin_pool - margin;

        insurance_info.margin = Uint128::zero();

        let transfer_msg = WasmMsg::Execute {
            contract_addr: vault.contract_addr.clone(),
            msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
                recipient: insurance_info.buyer.to_string(),
                amount: margin,
            })?,
            funds: vec![],
        };

        INSURANCE_INFOR.save(deps.storage, id_insurance.clone(), &insurance_info)?;
        VAULT.save(deps.storage, &vault)?;

        let event = Event::new("EInsurance")
            .add_attribute("id_insurance", id_insurance.clone())
            .add_attribute("buyer", insurance_info.buyer)
            .add_attribute("margin", Uint128::zero())
            .add_attribute("claim_amount", insurance_info.claim_amount)
            .add_attribute("expired_time", insurance_info.expired_time.to_string())
            .add_attribute("open_time", insurance_info.expired_time.to_string())
            .add_attribute("state", InsuranceState::INVALID.to_string())
            .add_attribute("event_type", InsuranceType::UPDATEINVALID.to_string());

        Ok(Response::new().add_message(transfer_msg).add_event(event))
    }

    pub fn claim_insurance(
        deps: DepsMut,
        info: MessageInfo,
        id_insurance: String,
    ) -> Result<Response, ContractError> {
        let moderators = MODERATOR.load(deps.storage)?;
        let mut insurance_info = INSURANCE_INFOR.load(deps.storage, id_insurance.clone())?;
        let mut vault = VAULT.load(deps.storage)?;
        if !moderators.contains(&info.sender) {
            return Err(ContractError::Unauthorized {});
        }

        insurance_info.state = InsuranceState::CLAIMED;

        vault.claim_pool = vault.claim_pool - insurance_info.claim_amount;

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

        INSURANCE_INFOR.save(deps.storage, id_insurance.clone(), &insurance_info)?;
        VAULT.save(deps.storage, &vault)?;

        let event = Event::new("EInsurance")
            .add_attribute("id_insurance", id_insurance.clone())
            .add_attribute("buyer", insurance_info.buyer)
            .add_attribute("margin", Uint128::zero())
            .add_attribute("claim_amount", Uint128::zero())
            .add_attribute("expired_time", insurance_info.expired_time.to_string())
            .add_attribute("open_time", insurance_info.expired_time.to_string())
            .add_attribute("state", InsuranceState::CLAIMED.to_string())
            .add_attribute("event_type", InsuranceType::CLAIM.to_string());

        Ok(Response::new().add_message(transfer_msg).add_event(event))
    }

    pub fn refund_insurance(
        deps: DepsMut,
        info: MessageInfo,
        id_insurance: String,
    ) -> Result<Response, ContractError> {
        let mut vault = VAULT.load(deps.storage)?;
        let mut insurance_info = INSURANCE_INFOR.load(deps.storage, id_insurance.clone())?;
        let moderators = MODERATOR.load(deps.storage)?;
        if !moderators.contains(&info.sender) {
            return Err(ContractError::Unauthorized {});
        }

        insurance_info.state = InsuranceState::REFUNDED;
        let margin = insurance_info.margin;

        vault.margin_pool = vault.margin_pool - margin;
        insurance_info.margin = Uint128::zero();

        let transfer_msg = WasmMsg::Execute {
            contract_addr: vault.contract_addr.clone(),
            msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
                recipient: insurance_info.buyer.to_string(),
                amount: margin,
            })?,
            funds: vec![],
        };

        INSURANCE_INFOR.save(deps.storage, id_insurance.clone(), &insurance_info)?;
        let event = Event::new("EInsurance")
            .add_attribute("id_insurance", id_insurance.clone())
            .add_attribute("buyer", insurance_info.buyer)
            .add_attribute("margin", Uint128::zero())
            .add_attribute("claim_amount", insurance_info.claim_amount)
            .add_attribute("expired_time", insurance_info.expired_time.to_string())
            .add_attribute("open_time", insurance_info.expired_time.to_string())
            .add_attribute("state", InsuranceState::REFUNDED.to_string())
            .add_attribute("event_type", InsuranceType::REFUND.to_string());

        Ok(Response::new().add_message(transfer_msg).add_event(event))
    }

    pub fn cancel_insurance(
        deps: DepsMut,
        info: MessageInfo,
        id_insurance: String,
    ) -> Result<Response, ContractError> {
        let moderators = MODERATOR.load(deps.storage)?;
        let mut vault = VAULT.load(deps.storage)?;
        let mut insurance_info = INSURANCE_INFOR.load(deps.storage, id_insurance.clone())?;

        if !moderators.contains(&info.sender) {
            return Err(ContractError::Unauthorized {});
        }

        insurance_info.state = InsuranceState::CANCELED;
        let margin = insurance_info.margin;
        vault.margin_pool = vault.margin_pool - margin;
        insurance_info.margin = Uint128::zero();

        let transfer_msg = WasmMsg::Execute {
            contract_addr: vault.contract_addr.clone(),
            msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
                recipient: insurance_info.buyer.to_string(),
                amount: margin,
            })?,
            funds: vec![],
        };
        INSURANCE_INFOR.save(deps.storage, id_insurance.clone(), &insurance_info)?;
        VAULT.save(deps.storage, &vault)?;

        let event = Event::new("EInsurance")
            .add_attribute("id_insurance", id_insurance.clone())
            .add_attribute("buyer", insurance_info.buyer)
            .add_attribute("margin", Uint128::zero())
            .add_attribute("claim_amount", insurance_info.claim_amount)
            .add_attribute("expired_time", insurance_info.expired_time.to_string())
            .add_attribute("open_time", insurance_info.expired_time.to_string())
            .add_attribute("state", InsuranceState::CANCELED.to_string())
            .add_attribute("event_type", InsuranceType::CANCEL.to_string());

        Ok(Response::new().add_message(transfer_msg).add_event(event))
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
