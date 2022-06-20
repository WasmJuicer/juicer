#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128 as U128, Uint256 as U256,
};

use cw_utils::must_pay;

use std::str::FromStr;

use cw2::set_contract_version;

use juicer::merkle_tree::MerkleTreeWithHistory;
use juicer::msg::PublicSignals;
use juicer::verifier::Verifier;

use crate::error::ContractError;
use crate::msg::{DepositMsg, ExecuteMsg, InstantiateMsg, IsKnownRootMsg, QueryMsg, WithdrawMsg};
use crate::state::{BASE_COIN, COMMITMENTS, NULLIFIER_HASHES, VERIFIER};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:juno-juicer";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    BASE_COIN.save(deps.storage, &Coin::new(msg.amount.u128(), msg.denom))?;

    let verifier = Verifier::new();

    VERIFIER.save(deps.storage, &verifier)?;

    let tree = MerkleTreeWithHistory::new(20);
    COMMITMENTS.save(deps.storage, &tree)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Deposit(msg) => execute_deposit(deps, info, msg),
        ExecuteMsg::Withdraw(msg) => execute_withdraw(deps, info, msg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::IsKnownRoot(msg) => to_binary(&query_is_known_root(deps, msg)?),
    }
}

pub fn query_is_known_root(deps: Deps, msg: IsKnownRootMsg) -> StdResult<bool> {
    let commitment_mt = COMMITMENTS.load(deps.storage)?;

    Ok(commitment_mt.is_known_root(&U256::from_str(&msg.root)?))
}

pub fn execute_deposit(
    deps: DepsMut,
    info: MessageInfo,
    msg: DepositMsg,
) -> Result<Response, ContractError> {
    let coin = BASE_COIN.load(deps.storage)?;

    let payment = must_pay(&info, &coin.denom)?;
    if payment != coin.amount {
        return Err(ContractError::InvalidAmount {
            denom: coin.denom,
            amount: coin.amount,
        });
    }

    let mut commitment_mt = COMMITMENTS.load(deps.storage)?;
    // TODO: confirm insert worked
    commitment_mt.insert(&U256::from_str(&msg.commitment)?);
    COMMITMENTS.save(deps.storage, &commitment_mt)?;

    Ok(Response::new()
        .add_attribute("action", "deposit")
        .add_attribute("from", info.sender))
}

pub fn execute_withdraw(
    deps: DepsMut,
    _info: MessageInfo,
    msg: WithdrawMsg,
) -> Result<Response, ContractError> {
    let coin = BASE_COIN.load(deps.storage)?;

    let recipient = deps.api.addr_validate(&msg.recipient)?;
    if msg.relayer != "" && msg.relayer != "0" {
        deps.api.addr_validate(&msg.relayer)?;
    }

    // TODO: check info.funds

    let public_signals = PublicSignals::from_values(
        msg.root.clone(),
        msg.nullifier_hash.clone(),
        msg.recipient.clone(),
        msg.relayer.clone(),
        msg.fee,
    );

    let commitment_mt = COMMITMENTS.load(deps.storage)?;
    assert_ne!(
        commitment_mt.current_root_index, 0,
        "commitment merkle tree shouldn't be 0"
    );

    // 1. check nullifier_hash is not in nullifier hashes
    match NULLIFIER_HASHES.may_load(deps.storage, msg.nullifier_hash.clone())? {
        Some(_) => return Err(ContractError::DuplicatedCommitment {}),
        None => (),
    };

    // 2. confirm root is ok
    if !commitment_mt.is_known_root(&U256::from_str(&msg.root).unwrap()) {
        return Err(ContractError::UnknownRoot {});
    }

    // 3. Confirm the circuit proof
    let verifier = VERIFIER.load(deps.storage)?;
    let proof = msg.proof.to_proof();
    let inputs = public_signals.get();
    if !verifier.verify_proof(proof, &inputs) {
        return Err(ContractError::InvalidProof {});
    };

    // 4. Store nullifier hash to nullifier_hashes map
    NULLIFIER_HASHES
        .save(deps.storage, msg.nullifier_hash, &true)
        .unwrap();

    // 5. Send the funds
    let mut msgs: Vec<BankMsg> = Vec::new();

    let amount_to_recipient = match coin.amount.checked_sub(U128::from(msg.fee)) {
        Ok(v) => v,
        Err(err) => {
            return Err(ContractError::FeesTooHigh {
                msg: err.to_string(),
            })
        }
    };

    msgs.push(BankMsg::Send {
        to_address: recipient.to_string(),
        amount: vec![Coin {
            denom: coin.denom.clone(),
            amount: amount_to_recipient,
        }],
    });
    if !msg.fee.is_zero() {
        msgs.push(BankMsg::Send {
            to_address: msg.relayer,
            amount: vec![Coin {
                denom: coin.denom.clone(),
                amount: msg.fee,
            }],
        });
    }

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "withdraw"))
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    use juicer::msg::Deposit;

    use super::*;

    #[test]
    fn test_deposit() {
        let mut deps = mock_dependencies();
        let info = mock_info(&"Alice".to_string(), &coins(10, "TKN"));

        // let deposit = generate_deposit();
        let deposit = Deposit::new(
            "276277773929387392791096474084808108569850403587654342680891529007770412737"
                .to_string(),
        );

        // instantiate an empty contract
        let instantiate_msg = InstantiateMsg {
            amount: U128::from(10 as u128),
            denom: "TKN".to_string(),
        };
        let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        let deposit_msg = DepositMsg {
            commitment: deposit.get_commitment(),
        };

        let msg = ExecuteMsg::Deposit(deposit_msg.clone());

        let info = mock_info(&"Alice".to_string(), &coins(10, "TKN"));
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        // TODO
        // let commitment = COMMITMENTS
        //     .load(&deps.storage, deposit.get_commitment())
        //     .unwrap();
        // assert_eq!(Uint128::new(10), commitment);
    }

    #[test]
    fn test_withdraw_1() {
        let mut deps = mock_dependencies();

        // instantiate an empty contract
        let instantiate_msg = InstantiateMsg {
            amount: U128::from(10 as u128),
            denom: "TKN".to_string(),
        };
        let info = mock_info(&"Alice".to_string(), &[]);

        let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        let mut tree = COMMITMENTS.load(&deps.storage).unwrap();

        let deposit = Deposit {
            nullifier: "54154714943715201094961901040590459639892306160131965986154511512546000403"
                .to_string(),
        };

        let leaf_index = tree
            .insert(&U256::from_str(&deposit.clone().get_commitment()).unwrap())
            .unwrap();

        COMMITMENTS.save(&mut deps.storage, &tree).unwrap();

        let msg = ExecuteMsg::Withdraw(WithdrawMsg {
            proof: juicer::msg::CircomProof::from(
                r#"
                {"pi_a":["13899269723484849480002065473374493568327469679987898626585656783152635224196","4644776364206331144208370772102729462540382294894335687634266360911567618285","1"],"pi_b":[["11550199660326834097658136558533988234178757731057308044978347076813572730094","2682881763463105242359875271001109719339722524261167828167916342514182934974"],["95039516498389015079170513998234052571784823209713661742933740886373624805","3428917488231875962754312177544595651247105738928930070869265869601586471119"],["1","0"]],"pi_c":["18932896497737520548726210332000803585517357164811625711564892288268655803594","3898942506810745753991535926637360084087400921771473613166702262820083122159","1"],"protocol":"groth16","curve":"bn128"}
                "#.to_string(),
            ),
            root: "7867364560627547019086598689541673085228895175200585554350937642876639323043".to_string(),
            nullifier_hash: deposit.get_nullifier_hash((leaf_index) as u128),
            recipient: "juno14spgzl9ps5tyev32ny74fa6m0s9q9828v0vrga".to_string(),
            relayer: "juno1am5sw4geda8xfvmn4pkzruhv8ah0l3jx5hgchh".to_string(),
            fee: U128::from(0 as u128),
        });
        let info = mock_info(&"Alice".to_string(), &[]);

        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(1, res.messages.len());
    }

    // #[test]
    // fn test_withdraw_20() {
    //     let mut deps = mock_dependencies();

    //     let deposit = Deposit::from_note("juno-juicer-86ca9e972ed3784d9407f431e045be9b3c3c913327b0d3a669edce2ef1399f13578e9a6ae07cd5bc749d41c33b03e876906fb36803508bec87c86ce5b142".to_string());
    //     COMMITMENTS
    //         .save(&mut deps.storage, deposit.clone().get_commitment(), &true)
    //         .unwrap();

    //     for _ in 0..20 {
    //         let d = Deposit::new();
    //         COMMITMENTS
    //             .save(&mut deps.storage, d.get_commitment(), &true)
    //             .unwrap();
    //     }

    //     // instantiate an empty contract
    //     let instantiate_msg = InstantiateMsg {
    //         amount: 10,
    //         denom: "TKN".to_string(),
    //     };
    //     let info = mock_info(&"Alice".to_string(), &[]);

    //     let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
    //     assert_eq!(0, res.messages.len());

    //     let msg = ExecuteMsg::Withdraw(WithdrawMsg {
    //         note: "juno-juicer-86ca9e972ed3784d9407f431e045be9b3c3c913327b0d3a669edce2ef1399f13578e9a6ae07cd5bc749d41c33b03e876906fb36803508bec87c86ce5b142".to_string()
    //     });
    //     let info = mock_info(&"Alice".to_string(), &[]);

    //     let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    //     assert_eq!(1, res.messages.len());

    //     assert!(false);
    // }
}
