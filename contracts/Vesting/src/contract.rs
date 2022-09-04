#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
   to_binary, Addr, AllBalanceResponse, BankMsg, BankQuery, Coin, CosmosMsg, DepsMut, Env,
   MessageInfo, QueryRequest, Response, Storage, Uint128, Uint64, WasmMsg,
};
use cw2::set_contract_version;
use cw20::{
   BalanceResponse as Cw20BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg, TokenInfoResponse,
};

use crate::error::ContractError;
use crate::state::{CONFIG, JUNO_PRICE, TOTAL, USDC_PRICE, USERS, VEST_PARAM};
use Interface::vesting::{Config, ExecuteMsg, InstantiateMsg, UserInfo, VestingParameter};

// version info for migration info
const CONTRACT_NAME: &str = "AquaVesting";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const JUNO_DENOM: &str = "ujunox";
const USDC_DENOM: &str = "ibc/EAC38D55372F38F1AFD68DF7FE9EF762DCF69F26520643CF3F9D292A738D8034";

const AQUA_PRICE: u128 = 30; //1000

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

   let token_addr = deps.api.addr_validate(msg.token_addr.as_str())?;
   let treasury = deps.api.addr_validate(msg.treasury.as_str())?;

   CONFIG.save(
      deps.storage,
      &Config {
         owner,
         treasury: treasury.to_string(),
         token_addr: token_addr.to_string(),
         start_time: Uint128::zero(),
      },
   )?;

   VEST_PARAM.save(
      deps.storage,
      &VestingParameter {
         soon: Uint128::zero(),
         after: Uint128::zero(),
         period: Uint128::new(7_776_000), //0%: tge, 0: after, 3 months: priod
      },
   )?;

   TOTAL.save(deps.storage, &Uint128::new(0))?;

   USDC_PRICE.save(deps.storage, &Uint128::new(1000))?;
   JUNO_PRICE.save(deps.storage, &Uint128::new(5280))?;
   Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
   deps: DepsMut,
   env: Env,
   info: MessageInfo,
   msg: ExecuteMsg,
) -> Result<Response, ContractError> {
   match msg {
      ExecuteMsg::StartRelease { start_time } => try_startrelease(deps, info, start_time),

      ExecuteMsg::SetPrice {
         usdc_price,
         juno_price,
      } => try_setprice(deps, info, usdc_price, juno_price),

      ExecuteMsg::SetConfig {
         admin,
         treasury,
         token_addr,
         start_time,
      } => try_setconfig(deps, info, admin, treasury, token_addr, start_time),

      ExecuteMsg::SetVestingParameters { params } => try_setvestingparameters(deps, info, params),

      ExecuteMsg::AddUser {} => try_adduser(deps, info),

      ExecuteMsg::AddUserByOwner { wallet, amount } => {
         try_adduser_byowner(deps, info, wallet, amount)
      }

      ExecuteMsg::ClaimPendingTokens {} => try_claimpendingtokens(deps, env, info),

      ExecuteMsg::Withdraw { wallet } => try_withdraw(deps, env, info, wallet),
   }
}

pub fn try_startrelease(
   deps: DepsMut,
   info: MessageInfo,
   start_time: Uint128,
) -> Result<Response, ContractError> {
   let mut config = CONFIG.load(deps.storage)?;
   if config.owner != info.sender && config.treasury != info.sender {
      return Err(ContractError::Unauthorized {});
   }

   config.start_time = start_time;
   CONFIG.save(deps.storage, &config)?;
   Ok(Response::new().add_attribute("action", "Start Release"))
}

pub fn try_setvestingparameters(
   deps: DepsMut,
   info: MessageInfo,
   params: VestingParameter,
) -> Result<Response, ContractError> {
   let config = CONFIG.load(deps.storage).unwrap();
   if config.owner != info.sender && config.treasury != info.sender {
      return Err(ContractError::Unauthorized {});
   }

   VEST_PARAM.save(deps.storage, &params)?;
   Ok(Response::new().add_attribute("action", "Set Vesting parameters"))
}

pub fn calc_pending(store: &dyn Storage, env: Env, user: &UserInfo) -> Uint128 {
   let config = CONFIG.load(store).unwrap();
   if config.start_time == Uint128::zero() {
      return Uint128::zero();
   }

   let vest_param = VEST_PARAM.load(store).unwrap();

   let past_time = Uint128::new(env.block.time.seconds() as u128) - config.start_time;

   let mut unlocked = Uint128::zero();
   if past_time > Uint128::zero() {
      unlocked = user.total_amount * vest_param.soon / Uint128::new(100);
   }
   let locked = user.total_amount - unlocked;
   if past_time > vest_param.after {
      unlocked += (past_time - vest_param.after) * locked / vest_param.period;
      if unlocked >= locked {
         unlocked = locked;
      }
   }

   return unlocked - user.released_amount;
}

pub fn try_claimpendingtokens(
   deps: DepsMut,
   env: Env,
   info: MessageInfo,
) -> Result<Response, ContractError> {
   let mut user_info = USERS.load(deps.storage, info.sender.clone())?;
   let mut pending_amount = calc_pending(deps.storage, env.clone(), &user_info);
   if pending_amount == Uint128::zero() {
      return Err(ContractError::NoPendingTokens {});
   }

   user_info.released_amount += pending_amount;
   USERS.save(deps.storage, info.sender.clone(), &user_info)?;

   let config = CONFIG.load(deps.storage)?;
   let token_balance: Cw20BalanceResponse = deps.querier.query_wasm_smart(
      config.token_addr.clone(),
      &Cw20QueryMsg::Balance {
         address: config.treasury.clone(),
      },
   )?;
   if token_balance.balance < pending_amount {
      return Err(ContractError::NotEnoughBalance {});
   }

   let bank_cw20 = WasmMsg::Execute {
      contract_addr: config.token_addr,
      msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
         owner: config.treasury,
         recipient: info.sender.to_string(),
         amount: pending_amount,
      })
      .unwrap(),
      funds: Vec::new(),
   };

   Ok(Response::new()
      .add_message(CosmosMsg::Wasm(bank_cw20))
      .add_attribute("action", "Claim pending tokens"))
}

fn get_aqua_amount(storage: &dyn Storage, fund: &Coin) -> (bool, Uint128) {
   if fund.denom == USDC_DENOM {
      let usdc_price = USDC_PRICE.load(storage).unwrap();
      let amount = fund.amount.u128() * usdc_price.u128() * 1_000 / AQUA_PRICE / 1_000;
      return (true, Uint128::new(amount));
   } else if fund.denom == JUNO_DENOM {
      let juno_price = JUNO_PRICE.load(storage).unwrap();
      let amount = fund.amount.u128() * juno_price.u128() * 1_000 / AQUA_PRICE / 1_000;
      return (true, Uint128::new(amount));
   }
   (false, Uint128::zero())
}
pub fn try_adduser(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
   if info.funds.len() == 0 {
      return Err(ContractError::NeedFunds {});
   }

   let (is_support, amount) = get_aqua_amount(deps.storage, &info.funds[0]);
   if !is_support {
      return Err(ContractError::NotSupportToken {});
   }

   let mut user_info = USERS
      .may_load(deps.storage, info.sender.clone())?
      .unwrap_or(UserInfo {
         total_amount: Uint128::zero(),
         released_amount: Uint128::zero(),
      });
   user_info.total_amount += amount;

   USERS.save(deps.storage, info.sender, &user_info)?;
   let mut total = TOTAL.load(deps.storage)?;
   total += amount;
   TOTAL.save(deps.storage, &total)?;

   Ok(Response::new().add_attribute("action", "Add  User info"))
}
pub fn try_adduser_byowner(
   deps: DepsMut,
   info: MessageInfo,
   wallet: String,
   amount: Uint128,
) -> Result<Response, ContractError> {
   //-----------check owner--------------------------
   let config = CONFIG.load(deps.storage).unwrap();
   if config.owner != info.sender && config.treasury != info.sender {
      return Err(ContractError::Unauthorized {});
   }

   let address = deps.api.addr_validate(wallet.as_str()).unwrap();
   let mut user_info = USERS
      .may_load(deps.storage, address.clone())?
      .unwrap_or(UserInfo {
         total_amount: Uint128::zero(),
         released_amount: Uint128::zero(),
      });
   user_info.total_amount += amount;

   USERS.save(deps.storage, address, &user_info)?;
   let mut total = TOTAL.load(deps.storage)?;
   total += amount;
   TOTAL.save(deps.storage, &total)?;

   Ok(Response::new().add_attribute("action", "Add  User info"))
}
pub fn try_setconfig(
   deps: DepsMut,
   info: MessageInfo,
   admin: String,
   treasury: String,
   token_addr: String,
   start_time: Uint128,
) -> Result<Response, ContractError> {
   //-----------check owner--------------------------
   let mut config = CONFIG.load(deps.storage).unwrap();
   if info.sender != config.owner {
      return Err(ContractError::Unauthorized {});
   }

   config.owner = deps.api.addr_validate(admin.as_str())?;
   config.treasury = treasury;
   config.token_addr = token_addr;
   config.start_time = start_time;

   CONFIG.save(deps.storage, &config)?;
   Ok(Response::new().add_attribute("action", "SetConfig"))
}

pub fn try_setprice(
   deps: DepsMut,
   info: MessageInfo,
   usdc_price: Uint128,
   juno_price: Uint128,
) -> Result<Response, ContractError> {
   let config = CONFIG.load(deps.storage)?;
   if config.owner != info.sender && config.treasury != info.sender {
      return Err(ContractError::Unauthorized {});
   }

   USDC_PRICE.save(deps.storage, &usdc_price)?;
   JUNO_PRICE.save(deps.storage, &juno_price)?;
   Ok(Response::new().add_attribute("action", "SetPrice"))
}

pub fn try_withdraw(
   deps: DepsMut,
   env: Env,
   info: MessageInfo,
   wallet: String,
) -> Result<Response, ContractError> {
   //-----------check owner--------------------------
   let config = CONFIG.load(deps.storage).unwrap();
   if config.owner != info.sender && config.treasury != info.sender {
      return Err(ContractError::Unauthorized {});
   }

   //--------get all native coins ----------------------
   let balance: AllBalanceResponse =
      deps
         .querier
         .query(&QueryRequest::Bank(BankQuery::AllBalances {
            address: env.contract.address.to_string(),
         }))?;

   let bank_native = BankMsg::Send {
      to_address: wallet.clone(),
      amount: balance.amount,
   };

   Ok(Response::new()
      .add_message(CosmosMsg::Bank(bank_native))
      .add_attribute("action", "transfer all coins"))
}
