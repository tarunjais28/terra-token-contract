use super::*;
use crate::{
    data::{ProposalData, WithdrawData},
    error::ContractError,
    msg::{Execute, Instantiate, Query},
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Uint128, Uint64,
};
use cw2::set_contract_version;
use cw20::Cw20Coin;
use cw20_base::{
    allowances::query_allowance,
    contract::{
        execute_burn, execute_mint, execute_transfer, query_balance, query_minter, query_token_info,
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
    let total_supply = create_accounts(&mut deps, &msg.initial_balances)?;

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

fn create_accounts(deps: &mut DepsMut, accounts: &[Cw20Coin]) -> StdResult<Uint128> {
    let mut total_supply = Uint128::zero();
    for row in accounts {
        let address = deps.api.addr_validate(&row.address)?;
        BALANCES.save(deps.storage, &address, &row.amount)?;
        total_supply += row.amount;
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
        Execute::Deposit { resource_id, data } => {
            execute_deposit(deps, env, info, resource_id, &data)
        }
        Execute::Mint { recipient, amount } => mint(deps, env, info, recipient, amount),
        Execute::WhiteList { address } => Ok(add_to_list(deps, address, ListType::WhiteList)?),
        Execute::BurnList { address } => Ok(add_to_list(deps, address, ListType::BurnList)?),
        Execute::SetResourceId {
            resource_id,
            address,
        } => Ok(set_resource_id(deps, resource_id, address)?),
        Execute::Proposal { resource_id, data } => {
            execute_proposal(deps, env, info, resource_id, &data)
        }
        Execute::Withdraw { data } => execute_withdraw(deps, env, info, &data),
    }
}

fn execute_deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    resource_id: Uint64,
    data: &[u8],
) -> Result<Response, ContractError> {
    let amount = Uint128::new(u128::from_be_bytes(*array_ref!(data, 0, 16)));

    let token_addr =
        match RESOURCE_ID_TO_TOKEN_CONTRACT_ADDRESS.load(deps.storage, &resource_id.to_string()) {
            Ok(addr) => addr,
            Err(_) => {
                return Err(Err(
                    ContractError::MissingResourceIdToTokenAddressMapping {},
                )?)
            }
        };

    if !WHITELIST.has(deps.storage, &token_addr) {
        return Err(Err(ContractError::AddressNotFoundInWhiteList {})?);
    }

    if BURNLIST.has(deps.storage, &token_addr) {
        execute_burn(deps, env, info.clone(), amount)?;
    } else {
        execute_transfer(deps, env, info.clone(), token_addr.to_string(), amount)?;
    }

    let res = Response::new()
        .add_attribute("action", "deposit")
        .add_attribute("from", info.sender)
        .add_attribute("to", token_addr)
        .add_attribute("amount", amount);
    Ok(res)
}

fn execute_proposal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    resource_id: Uint64,
    mut data: &[u8],
) -> Result<Response, ContractError> {
    let proposal_data: ProposalData = match Decode::decode(&mut data) {
        Ok(data) => data,
        Err(_) => return Err(ContractError::InvalidProposalData {}),
    };

    let amount = Uint128::new(proposal_data.amount);
    let token_address =
        match RESOURCE_ID_TO_TOKEN_CONTRACT_ADDRESS.load(deps.storage, &resource_id.to_string()) {
            Ok(addr) => addr,
            Err(_) => {
                return Err(Err(
                    ContractError::MissingResourceIdToTokenAddressMapping {},
                )?)
            }
        };

    if !WHITELIST.has(deps.storage, &token_address) {
        return Err(Err(ContractError::AddressNotFoundInWhiteList {})?);
    }

    if BURNLIST.has(deps.storage, &token_address) {
        mint(
            deps,
            env,
            info,
            proposal_data.recipient_address.to_string(),
            amount,
        )?;
    } else {
        execute_transfer(deps, env, info.clone(), token_address.to_string(), amount)?;
    }

    let res = Response::new()
        .add_attribute("action", "proposal")
        .add_attribute("recipient_address", proposal_data.recipient_address)
        .add_attribute("amount", amount);
    Ok(res)
}

fn execute_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut data: &[u8],
) -> Result<Response, ContractError> {
    let withdraw_data: WithdrawData = match Decode::decode(&mut data) {
        Ok(data) => data,
        Err(_) => return Err(ContractError::InvalidWithdrawData {}),
    };

    let amount = Uint128::new(withdraw_data.amount);
    execute_transfer(
        deps,
        env,
        info.clone(),
        withdraw_data.token_address.to_string(),
        amount,
    )?;

    let res = Response::new()
        .add_attribute("action", "withdraw")
        .add_attribute("token_address", withdraw_data.token_address)
        .add_attribute("recipient_address", withdraw_data.recipient_address)
        .add_attribute("amount", amount);
    Ok(res)
}

fn mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    Ok(execute_mint(deps, env, info, recipient, amount)?)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: Query) -> StdResult<Binary> {
    match msg {
        // inherited from cw20-base
        Query::TokenInfo {} => to_binary(&query_token_info(deps)?),
        Query::Balance { address } => to_binary(&query_balance(deps, address)?),
        Query::Allowance { owner, spender } => {
            to_binary(&query_allowance(deps, owner, spender)?)
        }
        Query::Minter {} => to_binary(&query_minter(deps)?),
    }
}
