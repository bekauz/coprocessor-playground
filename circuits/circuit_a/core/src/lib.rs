#![no_std]

use alloc::{
    string::{String, ToString as _},
    vec::Vec,
};
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
        contract_address: valence_library_utils::LibraryAccountType::Addr(consts::CW20_ADDR.to_string()),
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
