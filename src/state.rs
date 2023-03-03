use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Map;

pub const FROZEN_BALANCES: Map<&Addr, Uint128> = Map::new("frozen_balances");
