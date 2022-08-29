#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
   to_binary, Addr, AllBalanceResponse, BalanceResponse, BankMsg, BankQuery, Coin, CosmosMsg,
   DepsMut, Env, MessageInfo, QueryRequest, Response, StdResult, SubMsg, Uint128, Uint64, WasmMsg,
};
use cw2::set_contract_version;
use cw20::{
   BalanceResponse as Cw20BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg, TokenInfoResponse,
};

use crate::error::ContractError;
use crate::state::{
   save_projectstate,
   COMMUNITY,
   CONFIG,
   PROFIT,
   PROJECTSTATES,
   PROJECT_SEQ,
   // UUSD_AMOUNT,AUST_AMOUNT, PROJECT_ID,
};
use Interface::wefund::{
   BackerState, Config, ExecuteMsg, InstantiateMsg, Milestone, ProjectState, ProjectStatus,
   TeamMember, VestingParameter, Vote, WhitelistState,
};

use Interface::staking::CardType;
use Interface::vesting::{ExecuteMsg as VestingMsg, VestingParameter as VestingParam};

// version info for migration info
const CONTRACT_NAME: &str = "WEFUND";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const UST: u128 = 1_000_000; //ust unit

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
   deps: DepsMut,
   _env: Env,
   info: MessageInfo,
   msg: InstantiateMsg,
) -> Result<Response, ContractError> {
   set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

   let owner = msg
      .admin
      .and_then(|s| deps.api.addr_validate(s.as_str()).ok())
      .unwrap_or(info.sender.clone());

   let wefund = msg
      .wefund
      .and_then(|s| deps.api.addr_validate(s.as_str()).ok())
      .unwrap_or(info.sender.clone());

   let denom = msg.denom.unwrap_or(
      "ibc/EAC38D55372F38F1AFD68DF7FE9EF762DCF69F26520643CF3F9D292A738D8034".to_string(),
   );

   let decimals = msg.decimals.unwrap_or(Uint64::new(6)).u64() as u32;

   let vesting_contract = msg
      .vesting_contract
      .and_then(|s| deps.api.addr_validate(s.as_str()).ok())
      .unwrap_or(Addr::unchecked("".to_string()));

   let config = Config {
      owner,
      wefund,
      denom,
      decimals,
      vesting_contract,
   };

   CONFIG.save(deps.storage, &config)?;
   PROJECT_SEQ.save(deps.storage, &Uint64::zero())?;
   COMMUNITY.save(deps.storage, &Vec::new())?;

   // AUST_AMOUNT.save(deps.storage, &Uint128::zero())?;
   // UUSD_AMOUNT.save(deps.storage, &Uint128::zero())?;
   // PROJECT_ID.save(deps.storage, &Uint128::zero())?;

   PROFIT.save(deps.storage, &Uint128::zero())?;

   Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
   deps: DepsMut,
   _env: Env,
   info: MessageInfo,
   msg: ExecuteMsg,
) -> Result<Response, ContractError> {
   match msg {
      ExecuteMsg::SetConfig {
         admin,
         wefund,
         denom,
         decimals,
         vesting_contract,
      } => try_setconfig(
         deps,
         _env,
         info,
         admin,
         wefund,
         denom,
         decimals,
         vesting_contract,
      ),
      ExecuteMsg::AddProject {
         project_id,
         project_company,
         project_title,
         project_description,
         project_ecosystem,
         project_fundtype,
         project_createddate,
         project_saft,
         project_logo,
         project_whitepaper,
         project_website,
         project_email,
         creator_wallet,
         project_collected,
         project_milestones,
         project_teammembers,
         vesting,
         token_addr,
         country,
         cofounder_name,
         service_wefund,
         service_charity,
         professional_link,
      } => try_addproject(
         deps,
         _env,
         info,
         project_id,
         project_company,
         project_title,
         project_description,
         project_ecosystem,
         project_fundtype,
         project_createddate,
         project_saft,
         project_logo,
         project_whitepaper,
         project_website,
         project_email,
         creator_wallet,
         project_collected,
         project_milestones,
         project_teammembers,
         vesting,
         token_addr,
         country,
         cofounder_name,
         service_wefund,
         service_charity,
         professional_link,
      ),
      ExecuteMsg::Back2ProjectWithout {
         project_id,
         backer_wallet,
         denom,
         amount,
         fundraising_stage,
         token_amount,
         otherchain,
         otherchain_wallet,
      } => try_back2projectwithout(
         deps,
         _env,
         info,
         project_id,
         backer_wallet,
         denom,
         amount,
         fundraising_stage,
         token_amount,
         otherchain,
         otherchain_wallet,
      ),
      ExecuteMsg::Back2Project {
         project_id,
         backer_wallet,
         fundraising_stage,
         token_amount,
         otherchain,
         otherchain_wallet,
      } => try_back2project(
         deps,
         _env,
         info,
         project_id,
         backer_wallet,
         fundraising_stage,
         token_amount,
         otherchain,
         otherchain_wallet,
      ),

      ExecuteMsg::CompleteProject { project_id } => try_completeproject(deps, _env, project_id),

      ExecuteMsg::FailProject { project_id } => try_failproject(deps, _env, project_id),

      ExecuteMsg::RemoveProject { project_id } => try_removeproject(deps, info, project_id),

      ExecuteMsg::TransferAllCoins { wallet } => try_transferallcoins(deps, _env, info, wallet),

      ExecuteMsg::AddCommunitymember { wallet } => try_addcommunitymember(deps, wallet),

      ExecuteMsg::RemoveCommunitymember { wallet } => try_removecommunitymember(deps, wallet),

      ExecuteMsg::WefundApprove { project_id } => try_wefundapprove(deps, info, project_id),

      ExecuteMsg::SetFundraisingStage { project_id, stage } => {
         try_setfundraisingstage(deps, project_id, stage)
      }

      ExecuteMsg::SetMilestoneVote {
         project_id,
         wallet,
         voted,
      } => try_setmilestonevote(deps, _env, info, project_id, wallet, voted),

      ExecuteMsg::ReleaseMilestone { project_id } => try_releasemilestone(deps, _env, project_id),

      ExecuteMsg::SetProjectStatus { project_id, status } => {
         try_setprojectstatus(deps, info, project_id, status)
      }

      ExecuteMsg::OpenWhitelist {
         project_id,
         holder_alloc,
      } => try_openwhitelist(deps, _env, info, project_id, holder_alloc),

      ExecuteMsg::RegisterWhitelist {
         project_id,
         card_type,
      } => try_registerwhitelist(deps, _env, info, project_id, card_type),

      ExecuteMsg::CloseWhitelist { project_id } => try_closewhitelist(deps, _env, info, project_id),
   }
}
pub fn try_setprojectstatus(
   deps: DepsMut,
   info: MessageInfo,
   project_id: Uint64,
   status: Uint128,
) -> Result<Response, ContractError> {
   //-----------check owner--------------------------
   let config = CONFIG.load(deps.storage).unwrap();
   if info.sender != config.owner {
      return Err(ContractError::Unauthorized {});
   }
   //    let x:ProjectState = PROJECTSTATES.load(deps.storage, _project_id.u64())?;
   //-------update-------------------------
   PROJECTSTATES.update(deps.storage, project_id.u64(), |op| match op {
      None => Err(ContractError::NotRegisteredProject {}),
      Some(mut project) => {
         if status == Uint128::zero() {
            project.project_status = ProjectStatus::WefundVote;
         } else if status == Uint128::from(1u64) {
            project.project_status = ProjectStatus::Whitelist;
         } else if status == Uint128::from(2u64) {
            project.project_status = ProjectStatus::Fundraising;
         } else if status == Uint128::from(3u64) {
            project.project_status = ProjectStatus::Releasing;
         } else if status == Uint128::from(4u64) {
            project.project_status = ProjectStatus::Done;
         } else if status == Uint128::from(5u64) {
            project.project_status = ProjectStatus::Fail;
         }
         Ok(project)
      }
   })?;
   Ok(Response::new().add_attribute("action", "Set project status"))
}
pub fn convert_str_int(str: String) -> u128 {
   let bytes = str.into_bytes();
   let mut res: u128 = 0;
   let mut dot = false;
   let mut dotbelow = 0;

   for i in 0..bytes.len() {
      if bytes[i] < 48 {
         dot = true;
      } else if dotbelow < 6 {
         res = res * 10 + (bytes[i] - 48) as u128;
         if dot {
            dotbelow += 1;
         }
      }
   }
   return res;
}
pub fn try_releasemilestone(
   deps: DepsMut,
   _env: Env,
   _project_id: Uint64,
) -> Result<Response, ContractError> {
   //--------Get project info----------------------------
   let mut x: ProjectState = PROJECTSTATES.load(deps.storage, _project_id.u64())?;

   //--------Checking project status-------------------------
   if x.project_status != ProjectStatus::Releasing {
      //only releasing status
      return Err(ContractError::NotCorrectStatus {
         status: x.project_status as u32,
      });
   }

   //---------get hope to release amount---------------------------
   let config = CONFIG.load(deps.storage).unwrap();
   let step = x.project_milestonestep.u128() as usize;
   let release_amount =
      x.milestone_states[step].milestone_amount.u128() * (100u128).pow(config.decimals);

   let coin = Coin::new(release_amount, config.denom);
   let send2_creator = BankMsg::Send {
      to_address: x.creator_wallet.to_string(),
      amount: vec![coin],
   };

   x.milestone_states[step].milestone_status = Uint128::new(2); //switch to released status
   x.project_milestonestep += Uint128::new(1); //switch to next milestone step
                                               //-----------check milestone done---------------------
   if x.project_milestonestep >= Uint128::new(x.milestone_states.len() as u128) {
      x.project_status = ProjectStatus::Done; //switch to project done status
   }

   x.backerbacked_amount -= Uint128::from(release_amount);
   PROJECTSTATES.save(deps.storage, _project_id.u64(), &x)?;

   Ok(Response::new()
      .add_message(send2_creator)
      .add_attribute("action", "release milestone"))
}
pub fn try_setmilestonevote(
   deps: DepsMut,
   _env: Env,
   info: MessageInfo,
   project_id: Uint64,
   wallet: String,
   voted: bool,
) -> Result<Response, ContractError> {
   let mut x: ProjectState = PROJECTSTATES.load(deps.storage, project_id.u64())?;
   //-------check project status-------------------
   if x.project_status != ProjectStatus::Releasing {
      //only releasing status
      return Err(ContractError::NotCorrectStatus {
         status: x.project_status as u32,
      });
   }

   let wallet = deps.api.addr_validate(&wallet).unwrap();
   let step = x.project_milestonestep.u128() as usize;

   if x.milestone_states[step].milestone_status != Uint128::zero() {
      //only voting status
      return Err(ContractError::NotCorrectMilestoneStatus {
         step: step,
         status: x.milestone_states[step].milestone_status,
      });
   }

   //------set vot status and check all voted for same backer--------------------
   let mut all_voted = true;
   for vote in x.milestone_states[step].milestone_votes.iter_mut() {
      if vote.wallet == wallet {
         vote.voted = voted;
      }
      all_voted = all_voted & vote.voted;
   }

   let mut deps = deps;
   if all_voted {
      x.milestone_states[step].milestone_status = Uint128::new(1); //switch to releasing status

      //-------update-------------------------
      PROJECTSTATES.update(deps.storage, project_id.u64(), |op| match op {
         None => Err(ContractError::NotRegisteredProject {}),
         Some(mut project) => {
            project.milestone_states = x.milestone_states;
            Ok(project)
         }
      })?;

      return try_releasemilestone(deps.branch(), _env, project_id);
   }
   //-------update-------------------------
   PROJECTSTATES.update(deps.storage, project_id.u64(), |op| match op {
      None => Err(ContractError::NotRegisteredProject {}),
      Some(mut project) => {
         project.milestone_states = x.milestone_states;
         project.project_milestonestep = x.project_milestonestep;
         project.project_status = x.project_status;
         Ok(project)
      }
   })?;

   Ok(Response::new().add_attribute("action", "Set milestone vote"))
}

pub fn try_setfundraisingstage(
   deps: DepsMut,
   project_id: Uint64,
   stage: Uint128,
) -> Result<Response, ContractError> {
   PROJECTSTATES.update(deps.storage, project_id.u64(), |op| match op {
      None => Err(ContractError::NotRegisteredProject {}),
      Some(mut project) => {
         project.fundraising_stage = stage;
         Ok(project)
      }
   })?;

   Ok(Response::new().add_attribute("action", "Set Fundraising stage"))
}

pub fn try_wefundapprove(
   deps: DepsMut,
   info: MessageInfo,
   project_id: Uint64,
) -> Result<Response, ContractError> {
   //-----------check owner--------------------------
   let config = CONFIG.load(deps.storage).unwrap();
   if info.sender != config.owner {
      return Err(ContractError::Unauthorized {});
   }

   let mut x: ProjectState = PROJECTSTATES.load(deps.storage, project_id.u64())?;
   //-------check project status-------------------
   if x.project_status != ProjectStatus::WefundVote {
      //only wefund approve status
      return Err(ContractError::NotCorrectStatus {
         status: x.project_status as u32,
      });
   }
   x.project_status = ProjectStatus::Whitelist; //switch to fundraising status

   PROJECTSTATES.update(deps.storage, project_id.u64(), |op| match op {
      None => Err(ContractError::NotRegisteredProject {}),
      Some(mut project) => {
         project.project_status = x.project_status;
         Ok(project)
      }
   })?;

   Ok(Response::new().add_attribute("action", "Wefund Approve"))
}

pub fn try_removecommunitymember(deps: DepsMut, wallet: String) -> Result<Response, ContractError> {
   let wallet = deps.api.addr_validate(&wallet).unwrap();

   let mut community = COMMUNITY.load(deps.storage).unwrap();
   let res = community.iter().find(|&x| x == &wallet);
   if res == None {
      return Err(ContractError::NotRegisteredCommunity {});
   }

   community.retain(|x| x != &wallet);
   COMMUNITY.save(deps.storage, &community)?;

   Ok(Response::new().add_attribute("action", "remove community member"))
}

pub fn try_addcommunitymember(deps: DepsMut, wallet: String) -> Result<Response, ContractError> {
   let wallet = deps.api.addr_validate(&wallet).unwrap();

   let mut community = COMMUNITY.load(deps.storage).unwrap();
   let res = community.iter().find(|&x| x == &wallet);
   if res != None {
      return Err(ContractError::AlreadyRegisteredCommunity {});
   }

   community.push(wallet);
   COMMUNITY.save(deps.storage, &community)?;

   Ok(Response::new().add_attribute("action", "add community member"))
}
pub fn try_transferallcoins(
   deps: DepsMut,
   _env: Env,
   info: MessageInfo,
   wallet: String,
) -> Result<Response, ContractError> {
   //-----------check owner--------------------------
   let config = CONFIG.load(deps.storage).unwrap();
   if info.sender != config.owner {
      return Err(ContractError::Unauthorized {});
   }
   //--------get all native coins and ust - 4 ----------------------
   let balance: AllBalanceResponse =
      deps
         .querier
         .query(&QueryRequest::Bank(BankQuery::AllBalances {
            address: _env.contract.address.to_string(),
         }))?;

   let bank_native = BankMsg::Send {
      to_address: wallet.clone(),
      amount: balance.amount,
   };

   Ok(Response::new()
      .add_message(CosmosMsg::Bank(bank_native))
      .add_attribute("action", "trasnfer all coins"))
}
pub fn try_removeproject(
   deps: DepsMut,
   info: MessageInfo,
   project_id: Uint64,
) -> Result<Response, ContractError> {
   //-----------check owner--------------------------
   let config = CONFIG.load(deps.storage).unwrap();
   if info.sender != config.owner {
      return Err(ContractError::Unauthorized {});
   }
   let res = PROJECTSTATES.may_load(deps.storage, project_id.u64());
   if res == Ok(None) {
      return Err(ContractError::NotRegisteredProject {});
   }
   PROJECTSTATES.remove(deps.storage, project_id.u64());

   Ok(Response::new())
}

pub fn try_setconfig(
   deps: DepsMut,
   _env: Env,
   info: MessageInfo,
   admin: Option<String>,
   wefund: Option<String>,
   denom: Option<String>,
   decimals: Option<Uint64>,
   vesting_contract: Option<String>,
) -> Result<Response, ContractError> {
   //-----------check owner--------------------------
   let config = CONFIG.load(deps.storage).unwrap();
   if info.sender != config.owner {
      return Err(ContractError::Unauthorized {});
   }
   let mut config = CONFIG.load(deps.storage).unwrap();

   config.owner = admin
      .and_then(|s| deps.api.addr_validate(s.as_str()).ok())
      .unwrap_or(config.owner);

   config.wefund = wefund
      .and_then(|s| deps.api.addr_validate(s.as_str()).ok())
      .unwrap_or(config.wefund);

   config.denom = denom.unwrap_or(config.denom);

   config.decimals = decimals.unwrap_or(Uint64::from(config.decimals)).u64() as u32;

   config.vesting_contract = vesting_contract
      .and_then(|s| deps.api.addr_validate(s.as_str()).ok())
      .unwrap_or(config.vesting_contract);

   CONFIG.save(deps.storage, &config)?;

   Ok(Response::new().add_attribute("action", "SetConfig"))
}
pub fn try_completeproject(
   deps: DepsMut,
   _env: Env,
   _project_id: Uint64,
) -> Result<Response, ContractError> {
   //--------Get project info----------------------------
   let mut x: ProjectState = PROJECTSTATES.load(deps.storage, _project_id.u64())?;

   //--------Checking project status-------------------------
   if x.project_status != ProjectStatus::Releasing {
      //only releasing status
      return Err(ContractError::NotCorrectStatus {
         status: x.project_status as u32,
      });
   }

   let config = CONFIG.load(deps.storage).unwrap();
   let release_amount: u128 = x.backerbacked_amount.u128();

   let coin = Coin::new(release_amount, config.denom);
   let send2_creator = BankMsg::Send {
      to_address: x.creator_wallet.to_string(),
      amount: vec![coin],
   };

   x.backerbacked_amount = Uint128::zero();
   x.project_status = ProjectStatus::Done;
   PROJECTSTATES.save(deps.storage, _project_id.u64(), &x)?;

   Ok(Response::new()
      .add_message(send2_creator)
      .add_attribute("action", "complete milestone")
      .add_attribute("withdraw aust amount", release_amount.to_string()))
}
pub fn try_failproject(
   deps: DepsMut,
   _env: Env,
   _project_id: Uint64,
) -> Result<Response, ContractError> {
   //--------Get project info----------------------------
   let x: ProjectState = PROJECTSTATES.load(deps.storage, _project_id.u64())?;

   //--------Checking project status-------------------------
   if x.project_status != ProjectStatus::Releasing {
      //only releasing status
      return Err(ContractError::NotCorrectStatus {
         status: x.project_status as u32,
      });
   }

   // let config = CONFIG.load(deps.storage).unwrap();
   // let mut release_amount: u128 = x.backerbacked_amount.u128();
   // //---------send to backer wallet-------------
   // let mut msg = Vec::new();
   // for backer in x.backer_states.clone() {
   //    let mut backed = backer.amount.clone();

   //    //---while mistone releasing, suddenly failed, distribute with %
   //    backed.amount =
   //       backer.amount.amount * withdraw_amount / x.backerbacked_amount.clone();

   //    let send2_backer = BankMsg::Send {
   //       to_address: backer.backer_wallet.to_string(),
   //       amount: vec![backed],
   //    };
   //    msg.push(CosmosMsg::Bank(send2_backer));
   // }

   Ok(Response::new()
      // .add_submessage(withdraw)
      .add_attribute("action", "failed milestone"))
}

pub fn try_addproject(
   deps: DepsMut,
   _env: Env,
   _info: MessageInfo,
   _project_id: Uint64,
   _project_company: String,
   _project_title: String,
   _project_description: String,
   _project_ecosystem: String,
   _project_fundtype: String,
   _project_createddate: String,
   _project_saft: String,
   _project_logo: String,
   _project_whitepaper: String,
   _project_website: String,
   _project_email: String,
   _creator_wallet: String,
   _project_collected: Uint128,
   _project_milestones: Vec<Milestone>,
   _project_teammembers: Vec<TeamMember>,
   _vesting: Vec<VestingParameter>,
   _token_addr: String,

   _country: String,
   _cofounder_name: String,
   _service_wefund: String,
   _service_charity: String,
   _professional_link: String,
) -> Result<Response, ContractError> {
   let token_addr = deps
      .api
      .addr_validate(_token_addr.as_str())
      .unwrap_or(Addr::unchecked("".to_string()));

   let mut new_project: ProjectState = ProjectState {
      project_company: _project_company,
      project_title: _project_title,
      project_description: _project_description,
      project_ecosystem: _project_ecosystem,
      project_fundtype: _project_fundtype,
      project_createddate: _project_createddate,
      project_saft: _project_saft,
      project_logo: _project_logo,
      project_whitepaper: _project_whitepaper,
      project_website: _project_website,
      project_email: _project_email,
      //-----------------------------------
      project_id: Uint64::zero(), //auto increment
      project_status: ProjectStatus::WefundVote,
      fundraising_stage: Uint128::zero(),

      backerbacked_amount: Uint128::zero(),

      backer_states: Vec::new(),

      project_milestonestep: Uint128::zero(), //first milestonestep

      whitelist: Vec::new(),
      holder_alloc: Uint128::from(80u128),
      holder_ticket: Uint128::zero(),
      community_ticket: Uint128::zero(),
      //-------------------------------------------
      creator_wallet: deps.api.addr_validate(&_creator_wallet).unwrap(),
      project_collected: _project_collected,

      milestone_states: _project_milestones,
      teammember_states: _project_teammembers,
      vesting: _vesting.clone(),
      token_addr: token_addr.clone(),
      //---------------------------------------------------
      country: _country,
      cofounder_name: _cofounder_name,
      service_wefund: _service_wefund,
      service_charity: _service_charity,
      professional_link: _professional_link,
   };

   if _project_id == Uint64::zero() {
      save_projectstate(deps.storage, &mut new_project)?;
   } else {
      let x = PROJECTSTATES.load(deps.storage, _project_id.u64())?;
      new_project.project_id = x.project_id;
      new_project.project_status = x.project_status;
      new_project.fundraising_stage = x.fundraising_stage;
      new_project.backerbacked_amount = x.backerbacked_amount;
      new_project.backer_states = x.backer_states;
      new_project.project_milestonestep = x.project_milestonestep;
      new_project.whitelist = x.whitelist;
      new_project.holder_alloc = x.holder_alloc;
      new_project.holder_ticket = x.holder_ticket;
      new_project.community_ticket = x.community_ticket;
      PROJECTSTATES.save(deps.storage, _project_id.u64(), &new_project)?;
   }

   let config = CONFIG.load(deps.storage)?;
   if config.vesting_contract != "".to_string() {
      let mut vesting_params: Vec<VestingParam> = Vec::new();
      for param in _vesting {
         vesting_params.push(VestingParam {
            soon: param.stage_soon,
            after: param.stage_after,
            period: param.stage_period,
         })
      }
      //----------add fundraising project------------------------
      let add_vesting_project = WasmMsg::Execute {
         contract_addr: config.vesting_contract.to_string(),
         msg: to_binary(&VestingMsg::AddProject {
            project_id: new_project.project_id,
            admin: _env.contract.address.to_string(),
            token_addr: token_addr.to_string(),
            vesting_params: vesting_params,
            start_time: Uint128::zero(),
         })
         .unwrap(),
         funds: vec![],
      };

      return Ok(Response::new()
         .add_messages(vec![CosmosMsg::Wasm(add_vesting_project)])
         .add_attribute("action", "add project")
         .add_attribute("id", new_project.project_id));
   }

   Ok(Response::new().add_attribute("action", "add project"))
}

pub fn try_back2projectwithout(
   deps: DepsMut,
   env: Env,
   info: MessageInfo,
   project_id: Uint64,
   backer_wallet: String,
   denom: String,
   amount: Uint128,
   fundraising_stage: Uint128,
   token_amount: Uint128,
   otherchain: String,
   otherchain_wallet: String,
) -> Result<Response, ContractError> {
   //-------check project exist-----------------------------------
   let res = PROJECTSTATES.may_load(deps.storage, project_id.u64());
   if res == Ok(None) {
      //not exist
      return Err(ContractError::NotRegisteredProject {});
   }
   //--------Get project info------------------------------------
   let mut x = PROJECTSTATES.load(deps.storage, project_id.u64())?;
   let config = CONFIG.load(deps.storage)?;
   let fund = Coin {
      denom: denom,
      amount: amount
   };
   let mut fund_real_back = fund.clone();
   let mut fund_wefund = fund.clone();

   //--------calc amount to desposit and to wefund
   fund_real_back.amount = Uint128::new(fund.amount.u128() * 95 / 100);
   fund_wefund.amount = Uint128::new(fund.amount.u128() * 5 / 100);

   let backer_wallet = deps.api.addr_validate(&backer_wallet)?;

   //-----sum in whitelist-------------------
   let index = x
      .whitelist
      .iter()
      .position(|x| x.wallet == backer_wallet.clone());
   if index == None {
      return Err(ContractError::NotRegisteredWhitelist {});
   }

   x.whitelist[index.unwrap()].backed += fund_real_back.amount;
   x.backerbacked_amount += fund_real_back.amount;

   let new_baker: BackerState = BackerState {
      backer_wallet: backer_wallet.clone(),
      otherchain: otherchain,
      otherchain_wallet: otherchain_wallet,
      amount: fund_real_back.clone(),
   };

   x.backer_states.push(new_baker);

   PROJECTSTATES.update(deps.storage, project_id.u64(), |op| match op {
      None => Err(ContractError::NotRegisteredProject {}),
      Some(mut project) => {
         project.project_status = x.project_status.clone();
         project.backerbacked_amount = x.backerbacked_amount;
         project.backer_states = x.backer_states;
         project.whitelist = x.whitelist;

         if x.project_status == ProjectStatus::Releasing {
            //only on switching releasing status
            project.milestone_states = x.milestone_states;
         }
         Ok(project)
      }
   })?;

   Ok(Response::new()
      .add_attribute("action", "back to project without"))
}

pub fn try_back2project(
   deps: DepsMut,
   env: Env,
   info: MessageInfo,
   project_id: Uint64,
   backer_wallet: String,
   fundraising_stage: Uint128,
   token_amount: Uint128,
   otherchain: String,
   otherchain_wallet: String,
) -> Result<Response, ContractError> {
   //-------check project exist-----------------------------------
   let res = PROJECTSTATES.may_load(deps.storage, project_id.u64());
   if res == Ok(None) {
      //not exist
      return Err(ContractError::NotRegisteredProject {});
   }
   //--------Get project info------------------------------------
   let mut x = PROJECTSTATES.load(deps.storage, project_id.u64())?;
   if x.project_status != ProjectStatus::Fundraising {
      //only fundraising status
      return Err(ContractError::NotCorrectStatus {
         status: x.project_status as u32,
      });
   }

   let config = CONFIG.load(deps.storage)?;
   //--------check sufficient back--------------------
   if info.funds.is_empty() {
      return Err(ContractError::NeedCoin {});
   }
   let fund = info.funds[0].clone();
   let mut fund_real_back = fund.clone();
   let mut fund_wefund = fund.clone();

   //--------calc amount to desposit and to wefund
   fund_real_back.amount = Uint128::new(fund.amount.u128() * 95 / 100);
   fund_wefund.amount = Uint128::new(fund.amount.u128() * 5 / 100);

   let backer_wallet = deps.api.addr_validate(&backer_wallet)?;

   //--------check backed amount----------------
   let collected = x.project_collected * Uint128::from((10u128).pow(config.decimals));

   // if x.backerbacked_amount >= collected{
   //     return Err(ContractError::AlreadyCollected{});
   // }
   //-----sum in whitelist-------------------
   let index = x
      .whitelist
      .iter()
      .position(|x| x.wallet == backer_wallet.clone());
   if index == None {
      return Err(ContractError::NotRegisteredWhitelist {});
   }

   x.whitelist[index.unwrap()].backed += fund_real_back.amount;
   x.backerbacked_amount += fund_real_back.amount;

   let new_baker: BackerState = BackerState {
      backer_wallet: backer_wallet.clone(),
      otherchain: otherchain,
      otherchain_wallet: otherchain_wallet,
      amount: fund_real_back.clone(),
   };

   x.backer_states.push(new_baker);

   //------check needback-----------------
   let mut backer_needback = true;

   if x.backerbacked_amount >= collected {
      backer_needback = false;
   }

   let mut msgs: Vec<CosmosMsg> = vec![];

   //---------check collection and switch to releasing status---------
   if backer_needback == false {
      x.project_status = ProjectStatus::Releasing; //releasing

      //------add milestone votes in every milestone---------------
      let community = COMMUNITY.load(deps.storage)?;
      let mut milestone_votes = Vec::new();
      for backer in x.backer_states.clone() {
         let index = community.iter().position(|x| x == &backer.backer_wallet);
         if index == None {
            milestone_votes.push(Vote {
               wallet: backer.backer_wallet,
               voted: false,
            });
         }
      }
      //-----add wefund vote------------------
      let config = CONFIG.load(deps.storage)?;
      milestone_votes.push(Vote {
         wallet: config.owner,
         voted: true,
      });

      for i in 0..(x.milestone_states.len() as usize) {
         x.milestone_states[i].milestone_votes = milestone_votes.clone();
      }

      if config.vesting_contract != "".to_string() && x.token_addr != "".to_string() {
         let vesting = x.vesting.clone();
         let mut token_amount = Uint128::zero();
         for stage in vesting {
            token_amount += stage.stage_amount;
         }

         let token_info: TokenInfoResponse = deps
            .querier
            .query_wasm_smart(x.token_addr.clone(), &Cw20QueryMsg::TokenInfo {})?;

         token_amount = token_amount * Uint128::new((10 as u128).pow(token_info.decimals as u32)); //for decimals
         let token_transfer = WasmMsg::Execute {
            contract_addr: x.token_addr.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
               owner: x.creator_wallet.to_string(),
               recipient: config.vesting_contract.to_string(),
               amount: token_amount,
            })
            .unwrap(),
            funds: vec![],
         };
         msgs.push(CosmosMsg::Wasm(token_transfer));
         //---------start vesting-----------------------------
         let start_vesting = WasmMsg::Execute {
            contract_addr: config.vesting_contract.to_string(),
            msg: to_binary(&VestingMsg::StartRelease {
               project_id: x.project_id,
               start_time: Uint128::from(env.block.time.seconds()),
            })
            .unwrap(),
            funds: vec![],
         };
         msgs.push(CosmosMsg::Wasm(start_vesting));
      }
   }

   PROJECTSTATES.update(deps.storage, project_id.u64(), |op| match op {
      None => Err(ContractError::NotRegisteredProject {}),
      Some(mut project) => {
         project.project_status = x.project_status.clone();
         project.backerbacked_amount = x.backerbacked_amount;
         project.backer_states = x.backer_states;
         project.whitelist = x.whitelist;

         if x.project_status == ProjectStatus::Releasing {
            //only on switching releasing status
            project.milestone_states = x.milestone_states;
         }
         Ok(project)
      }
   })?;

   //---------send to Wefund with 5/100--------------------
   let bank_wefund = BankMsg::Send {
      to_address: config.wefund.to_string(),
      amount: vec![fund_wefund],
   };
   msgs.push(CosmosMsg::Bank(bank_wefund));

   if config.vesting_contract != "".to_string() {
      //----------add fundraising user------------------------
      let add_fundraising_user = WasmMsg::Execute {
         contract_addr: config.vesting_contract.to_string(),
         msg: to_binary(&VestingMsg::AddUser {
            project_id: project_id,
            wallet: info.sender,
            stage: fundraising_stage,
            amount: token_amount,
         })
         .unwrap(),
         funds: vec![],
      };
      msgs.push(CosmosMsg::Wasm(add_fundraising_user));
   }

   Ok(Response::new()
      .add_messages(msgs)
      .add_attribute("action", "back to project"))
}

pub fn try_openwhitelist(
   deps: DepsMut,
   env: Env,
   info: MessageInfo,
   project_id: Uint64,
   holder_alloc: Uint128,
) -> Result<Response, ContractError> {
   let mut x = PROJECTSTATES.load(deps.storage, project_id.u64())?;
   if info.sender != x.creator_wallet {
      return Err(ContractError::Unauthorized {});
   }
   x.project_status = ProjectStatus::Whitelist;
   x.whitelist = Vec::new();
   x.holder_alloc = holder_alloc;
   PROJECTSTATES.save(deps.storage, project_id.u64(), &x)?;
   Ok(Response::new())
}

pub fn try_registerwhitelist(
   deps: DepsMut,
   env: Env,
   info: MessageInfo,
   project_id: Uint64,
   card_type: CardType,
) -> Result<Response, ContractError> {
   let mut x = PROJECTSTATES.load(deps.storage, project_id.u64())?;
   let res = x.whitelist.iter().find(|x| x.wallet == info.sender);
   if res == None {
      x.whitelist.push(WhitelistState {
         wallet: info.sender,
         card_type: card_type,
         allocation: Uint128::zero(),
         backed: Uint128::zero(),
      });
      PROJECTSTATES.save(deps.storage, project_id.u64(), &x)?;
   }
   Ok(Response::new())
}

pub fn try_closewhitelist(
   deps: DepsMut,
   env: Env,
   info: MessageInfo,
   project_id: Uint64,
) -> Result<Response, ContractError> {
   let mut x = PROJECTSTATES.load(deps.storage, project_id.u64())?;
   if info.sender != x.creator_wallet {
      return Err(ContractError::Unauthorized {});
   }
   if x.project_status != ProjectStatus::Whitelist {
      return Err(ContractError::NotCorrectStatus {
         status: x.project_status as u32,
      });
   }

   let backamount;
   if x.backerbacked_amount >= x.project_collected {
      backamount = Uint128::zero()
   } else {
      backamount = x.project_collected * Uint128::from(UST) - x.backerbacked_amount;
   }
   let mut platium_count = 0;
   let mut gold_count = 0;
   let mut silver_count = 0;
   let mut bronze_count = 0;

   for one in x.whitelist.clone() {
      match one.card_type {
         CardType::Platium => {
            platium_count += 1;
         }
         CardType::Gold => {
            gold_count += 1;
         }
         CardType::Silver => {
            silver_count += 1;
         }
         CardType::Bronze => {
            bronze_count += 1;
         }
         other => {}
      }
   }

   x.holder_ticket = backamount * x.holder_alloc
      / Uint128::from(100u128)
      / Uint128::from(
         (platium_count * 120 + gold_count * 50 + silver_count * 11 + bronze_count) as u128,
      );

   let community = COMMUNITY.load(deps.storage)?;
   x.community_ticket = backamount * (Uint128::from(100u128) - x.holder_alloc)
      / Uint128::from(100u128)
      / Uint128::from(community.len() as u128);

   for i in 0..x.whitelist.len() {
      let card_type = x.whitelist[i].card_type.clone();
      match card_type {
         CardType::Platium => {
            x.whitelist[i].allocation = x.holder_ticket * Uint128::from(120u128);
         }
         CardType::Gold => {
            x.whitelist[i].allocation = x.holder_ticket * Uint128::from(50u128);
         }
         CardType::Silver => {
            x.whitelist[i].allocation = x.holder_ticket * Uint128::from(11u128);
         }
         CardType::Bronze => {
            x.whitelist[i].allocation = x.holder_ticket;
         }
         other => {}
      }
   }
   for one in community {
      x.whitelist.push(WhitelistState {
         wallet: one,
         card_type: CardType::Other,
         allocation: x.community_ticket.clone(),
         backed: Uint128::zero(),
      })
   }
   x.project_status = ProjectStatus::Fundraising;

   PROJECTSTATES.save(deps.storage, project_id.u64(), &x)?;
   Ok(Response::new())
}
