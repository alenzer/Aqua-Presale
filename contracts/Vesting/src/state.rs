use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use Interface::vesting::{Config, UserInfo, VestingParameter};

pub const CONFIG: Item<Config> = Item::new("config");

pub const VEST_PARAM: Item<VestingParameter> = Item::new("vesting param");
pub const USERS: Map<Addr, UserInfo> = Map::new("users");
pub const TOTAL: Item<Uint128> = Item::new("total");

pub const USDC_PRICE: Item<Uint128> = Item::new("usdc_price");
pub const JUNO_PRICE: Item<Uint128> = Item::new("juno_price");