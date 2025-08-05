#![no_std]

use alloc::{
    string::{String, ToString as _},
    vec::Vec,
};
use alloy_primitives::{Address, B256, U256, keccak256};
use cosmwasm_std::{Uint128, to_json_binary};
use valence_authorization_utils::{
    authorization::{AtomicSubroutine, AuthorizationMsg, Priority, Subroutine},
    authorization_message::{Message, MessageDetails, MessageType},
    domain::Domain,
    function::AtomicFunction,
    msg::ProcessorMessage,
    zk_authorization::ZkMessage,
};

extern crate alloc;

mod consts;

pub fn mapping_slot_key(holder: Address, slot_index: u64) -> B256 {
    // left-pad address to 32 bytes
    let mut addr_padded = [0u8; 32];
    addr_padded[12..].copy_from_slice(holder.as_slice());

    let slot_b256: B256 = U256::from(slot_index).into();

    // preimage = pad(addr) || pad(slot)
    let mut preimage = [0u8; 64];
    preimage[..32].copy_from_slice(&addr_padded);
    preimage[32..].copy_from_slice(slot_b256.as_slice());

    keccak256(preimage)
}

pub fn build_zk_msg(recipient: String, amount: u128) -> ZkMessage {
    let mint_cw20_msg = cw20::Cw20ExecuteMsg::Mint {
        recipient,
        amount: Uint128::new(amount),
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
        contract_address: valence_library_utils::LibraryAccountType::Addr(
            consts::CW20_ADDR.to_string(),
        ),
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
