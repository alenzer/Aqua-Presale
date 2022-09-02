#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, AllBalanceResponse, BankQuery, Binary, Coin, Deps, Env, QueryRequest,
    StdResult, Uint128, Uint64,
};

use cw20::{BalanceResponse as Cw20BalanceResponse, Cw20QueryMsg, TokenInfoResponse};

use crate::contract::calc_pending;
use crate::state::{CONFIG, TOTAL, USERS, VEST_PARAM};
use Interface::vesting::{Config, QueryMsg, UserInfo};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetBalance { wallet } => to_binary(&query_balance(deps, env, wallet)?),

        QueryMsg::GetConfig {} => to_binary(&query_getconfig(deps)?),

        QueryMsg::GetPendingTokens { wallet } => {
            to_binary(&query_pendingtokens(deps, env, wallet)?)
        }

        QueryMsg::GetUserInfo { wallet } => to_binary(&query_getuserinfo(deps, wallet)?),
    }
}
fn query_pendingtokens(deps: Deps, env: Env, wallet: Addr) -> StdResult<Uint128> {
    let user_info = USERS.load(deps.storage, wallet)?;

    let pending_amount = calc_pending(
        deps.storage,
        env.clone(),
        &user_info,
    );

    Ok(pending_amount)
}

fn query_balance(
    deps: Deps,
    _env: Env,
    wallet: Addr,
) -> StdResult<AllBalanceResponse> {
    // let uusd_denom = String::from("uusd");
    let mut balance: AllBalanceResponse =
        deps.querier
            .query(&QueryRequest::Bank(BankQuery::AllBalances {
                address: wallet.to_string(),
            }))?;

    let config = CONFIG.load(deps.storage)?;

    let token_balance: Cw20BalanceResponse = deps.querier.query_wasm_smart(
        config.token_addr.clone(),
        &Cw20QueryMsg::Balance { address: wallet.to_string() },
    )?;
    let token_info: TokenInfoResponse = deps
        .querier
        .query_wasm_smart(config.token_addr, &Cw20QueryMsg::TokenInfo {})?;
    balance
        .amount
        .push(Coin::new(token_balance.balance.u128(), token_info.name));

    Ok(balance)
}

fn query_getconfig(deps: Deps) -> StdResult<Config> {
    let x = CONFIG.load(deps.storage)?;
    Ok(x)
}

fn query_getuserinfo(deps: Deps, wallet: Addr) -> StdResult<UserInfo> {
    let user = USERS.load(deps.storage, wallet)?;
    Ok(user)
}
