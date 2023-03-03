use super::*;
use crate::{
    error::ContractError,
    msg::{Execute, Instantiate, Query},
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Uint128,
};
use cw2::set_contract_version;
use cw20::Cw20Coin;
use cw20_base::{
    allowances::{
        execute_burn_from, execute_decrease_allowance, execute_increase_allowance,
        execute_send_from, execute_transfer_from, query_allowance,
    },
    contract::{
        execute_burn, execute_mint, execute_send, execute_transfer, execute_update_marketing,
        execute_upload_logo, query_balance, query_minter, query_token_info,
    },
    state::*,
};

// version info for migration info
const CONTRACT_NAME: &str = "tokenContract";
const CONTRACT_VERSION: &str = "1.0.0";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: Instantiate,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // check valid token info
    msg.validate()?;
    // create initial accounts
    let total_supply = create_accounts(&mut deps, &msg.initial_balances, &msg.frozen_balances)?;

    if let Some(limit) = msg.get_cap() {
        if total_supply > limit {
            return Err(StdError::generic_err("Initial supply greater than cap").into());
        }
    }

    let mint = match msg.mint {
        Some(m) => Some(MinterData {
            minter: deps.api.addr_validate(&m.minter)?,
            cap: m.cap,
        }),
        None => None,
    };

    // store token info
    let data = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        total_supply,
        mint,
    };
    TOKEN_INFO.save(deps.storage, &data)?;

    Ok(Response::new().add_attribute("action", "intantiated"))
}

fn create_accounts(
    deps: &mut DepsMut,
    accounts: &[Cw20Coin],
    frozen_accounts: &[Cw20Coin],
) -> StdResult<Uint128> {
    let mut total_supply = Uint128::zero();
    for account in accounts {
        let address = deps.api.addr_validate(&account.address)?;
        BALANCES.save(deps.storage, &address, &account.amount)?;
        total_supply += account.amount;
    }

    for account in frozen_accounts {
        let address = deps.api.addr_validate(&account.address)?;
        FROZEN_BALANCES.save(deps.storage, &address, &account.amount)?;
    }
    Ok(total_supply)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Execute,
) -> Result<Response, ContractError> {
    match msg {
        Execute::Mint { recipient, amount } => {
            Ok(execute_mint(deps, env, info, recipient, amount)?)
        }
        Execute::Transfer { recipient, amount } => transfer(deps, env, info, recipient, amount),
        Execute::Send {
            contract,
            amount,
            msg,
        } => send(deps, env, info, contract, amount, msg),
        Execute::Burn { amount } => burn(deps, env, info, amount),
        Execute::IncreaseAllowance {
            spender,
            amount,
            expires,
        } => Ok(execute_increase_allowance(
            deps, env, info, spender, amount, expires,
        )?),
        Execute::DecreaseAllowance {
            spender,
            amount,
            expires,
        } => Ok(execute_decrease_allowance(
            deps, env, info, spender, amount, expires,
        )?),
        Execute::TransferFrom {
            owner,
            recipient,
            amount,
        } => transfer_from(deps, env, info, owner, recipient, amount),
        Execute::BurnFrom { owner, amount } => burn_from(deps, env, info, owner, amount),
        Execute::SendFrom {
            owner,
            contract,
            amount,
            msg,
        } => send_from(deps, env, info, owner, contract, amount, msg),
        Execute::UpdateMarketing {
            project,
            description,
            marketing,
        } => Ok(execute_update_marketing(
            deps,
            env,
            info,
            project,
            description,
            marketing,
        )?),
        Execute::UploadLogo(logo) => Ok(execute_upload_logo(deps, env, info, logo)?),
    }
}

fn transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // Ensuring balance is unlocked for transaction
    let balance = BALANCES.load(deps.storage, &info.sender)?;
    let frozen_balance = FROZEN_BALANCES
        .load(deps.storage, &info.sender)
        .unwrap_or(Uint128::zero());
    if (balance - frozen_balance) < amount {
        return Err(ContractError::BalanceFrozen {});
    }

    Ok(execute_transfer(deps, env, info, recipient, amount)?)
}

fn send(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract: String,
    amount: Uint128,
    msg: Binary,
) -> Result<Response, ContractError> {
    // Ensuring balance is unlocked for transaction
    let balance = BALANCES.load(deps.storage, &info.sender)?;
    let frozen_balance = FROZEN_BALANCES
        .load(deps.storage, &info.sender)
        .unwrap_or(Uint128::zero());
    if (balance - frozen_balance) < amount {
        return Err(ContractError::BalanceFrozen {});
    }

    Ok(execute_send(deps, env, info, contract, amount, msg)?)
}

fn burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // Ensuring balance is unlocked for transaction
    let balance = BALANCES.load(deps.storage, &info.sender)?;
    let frozen_balance = FROZEN_BALANCES
        .load(deps.storage, &info.sender)
        .unwrap_or(Uint128::zero());
    if (balance - frozen_balance) < amount {
        return Err(ContractError::BalanceFrozen {});
    }

    Ok(execute_burn(deps, env, info, amount)?)
}

fn transfer_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // Ensuring balance is unlocked for transaction
    let balance = BALANCES.load(deps.storage, &info.sender)?;
    let frozen_balance = FROZEN_BALANCES
        .load(deps.storage, &info.sender)
        .unwrap_or(Uint128::zero());
    if (balance - frozen_balance) < amount {
        return Err(ContractError::BalanceFrozen {});
    }

    Ok(execute_transfer_from(
        deps, env, info, owner, recipient, amount,
    )?)
}

fn burn_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // Ensuring balance is unlocked for transaction
    let balance = BALANCES.load(deps.storage, &info.sender)?;
    let frozen_balance = FROZEN_BALANCES
        .load(deps.storage, &info.sender)
        .unwrap_or(Uint128::zero());
    if (balance - frozen_balance) < amount {
        return Err(ContractError::BalanceFrozen {});
    }

    Ok(execute_burn_from(deps, env, info, owner, amount)?)
}

pub fn send_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    contract: String,
    amount: Uint128,
    msg: Binary,
) -> Result<Response, ContractError> {
    // Ensuring balance is unlocked for transaction
    let balance = BALANCES.load(deps.storage, &info.sender)?;
    let frozen_balance = FROZEN_BALANCES
        .load(deps.storage, &info.sender)
        .unwrap_or(Uint128::zero());
    if (balance - frozen_balance) < amount {
        return Err(ContractError::BalanceFrozen {});
    }

    Ok(execute_send_from(
        deps, env, info, owner, contract, amount, msg,
    )?)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: Query) -> StdResult<Binary> {
    match msg {
        // inherited from cw20-base
        Query::TokenInfo {} => to_binary(&query_token_info(deps)?),
        Query::Balance { address } => to_binary(&query_balance(deps, address)?),
        Query::Allowance { owner, spender } => to_binary(&query_allowance(deps, owner, spender)?),
        Query::Minter {} => to_binary(&query_minter(deps)?),
    }
}
