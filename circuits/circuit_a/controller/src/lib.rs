#![no_std]

extern crate alloc;

use core::str::FromStr;

use alloc::{format, string::ToString as _, vec::Vec};
use alloy_primitives::{hex, keccak256, Address, B256, U256};
use alloy_rpc_types_eth::EIP1186AccountProofResponse;
// use circuit_a_core::mapping_slot;
use serde_json::{json, Value};
use valence_coprocessor::{StateProof, Witness};
use valence_coprocessor_wasm::abi;

const NETWORK: &str = "eth-mainnet";
const DOMAIN: &str = "ethereum-electra-alpha";

// This component contains off-chain logic executed as Wasm within the
// Valence ZK Coprocessor's sandboxed environment.
//
// This Controller acts as an intermediary between user inputs and the ZK circuit.
// Key responsibilities include:
// - receiving input arguments (often JSON) for proof requests
// - processing inputs to generate a "witness" (private and
//   public data the ZK circuit needs)
// - interacting with the Coprocessor service to initiate proof generation.
//
// The Controller handles proof computation results; it has an entrypoint
// function the Coprocessor calls upon successful proof generation,
// allowing the Controller to store the proof or log information.
//
// expects json input in the following format:
// {
//   "erc20": "0x...",
//   "eth_addr": "0x...",
//   "neutron_addr": "neutron1..."
// }
pub fn get_witnesses(args: Value) -> anyhow::Result<Vec<Witness>> {
    let args_pretty = serde_json::to_string_pretty(&args)?;
    abi::log!("received a proof request with arguments {args_pretty}")?;

    let erc20_addr = args["erc20"].as_str().unwrap();
    let erc20_addr = Address::from_str(erc20_addr)?;
    let eth_addr = args["eth_addr"].as_str().unwrap();
    let eth_addr = Address::from_str(eth_addr)?;
    let neutron_addr = args["neutron_addr"].as_str().unwrap().to_string();

    let block =
        abi::get_latest_block(DOMAIN)?.ok_or_else(|| anyhow::anyhow!("no valid domain block"))?;

    let root = block.root;
    let block = format!("{:#x}", block.number);

    // 9 for usdc, probably different index for most erc20
    let slot_key = mapping_slot_key(eth_addr, 9u64);

    abi::log!("storage key = 0x{}", hex::encode(slot_key))?;

    let proof = abi::alchemy(
        NETWORK,
        "eth_getProof",
        &json!([erc20_addr, [format!("0x{}", hex::encode(slot_key))], block,]),
    )?;

    let proof: EIP1186AccountProofResponse = serde_json::from_value(proof)?;

    let pretty_proof = serde_json::to_string_pretty(&proof)?;
    abi::log!("{pretty_proof}")?;

    let proof = serde_json::to_vec(&proof)?;

    let witnesses = [
        // witness 0: eth address state proof
        Witness::StateProof(StateProof {
            domain: DOMAIN.into(),
            root,
            payload: Default::default(),
            proof,
        }),
        // witness 1: neutron addr (destination)
        Witness::Data(neutron_addr.as_bytes().to_vec()),
        // witness 2: erc20 addr (src)
        Witness::Data(erc20_addr.to_vec()),
    ]
    .to_vec();

    Ok(witnesses)
}

fn mapping_slot_key(holder: Address, slot_index: u64) -> B256 {
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

pub fn entrypoint(args: Value) -> anyhow::Result<Value> {
    abi::log!(
        "received an entrypoint request with arguments {}",
        serde_json::to_string(&args).unwrap_or_default()
    )?;

    let cmd = args["payload"]["cmd"].as_str().unwrap();

    match cmd {
        "store" => {
            let path = args["payload"]["path"].as_str().unwrap().to_string();
            let bytes = serde_json::to_vec(&args).unwrap();

            abi::set_storage_file(&path, &bytes).unwrap();
        }

        _ => panic!("unknown entrypoint command"),
    }

    Ok(args)
}
