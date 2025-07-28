#![no_std]

extern crate alloc;

use alloc::string::ToString as _;
use alloc::vec::Vec;
use alloy_rpc_types_eth::EIP1186AccountProofResponse;

use valence_coprocessor::Witness;

pub fn circuit(witnesses: Vec<Witness>) -> Vec<u8> {
    assert_eq!(
        witnesses.len(),
        2,
        "Expected 2 witnesses: account state proof & neutron addr"
    );
    // get the state proof from first witness
    let state = witnesses[0]
        .as_state_proof()
        .expect("Failed to get state proof bytes");

    let proof: EIP1186AccountProofResponse = serde_json::from_slice(&state.proof).unwrap();

    // this should contain the destination neutron address
    let neutron_addr_bytes = witnesses[1]
        .as_data()
        .expect("Failed to get neutron_addr bytes");
    let neutron_addr = core::str::from_utf8(neutron_addr_bytes)
        .expect("Failed to build neutron_addr string from bytes");

    let evm_balance_u128 =
        u128::try_from(proof.balance).expect("Failed to convert solidity U256 to u128");

    let zk_msg = circuit_a_core::build_zk_msg(neutron_addr.to_string(), evm_balance_u128);

    serde_json::to_vec(&zk_msg).unwrap()
}
