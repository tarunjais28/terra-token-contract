use super::*;
use crate::{
    add_to_list,
    contract::{execute, instantiate, query},
    data::{ProposalData, WithdrawData},
    error::*,
    msg::*,
};
use cosmwasm_std::{
    from_binary,
    testing::{mock_dependencies, mock_env, mock_info},
    Coin, Deps, DepsMut, Uint128, Uint64,
};
use cw20::{BalanceResponse, Cw20Coin, MinterResponse, TokenInfoResponse};
use cw20_base::contract::{query_balance, query_minter, query_token_info};

fn get_balance<T: Into<String>>(deps: Deps, address: T) -> Uint128 {
    query_balance(deps, address.into()).unwrap().balance
}

// this will set up the instantiation for other tests
fn do_instantiate_with_minter(
    deps: DepsMut,
    addr: &str,
    amount: Uint128,
    minter: &str,
    cap: Option<Uint128>,
) -> TokenInfoResponse {
    _do_instantiate(
        deps,
        addr,
        amount,
        Some(MinterResponse {
            minter: minter.to_string(),
            cap,
        }),
    )
}

// this will set up the instantiation for other tests
fn do_instantiate(deps: DepsMut, addr: &str, amount: Uint128) -> TokenInfoResponse {
    _do_instantiate(deps, addr, amount, None)
}

// this will set up the instantiation for other tests
fn _do_instantiate(
    mut deps: DepsMut,
    addr: &str,
    amount: Uint128,
    mint: Option<MinterResponse>,
) -> TokenInfoResponse {
    let instantiate_msg = Instantiate {
        name: "Auto Gen".to_string(),
        symbol: "AUTO".to_string(),
        decimals: 3,
        initial_balances: vec![Cw20Coin {
            address: addr.to_string(),
            amount,
        }],
        mint: mint.clone(),
    };
    let info = mock_info("creator", &[]);
    let env = mock_env();
    let res = instantiate(deps.branch(), env, info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    let meta = query_token_info(deps.as_ref()).unwrap();
    assert_eq!(
        meta,
        TokenInfoResponse {
            name: "Auto Gen".to_string(),
            symbol: "AUTO".to_string(),
            decimals: 3,
            total_supply: amount,
        }
    );
    assert_eq!(get_balance(deps.as_ref(), addr), amount);
    assert_eq!(query_minter(deps.as_ref()).unwrap(), mint,);
    meta
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
fn test_add_to_list_whitelist() {
    let mut deps = mock_dependencies(&[Coin {
        amount: Uint128::default(),
        denom: String::default(),
    }]);
    let address = String::from("adr001");
    do_instantiate(deps.as_mut(), &String::from("genesis"), Uint128::new(1234));

    let _ = add_to_list(deps.as_mut(), address.clone(), ListType::WhiteList).unwrap();
    assert!(WHITELIST.load(&deps.storage, &address).unwrap());
}

#[test]
fn test_add_to_list_burnlist() {
    let mut deps = mock_dependencies(&[Coin {
        amount: Uint128::default(),
        denom: String::default(),
    }]);
    let address = String::from("adr001");
    do_instantiate(deps.as_mut(), &String::from("genesis"), Uint128::new(1234));

    let _ = add_to_list(deps.as_mut(), address.clone(), ListType::BurnList).unwrap();
    assert!(BURNLIST.load(&deps.storage, &address).unwrap());
}

#[test]
fn test_add_to_list_whitelist_empty() {
    let mut deps = mock_dependencies(&[Coin {
        amount: Uint128::default(),
        denom: String::default(),
    }]);
    let address = String::from("adr001");
    do_instantiate(deps.as_mut(), &String::from("genesis"), Uint128::new(1234));

    assert!(WHITELIST.load(&deps.storage, &address).is_err());
}

#[test]
fn test_add_to_list_burnlist_empty() {
    let mut deps = mock_dependencies(&[Coin {
        amount: Uint128::default(),
        denom: String::default(),
    }]);
    let address = String::from("adr001");
    do_instantiate(deps.as_mut(), &String::from("genesis"), Uint128::new(1234));

    assert!(BURNLIST.load(&deps.storage, &address).is_err());
}

#[test]
fn test_set_resource_id() {
    let mut deps = mock_dependencies(&[Coin {
        amount: Uint128::default(),
        denom: String::default(),
    }]);
    let address = String::from("adr001");
    do_instantiate(deps.as_mut(), &String::from("genesis"), Uint128::new(1234));

    let resource_id = Uint64::new(1);
    let _ = set_resource_id(deps.as_mut(), resource_id, address.clone()).unwrap();

    assert_eq!(
        RESOURCE_ID_TO_TOKEN_CONTRACT_ADDRESS
            .load(&deps.storage, &resource_id.to_string())
            .unwrap(),
        address
    );

    assert_eq!(
        TOKEN_CONTRACT_ADDRESS_TO_RESOURCE_ID
            .load(&deps.storage, &address)
            .unwrap(),
        resource_id
    );
}

#[test]
fn test_set_resource_id_empty_maps() {
    let mut deps = mock_dependencies(&[Coin {
        amount: Uint128::default(),
        denom: String::default(),
    }]);
    let address = String::from("adr001");
    do_instantiate(deps.as_mut(), &String::from("genesis"), Uint128::new(1234));

    let resource_id = Uint64::new(1);

    assert!(RESOURCE_ID_TO_TOKEN_CONTRACT_ADDRESS
        .load(&deps.storage, &resource_id.to_string())
        .is_err());

    assert!(TOKEN_CONTRACT_ADDRESS_TO_RESOURCE_ID
        .load(&deps.storage, &address)
        .is_err());
}

mod instantiate {
    use cosmwasm_std::{Coin, StdError};

    use super::*;

    #[test]
    fn basic() {
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
    fn mintable() {
        let mut deps = mock_dependencies(&[Coin {
            amount: Uint128::default(),
            denom: String::default(),
        }]);
        let amount = Uint128::new(11223344);
        let minter = String::from("asmodat");
        let limit = Uint128::new(511223344);
        let instantiate_msg = Instantiate {
            name: "Cash Token".to_string(),
            symbol: "CASH".to_string(),
            decimals: 9,
            initial_balances: vec![Cw20Coin {
                address: "addr0000".into(),
                amount,
            }],
            mint: Some(MinterResponse {
                minter: minter.clone(),
                cap: Some(limit),
            }),
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
        assert_eq!(
            query_minter(deps.as_ref()).unwrap(),
            Some(MinterResponse {
                minter,
                cap: Some(limit),
            }),
        );
    }

    #[test]
    fn mintable_over_cap() {
        let mut deps = mock_dependencies(&[Coin {
            amount: Uint128::default(),
            denom: String::default(),
        }]);
        let amount = Uint128::new(11223344);
        let minter = String::from("asmodat");
        let limit = Uint128::new(11223300);
        let instantiate_msg = Instantiate {
            name: "Cash Token".to_string(),
            symbol: "CASH".to_string(),
            decimals: 9,
            initial_balances: vec![Cw20Coin {
                address: String::from("addr0000"),
                amount,
            }],
            mint: Some(MinterResponse {
                minter,
                cap: Some(limit),
            }),
        };
        let info = mock_info("creator", &[]);
        let env = mock_env();
        let err = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap_err();
        assert_eq!(
            err,
            StdError::generic_err("Initial supply greater than cap").into()
        );
    }
}

#[test]
fn can_mint_by_minter() {
    let mut deps = mock_dependencies(&[Coin {
        amount: Uint128::default(),
        denom: String::default(),
    }]);

    let genesis = String::from("genesis");
    let amount = Uint128::new(11223344);
    let minter = String::from("asmodat");
    let limit = Uint128::new(511223344);
    do_instantiate_with_minter(deps.as_mut(), &genesis, amount, &minter, Some(limit));

    // minter can mint coins to some winner
    let winner = String::from("lucky");
    let prize = Uint128::new(222_222_222);
    let msg = Execute::Mint {
        recipient: winner.clone(),
        amount: prize,
    };

    let info = mock_info(minter.as_ref(), &[]);
    let env = mock_env();
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(0, res.messages.len());
    assert_eq!(get_balance(deps.as_ref(), genesis), amount);
    assert_eq!(get_balance(deps.as_ref(), winner.clone()), prize);

    // but cannot mint nothing
    let msg = Execute::Mint {
        recipient: winner.clone(),
        amount: Uint128::zero(),
    };
    let info = mock_info(minter.as_ref(), &[]);
    let env = mock_env();
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::InvalidZeroAmount {});

    // but if it exceeds cap (even over multiple rounds), it fails
    // cap is enforced
    let msg = Execute::Mint {
        recipient: winner,
        amount: Uint128::new(333_222_222),
    };
    let info = mock_info(minter.as_ref(), &[]);
    let env = mock_env();
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::CannotExceedCap {});
}

#[test]
fn others_cannot_mint() {
    let mut deps = mock_dependencies(&[Coin {
        amount: Uint128::default(),
        denom: String::default(),
    }]);
    do_instantiate_with_minter(
        deps.as_mut(),
        &String::from("genesis"),
        Uint128::new(1234),
        &String::from("minter"),
        None,
    );

    let msg = Execute::Mint {
        recipient: String::from("lucky"),
        amount: Uint128::new(222),
    };
    let info = mock_info("anyone else", &[]);
    let env = mock_env();
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn no_one_mints_if_minter_unset() {
    let mut deps = mock_dependencies(&[Coin {
        amount: Uint128::default(),
        denom: String::default(),
    }]);
    do_instantiate(deps.as_mut(), &String::from("genesis"), Uint128::new(1234));

    let msg = Execute::Mint {
        recipient: String::from("lucky"),
        amount: Uint128::new(222),
    };
    let info = mock_info("genesis", &[]);
    let env = mock_env();
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn instantiate_multiple_accounts() {
    let mut deps = mock_dependencies(&[Coin {
        amount: Uint128::default(),
        denom: String::default(),
    }]);
    let amount1 = Uint128::from(11223344u128);
    let addr1 = String::from("addr0001");
    let amount2 = Uint128::from(7890987u128);
    let addr2 = String::from("addr0002");
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
                address: addr2.clone(),
                amount: amount2,
            },
        ],
        mint: None,
    };
    let info = mock_info("creator", &[]);
    let env = mock_env();
    let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    assert_eq!(
        query_token_info(deps.as_ref()).unwrap(),
        TokenInfoResponse {
            name: "Bash Shell".to_string(),
            symbol: "BASH".to_string(),
            decimals: 6,
            total_supply: amount1 + amount2,
        }
    );
    assert_eq!(get_balance(deps.as_ref(), addr1), amount1);
    assert_eq!(get_balance(deps.as_ref(), addr2), amount2);
}

#[test]
fn queries_work() {
    let mut deps = mock_dependencies(&[Coin {
        amount: Uint128::new(2),
        denom: String::from("token"),
    }]);
    let addr1 = String::from("addr0001");
    let amount1 = Uint128::from(12340000u128);

    let expected = do_instantiate(deps.as_mut(), &addr1, amount1);

    // check meta query
    let loaded = query_token_info(deps.as_ref()).unwrap();
    assert_eq!(expected, loaded);

    let _info = mock_info("test", &[]);
    let env = mock_env();
    // check balance query (full)
    let data = query(
        deps.as_ref(),
        env.clone(),
        Query::Balance { address: addr1 },
    )
    .unwrap();
    let loaded: BalanceResponse = from_binary(&data).unwrap();
    assert_eq!(loaded.balance, amount1);

    // check balance query (empty)
    let data = query(
        deps.as_ref(),
        env,
        Query::Balance {
            address: String::from("addr0002"),
        },
    )
    .unwrap();
    let loaded: BalanceResponse = from_binary(&data).unwrap();
    assert_eq!(loaded.balance, Uint128::zero());
}

#[test]
fn test_deposit_with_burn_list() {
    let mut deps = mock_dependencies(&[Coin {
        amount: Uint128::default(),
        denom: String::default(),
    }]);

    let genesis = String::from("genesis");
    let amount = Uint128::new(11223344);
    let deposit_amount: u128 = 1000;
    let data = deposit_amount.to_be_bytes().to_vec();
    let resource_id = Uint64::new(1);
    do_instantiate(deps.as_mut(), &genesis, amount);

    let info = mock_info(genesis.as_ref(), &[]);
    let env = mock_env();

    // Adding data to whitelist
    let msg = Execute::WhiteList {
        address: genesis.to_string(),
    };
    let _ = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Adding data to burnlist
    let msg = Execute::BurnList {
        address: genesis.to_string(),
    };
    let _ = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Adding data to burnlist
    let msg = Execute::SetResourceId {
        address: genesis.to_string(),
        resource_id,
    };
    let _ = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    let msg = Execute::Deposit { resource_id, data };
    let res = execute(deps.as_mut(), env, info, msg).unwrap();

    let updated_amount = amount.checked_sub(Uint128::new(deposit_amount)).unwrap();
    assert_eq!(0, res.messages.len());
    assert_eq!(get_balance(deps.as_ref(), genesis), updated_amount);
}

#[test]
fn test_deposit_without_burn_list() {
    let mut deps = mock_dependencies(&[Coin {
        amount: Uint128::default(),
        denom: String::default(),
    }]);

    let genesis = String::from("genesis");
    let addr = String::from("addr001");
    let amount = Uint128::new(11223344);
    let deposit_amount: u128 = 1000;
    let data = deposit_amount.to_be_bytes().to_vec();
    let resource_id = Uint64::new(1);
    do_instantiate(deps.as_mut(), &genesis, amount);

    let info = mock_info(genesis.as_ref(), &[]);
    let env = mock_env();

    // Adding data to whitelist
    let msg = Execute::WhiteList {
        address: addr.to_string(),
    };
    let _ = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Adding data to burnlist
    let msg = Execute::SetResourceId {
        address: addr.to_string(),
        resource_id,
    };
    let _ = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    let msg = Execute::Deposit { resource_id, data };
    let res = execute(deps.as_mut(), env, info, msg).unwrap();

    assert_eq!(0, res.messages.len());
    let dep_amt_unint128 = Uint128::new(deposit_amount);
    assert_eq!(
        get_balance(deps.as_ref(), genesis),
        amount - dep_amt_unint128
    );
    assert_eq!(get_balance(deps.as_ref(), addr), dep_amt_unint128);
}

#[test]
fn test_proposal_with_burn_list() {
    let mut deps = mock_dependencies(&[Coin {
        amount: Uint128::default(),
        denom: String::default(),
    }]);

    let genesis = "genesis";
    let amount = Uint128::new(11223344);
    let proposal_amount: u128 = 1000;
    let resource_id = Uint64::new(1);
    do_instantiate_with_minter(deps.as_mut(), genesis, amount, genesis, None);

    let info = mock_info(genesis, &[]);
    let env = mock_env();

    // Adding data to whitelist
    let msg = Execute::WhiteList {
        address: genesis.to_string(),
    };
    let _ = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Adding data to burnlist
    let msg = Execute::BurnList {
        address: genesis.to_string(),
    };
    let _ = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Adding data to burnlist
    let msg = Execute::SetResourceId {
        address: genesis.to_string(),
        resource_id,
    };
    let _ = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    let proposal_data = ProposalData {
        amount: proposal_amount,
        recipient_address: genesis.to_string(),
    }
    .encode();

    let msg = Execute::Proposal {
        resource_id,
        data: proposal_data,
    };
    let res = execute(deps.as_mut(), env, info, msg).unwrap();

    let updated_amount = amount.checked_add(Uint128::from(proposal_amount)).unwrap();
    assert_eq!(0, res.messages.len());
    assert_eq!(get_balance(deps.as_ref(), genesis), updated_amount);
}

#[test]
fn test_proposal_without_burn_list() {
    let mut deps = mock_dependencies(&[Coin {
        amount: Uint128::default(),
        denom: String::default(),
    }]);

    let genesis = "genesis";
    let receiver = "receiver";
    let amount = Uint128::new(11223344);
    let proposal_amount: u128 = 1000;
    let resource_id = Uint64::new(1);
    do_instantiate_with_minter(deps.as_mut(), genesis, amount, genesis, None);

    let info = mock_info(genesis, &[]);
    let env = mock_env();

    // Adding data to whitelist
    let msg = Execute::WhiteList {
        address: receiver.to_string(),
    };
    let _ = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Adding data to burnlist
    let msg = Execute::SetResourceId {
        address: receiver.to_string(),
        resource_id,
    };
    let _ = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    let proposal_data = ProposalData {
        amount: proposal_amount,
        recipient_address: genesis.to_string(),
    }
    .encode();

    let msg = Execute::Proposal {
        resource_id,
        data: proposal_data,
    };
    let res = execute(deps.as_mut(), env, info, msg).unwrap();

    let proposal_amount_uint128 = Uint128::from(proposal_amount);
    let updated_amount = amount.checked_sub(proposal_amount_uint128).unwrap();
    assert_eq!(0, res.messages.len());
    assert_eq!(get_balance(deps.as_ref(), genesis), updated_amount);
    assert_eq!(
        get_balance(deps.as_ref(), receiver),
        proposal_amount_uint128
    );
}

#[test]
fn test_withdraw() {
    let mut deps = mock_dependencies(&[Coin {
        amount: Uint128::default(),
        denom: String::default(),
    }]);

    let genesis = "genesis";
    let receiver = "receiver";
    let amount = Uint128::new(11223344);
    let withdrawal_amount: u128 = 1000;
    let resource_id = Uint64::new(1);
    do_instantiate(deps.as_mut(), &genesis, amount);

    let info = mock_info(genesis, &[]);
    let env = mock_env();

    // Adding data to whitelist
    let msg = Execute::WhiteList {
        address: receiver.to_string(),
    };
    let _ = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Adding data to burnlist
    let msg = Execute::SetResourceId {
        address: receiver.to_string(),
        resource_id,
    };
    let _ = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    let withdrawal_data = WithdrawData {
        amount: withdrawal_amount,
        recipient_address: genesis.to_string(),
        token_address: receiver.to_string(),
    }
    .encode();

    let msg = Execute::Withdraw {
        data: withdrawal_data,
    };
    let res = execute(deps.as_mut(), env, info, msg).unwrap();

    let withdrawal_amount_uint128 = Uint128::from(withdrawal_amount);
    let updated_amount = amount.checked_sub(withdrawal_amount_uint128).unwrap();
    assert_eq!(0, res.messages.len());
    assert_eq!(get_balance(deps.as_ref(), genesis), updated_amount);
    assert_eq!(
        get_balance(deps.as_ref(), receiver),
        withdrawal_amount_uint128
    );
}
