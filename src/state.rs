use cosmwasm_std::Uint64;
use cw_storage_plus::Map;

pub const RESOURCE_ID_TO_TOKEN_CONTRACT_ADDRESS: Map<&str, String> =
    Map::new("resource_id_to_token_contract_address");
pub const TOKEN_CONTRACT_ADDRESS_TO_RESOURCE_ID: Map<&str, Uint64> =
    Map::new("resource_id_to_token_contract_address");
pub const WHITELIST: Map<&str, bool> = Map::new("whitelist");
pub const BURNLIST: Map<&str, bool> = Map::new("burnlist");
