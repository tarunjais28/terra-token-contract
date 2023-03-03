use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

pub const FROZEN_BALANCES: Map<&Addr, Uint128> = Map::new("frozen_balances");
pub const BALANCE_CAP: Item<Uint128> = Item::new("balance_cap");
