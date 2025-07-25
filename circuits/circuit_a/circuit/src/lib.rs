// #![no_std]
#![cfg_attr(not(test), no_std)]
extern crate alloc;

use alloc::string::String;
use alloc::string::ToString as _;
use alloc::vec::Vec;
use alloy_rpc_types_eth::EIP1186AccountProofResponse;
use cosmwasm_std::to_json_binary;
use cosmwasm_std::Uint128;
use valence_authorization_utils::authorization::{
    AtomicSubroutine, AuthorizationMsg, Priority, Subroutine,
};
use valence_authorization_utils::authorization_message::{Message, MessageDetails, MessageType};
use valence_authorization_utils::domain::Domain;
use valence_authorization_utils::function::AtomicFunction;
use valence_authorization_utils::msg::ProcessorMessage;
use valence_authorization_utils::zk_authorization::ZkMessage;
use valence_coprocessor::Witness;

// TODO: can we fetch this from artifacts/deploy?
const CW20_ADDR: &str = "neutron10rrvph3ksn052mjqwz3gzprd8ef7gn6xgg7g539zdwqrmmfxxz0q7465ps";

pub fn circuit(witnesses: Vec<Witness>) -> Vec<u8> {
    // get the state proof from first witness
    let state = witnesses[0].as_state_proof().unwrap();
    // this should contain the destination neutron address
    let destination = witnesses[1].as_data().unwrap();

    let proof: EIP1186AccountProofResponse = serde_json::from_slice(&state.proof).unwrap();
    let neutron_addr: String =
        bincode::serde::decode_from_slice(destination, bincode::config::standard())
            .unwrap()
            .0;

    let evm_balance = proof.balance;
    let evm_balance_u128 = u128::try_from(evm_balance).unwrap();

    let zk_msg = build_zk_msg(neutron_addr, evm_balance_u128);

    serde_json::to_vec(&zk_msg).unwrap()
}

fn build_zk_msg(recipient: String, amount: impl Into<Uint128>) -> ZkMessage {
    let mint_cw20_msg = cw20::Cw20ExecuteMsg::Mint {
        recipient,
        amount: amount.into(),
    };

    let processor_msg = ProcessorMessage::CosmwasmExecuteMsg {
        msg: to_json_binary(&mint_cw20_msg).unwrap(),
    };

    let function = AtomicFunction {
        domain: Domain::Main,
        message_details: MessageDetails {
            message_type: MessageType::CosmwasmExecuteMsg,
            message: Message {
                name: "mint".to_string(),
                params_restrictions: None,
            },
        },
        contract_address: valence_library_utils::LibraryAccountType::Addr(CW20_ADDR.to_string()),
    };

    let subroutine = AtomicSubroutine {
        functions: Vec::from([function]),
        retry_logic: None,
        expiration_time: None,
    };

    let message = AuthorizationMsg::EnqueueMsgs {
        id: 0,
        msgs: Vec::from([processor_msg]),
        subroutine: Subroutine::Atomic(subroutine),
        priority: Priority::Medium,
        expiration_time: None,
    };

    ZkMessage {
        registry: 0,
        block_number: 0,
        domain: Domain::Main,
        authorization_contract: None,
        message,
    }
}
