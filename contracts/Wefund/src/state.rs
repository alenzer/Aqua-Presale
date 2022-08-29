use cosmwasm_std::{Addr, StdResult, Storage, Uint128, Uint64};
use cw_storage_plus::{Item, Map};

use Interface::wefund::{Config, ProjectState};

pub const CONFIG: Item<Config> = Item::new("config");

pub const PROJECT_SEQ: Item<Uint64> = Item::new("prj_seq");
pub const PROJECTSTATES: Map<u64, ProjectState> = Map::new("prj");

pub fn save_projectstate(store: &mut dyn Storage, _prj: &mut ProjectState) -> StdResult<()> {
    // increment id if exists, or return 1
    let id = PROJECT_SEQ.load(store)?;
    let id = id.checked_add(Uint64::new(1))?;
    PROJECT_SEQ.save(store, &id)?;

    _prj.project_id = id.clone();
    PROJECTSTATES.save(store, id.u64(), &_prj)
}

//------------community array------------------------------------------------
pub const COMMUNITY: Item<Vec<Addr>> = Item::new("community");

//------------Profit------------------------------------------------------------
pub const PROFIT: Item<Uint128> = Item::new("profit");

// //------------FOR REPLY-----------------------------------------
// pub const PROJECT_ID: Item<Uint128> = Item::new("project id");
// pub const AUST_AMOUNT: Item<Uint128> = Item::new("aust amount");
// pub const UUSD_AMOUNT: Item<Uint128> = Item::new("ust amount");
