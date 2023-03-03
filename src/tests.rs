use crate::{
    contract::{execute, instantiate},
    error::*,
    msg::*,
};
use cosmwasm_std::{
    testing::{mock_dependencies, mock_env, mock_info},
    Binary, Coin, CosmosMsg, Deps, DepsMut, SubMsg, Uint128, WasmMsg,
};
use cw20::{Cw20Coin, Cw20ReceiveMsg, TokenInfoResponse};
use cw20_base::contract::{query_balance, query_token_info};

fn get_balance<T: Into<String>>(deps: Deps, address: T) -> Uint128 {
    query_balance(deps, address.into()).unwrap().balance
}

// this will set up the instantiation for other tests
fn do_instantiate(
    deps: DepsMut,
    addr1: String,
    amount1: Uint128,
    addr2: String,
    amount2: Uint128,
    frozen_amount: Uint128,
) {
    let instantiate_msg = Instantiate {
        name: "Bash Shell".to_string(),
        symbol: "BASH".to_string(),
        decimals: 6,
        initial_balances: vec![
            Cw20Coin {
                address: addr1.clone(),
                amount: amount1,
            },
            Cw20Coin {
                address: addr2,
                amount: amount2,
            },
        ],
        mint: None,
        frozen_balances: vec![Cw20Coin {
            address: addr1,
            amount: frozen_amount,
        }],
    };
    let info = mock_info("creator", &[]);
    let env = mock_env();
    let _ = instantiate(deps, env, info, instantiate_msg).unwrap();
}

#[test]
fn test_basic() {
    let mut deps = mock_dependencies(&[Coin {
        amount: Uint128::default(),
        denom: String::default(),
    }]);
    let amount = Uint128::from(11223344u128);
    let instantiate_msg = Instantiate {
        name: "Cash Token".to_string(),
        symbol: "CASH".to_string(),
        decimals: 9,
        initial_balances: vec![Cw20Coin {
            address: String::from("addr0000"),
            amount,
        }],
        mint: None,
        frozen_balances: vec![],
    };
    let info = mock_info("creator", &[]);
    let env = mock_env();
    let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    assert_eq!(
        query_token_info(deps.as_ref()).unwrap(),
        TokenInfoResponse {
            name: "Cash Token".to_string(),
            symbol: "CASH".to_string(),
            decimals: 9,
            total_supply: amount,
        }
    );
    assert_eq!(
        get_balance(deps.as_ref(), "addr0000"),
        Uint128::new(11223344)
    );
}

#[test]
fn test_transfer() {
    let mut deps = mock_dependencies(&[Coin {
        amount: Uint128::default(),
        denom: String::default(),
    }]);
    let amount1 = Uint128::from(11223344u128);
    let frozen_amount = Uint128::from(10000000u128);
    let addr1 = String::from("addr0001");
    let amount2 = Uint128::from(7890987u128);
    let trans_amount = Uint128::from(500u128);
    let addr2 = String::from("addr0002");
    let addr3 = String::from("addr0003");

    do_instantiate(
        deps.as_mut(),
        addr1.clone(),
        amount1,
        addr2.clone(),
        amount2,
        frozen_amount,
    );

    // Balance before transfer
    assert_eq!(get_balance(deps.as_ref(), addr1.clone()), amount1);
    assert_eq!(get_balance(deps.as_ref(), addr2.clone()), amount2);
    assert_eq!(get_balance(deps.as_ref(), addr3.clone()), Uint128::zero());

    // cannot transfer all amount as some part of it frozen
    let info = mock_info(addr1.as_ref(), &[]);
    let env = mock_env();
    let msg = Execute::Transfer {
        recipient: addr3.clone(),
        amount: amount1,
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::BalanceFrozen {});

    // valid transfer
    let info = mock_info(addr1.as_ref(), &[]);
    let env = mock_env();
    let msg = Execute::Transfer {
        recipient: addr3.clone(),
        amount: trans_amount,
    };
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);
    assert_eq!(
        get_balance(deps.as_ref(), addr1.clone()),
        amount1 - trans_amount
    );
    assert_eq!(get_balance(deps.as_ref(), addr3.clone()), trans_amount);

    // can transfer entire amount from addr2 as frozen list is empty for addr2
    let info = mock_info(addr2.as_ref(), &[]);
    let env = mock_env();
    let msg = Execute::Transfer {
        recipient: addr3.clone(),
        amount: amount2,
    };
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);
    assert_eq!(get_balance(deps.as_ref(), addr2.clone()), Uint128::zero());
    assert_eq!(
        get_balance(deps.as_ref(), addr3.clone()),
        trans_amount + amount2
    );
}

#[test]
fn test_send() {
    let mut deps = mock_dependencies(&[Coin {
        amount: Uint128::default(),
        denom: String::default(),
    }]);
    let amount1 = Uint128::from(11223344u128);
    let frozen_amount = Uint128::from(10000000u128);
    let addr1 = String::from("addr0001");
    let amount2 = Uint128::from(7890987u128);
    let addr2 = String::from("addr0002");
    let contract = String::from("addr0003");
    let send_msg = Binary::from(r#"{"some":123}"#.as_bytes());
    let trans_amount = Uint128::from(500u128);

    do_instantiate(
        deps.as_mut(),
        addr1.clone(),
        amount1,
        addr2.clone(),
        amount2,
        frozen_amount,
    );

    // Balance before send
    assert_eq!(get_balance(deps.as_ref(), addr1.clone()), amount1);
    assert_eq!(get_balance(deps.as_ref(), addr2.clone()), amount2);

    // cannot send all amount as some part of it frozen
    let info = mock_info(addr1.as_ref(), &[]);
    let env = mock_env();
    let msg = Execute::Send {
        amount: amount1,
        contract: contract.clone(),
        msg: send_msg.clone(),
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::BalanceFrozen {});

    // valid transfer
    let info = mock_info(addr2.as_ref(), &[]);
    let env = mock_env();
    let msg = Execute::Send {
        contract: contract.clone(),
        amount: trans_amount,
        msg: send_msg.clone(),
    };
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(res.messages.len(), 1);

    // ensure proper send message sent
    // this is the message we want delivered to the other side
    let binary_msg = Cw20ReceiveMsg {
        sender: addr2.clone(),
        amount: trans_amount,
        msg: send_msg,
    }
    .into_binary()
    .unwrap();
    // and this is how it must be wrapped for the vm to process it
    assert_eq!(
        res.messages[0],
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract.clone(),
            msg: binary_msg,
            funds: vec![],
        }))
    );

    // ensure balance is properly transferred
    let remainder = amount2.checked_sub(trans_amount).unwrap();
    assert_eq!(get_balance(deps.as_ref(), addr2), remainder);
    assert_eq!(get_balance(deps.as_ref(), contract), trans_amount);
}

#[test]
fn test_burn() {
    let mut deps = mock_dependencies(&[Coin {
        amount: Uint128::default(),
        denom: String::default(),
    }]);
    let amount1 = Uint128::from(11223344u128);
    let frozen_amount = Uint128::from(10000000u128);
    let addr1 = String::from("addr0001");
    let amount2 = Uint128::from(7890987u128);
    let addr2 = String::from("addr0002");

    do_instantiate(
        deps.as_mut(),
        addr1.clone(),
        amount1,
        addr2.clone(),
        amount2,
        frozen_amount,
    );

    // cannot burn token as some balance is frozen
    let info = mock_info(addr1.as_ref(), &[]);
    let env = mock_env();
    let msg = Execute::Burn { amount: amount1 };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::BalanceFrozen {});

    // valid burn
    let info = mock_info(addr2.as_ref(), &[]);
    let env = mock_env();
    let msg = Execute::Burn { amount: amount2 };
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);
}
