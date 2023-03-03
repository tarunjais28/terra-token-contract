use super::*;
use crate::{
    error::ContractError,
    msg::{Execute, Instantiate, Query, UpdateType},
};

#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Uint128,
};
use cw2::set_contract_version;
use cw20::BalanceResponse;
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
const CONTRACT_NAME: &str = "token_contract";
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

    // ensuring balance capital is not exceeded for an user
    if msg
        .initial_balances
        .iter()
        .any(|init_bal| init_bal.amount > msg.bal_cap)
    {
        return Err(ContractError::CannotExceedCap {});
    }

    // create initial accounts
    let total_supply = create_accounts(&mut deps, &msg)?;

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

fn create_accounts(deps: &mut DepsMut, msg: &Instantiate) -> StdResult<Uint128> {
    let mut total_supply = Uint128::zero();
    for account in &msg.initial_balances {
        let address = deps.api.addr_validate(&account.address)?;
        BALANCES.save(deps.storage, &address, &account.amount)?;
        total_supply += account.amount;
    }

    for account in &msg.frozen_balances {
        let address = deps.api.addr_validate(&account.address)?;
        FROZEN_BALANCES.save(deps.storage, &address, &account.amount)?;
    }

    BALANCE_CAP.save(deps.storage, &msg.bal_cap)?;

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
        Execute::Mint { recipient, amount } => mint(deps, env, info, recipient, amount),
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
        Execute::UpdateFrozenList(update_type) => Ok(update_frozen_list(update_type, deps)?),
    }
}

pub fn mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // ensuring balance capital is not exceeded for an user
    let rcpt_addr = deps.api.addr_validate(&recipient)?;
    let token_bal = BALANCES.load(deps.storage, &rcpt_addr)?;
    let bal_cap = BALANCE_CAP.load(deps.storage)?;
    if (token_bal + amount) > bal_cap {
        return Err(ContractError::CannotExceedCap {});
    }

    Ok(execute_mint(deps, env, info, recipient, amount)?)
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

    // ensuring balance capital is not exceeded for an user
    let rcpt_addr = deps.api.addr_validate(&recipient)?;
    let token_bal = BALANCES
        .load(deps.storage, &rcpt_addr)
        .unwrap_or(Uint128::default());
    let bal_cap = BALANCE_CAP.load(deps.storage)?;
    if (token_bal + amount) > bal_cap {
        return Err(ContractError::CannotExceedCap {});
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

    // ensuring balance capital is not exceeded for an user
    let rcpt_addr = deps.api.addr_validate(&recipient)?;
    let token_bal = BALANCES
        .load(deps.storage, &rcpt_addr)
        .unwrap_or(Uint128::default());
    let bal_cap = BALANCE_CAP.load(deps.storage)?;
    if (token_bal + amount) > bal_cap {
        return Err(ContractError::CannotExceedCap {});
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

fn update_frozen_list(update_type: UpdateType, deps: DepsMut) -> Result<Response, ContractError> {
    match update_type {
        UpdateType::Add(coin) => {
            let address = deps.api.addr_validate(&coin.address)?;
            FROZEN_BALANCES.update(
                deps.storage,
                &address,
                |balance: Option<Uint128>| -> StdResult<_> {
                    Ok(balance.unwrap_or_default().checked_add(coin.amount)?)
                },
            )?;
        }
        UpdateType::Sub(coin) => {
            let address = deps.api.addr_validate(&coin.address)?;
            FROZEN_BALANCES.update(
                deps.storage,
                &address,
                |balance: Option<Uint128>| -> StdResult<_> {
                    Ok(balance.unwrap_or_default().checked_sub(coin.amount)?)
                },
            )?;
        }
        UpdateType::Discard(addr) => {
            let address = deps.api.addr_validate(&addr)?;
            FROZEN_BALANCES.remove(deps.storage, &address)
        }
    };

    let res = Response::new().add_attribute("action", "update_frozen_list");
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: Query) -> StdResult<Binary> {
    match msg {
        // inherited from cw20-base
        Query::TokenInfo {} => to_binary(&query_token_info(deps)?),
        Query::Balance { address } => to_binary(&query_balance(deps, address)?),
        Query::FrozenBalance { address } => to_binary(&query_frozen_balance(deps, address)?),
        Query::Allowance { owner, spender } => to_binary(&query_allowance(deps, owner, spender)?),
        Query::Minter {} => to_binary(&query_minter(deps)?),
    }
}

pub fn query_frozen_balance(deps: Deps, address: String) -> StdResult<BalanceResponse> {
    let address = deps.api.addr_validate(&address)?;
    let balance = FROZEN_BALANCES
        .may_load(deps.storage, &address)?
        .unwrap_or_default();
    Ok(BalanceResponse { balance })
}
