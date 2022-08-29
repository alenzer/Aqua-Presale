use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Addr, Uint64};
use cw_storage_plus::{Item, Map};
use Interface::vesting::{ProjectInfo};

pub const OWNER: Item<Addr> = Item::new("owner");

pub const PROJECT_INFOS:Map<u64, ProjectInfo> = Map::new("project_infos");
