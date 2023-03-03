#[cfg(not(feature = "library"))]
#[macro_use]
extern crate arrayref;

mod data;
mod error;
mod operations;
pub mod state;

pub mod contract;
pub mod msg;
#[cfg(test)]
mod tests;

pub use crate::{error::*, operations::*, state::*};
use codec::{Decode, Encode};
use cosmwasm_std::Uint64;
use cosmwasm_std::{DepsMut, Response};
use serde::{Deserialize, Serialize};
pub type Bytes = Vec<u8>;

pub fn add_to_list(
    deps: DepsMut,
    address: String,
    list_type: ListType,
) -> Result<Response, ContractError> {
    match list_type {
        ListType::WhiteList => WHITELIST.save(deps.storage, &address, &true)?,
        ListType::BurnList => BURNLIST.save(deps.storage, &address, &true)?,
    }

    Ok(Response::new()
        .add_attribute("action", list_type.clone().get_action())
        .add_attribute(list_type.get_addr_type(), address))
}

pub fn set_resource_id(
    deps: DepsMut,
    resource_id: Uint64,
    address: String,
) -> Result<Response, ContractError> {
    RESOURCE_ID_TO_TOKEN_CONTRACT_ADDRESS.save(
        deps.storage,
        &resource_id.to_string(),
        &address.to_string(),
    )?;
    TOKEN_CONTRACT_ADDRESS_TO_RESOURCE_ID.save(deps.storage, &address, &resource_id)?;

    Ok(Response::new()
        .add_attribute("action", "set_resource_id")
        .add_attribute("resource_id", resource_id)
        .add_attribute("contract_address", address))
}
