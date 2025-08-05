#![no_std]

extern crate alloc;

use alloc::string::ToString as _;
use alloc::vec::Vec;
use alloy_primitives::Address;
use alloy_rpc_types_eth::EIP1186AccountProofResponse;

use valence_coprocessor::Witness;

pub fn circuit(witnesses: Vec<Witness>) -> Vec<u8> {
    assert_eq!(
        witnesses.len(),
        3,
        "Expected 3 witnesses: account state proof, neutron addr, and erc20 addr"
    );
    // extract the witnesses
    let state_proof_bytes = witnesses[0]
        .as_state_proof()
        .expect("Failed to get state proof bytes");
    let proof: EIP1186AccountProofResponse =
        serde_json::from_slice(&state_proof_bytes.proof).unwrap();
    // let root = state_proof_bytes.root;

    let neutron_addr = core::str::from_utf8(witnesses[1].as_data().unwrap()).unwrap();

    // let erc20_addr = Address::from_slice(witnesses[2].as_data().unwrap());

    // naive, for testing
    let evm_balance = proof.storage_proof[0].value;
    let evm_balance_u128: u128 = evm_balance
        .try_into()
        .expect("failed to parse U256 -> u128");

    let zk_msg = circuit_a_core::build_zk_msg(neutron_addr.to_string(), evm_balance_u128);

    serde_json::to_vec(&zk_msg).unwrap()
}
