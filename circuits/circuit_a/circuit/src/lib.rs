// #![no_std]
#![cfg_attr(not(test), no_std)]
extern crate alloc;

use alloc::vec::Vec;
use alloy_rpc_types_eth::EIP1186AccountProofResponse;
use valence_coprocessor::Witness;

pub fn circuit(witnesses: Vec<Witness>) -> Vec<u8> {
    // get the state proof from first witness
    let state = witnesses[0].as_state_proof().unwrap();
    // this should contain the destination neutron address
    let destination = witnesses[1].as_data().unwrap();

    // TODO: think about whether it's possible to easily prevent
    // users from calling this multiple times. need some one-time
    // signature or something. but no additional contracts or whatnot.
    let root = state.root;

    let proof: EIP1186AccountProofResponse = serde_json::from_slice(&state.proof).unwrap();

    let evm_addr = proof.address;
    let evm_balance = proof.balance;

    alloc::vec::Vec::new()
}
