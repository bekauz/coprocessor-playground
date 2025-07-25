#![no_std]

extern crate alloc;

use alloc::{format, string::ToString as _, vec, vec::Vec};
use alloy_primitives::hex;
use alloy_rpc_types_eth::EIP1186AccountProofResponse;
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
//   "eth_addr": "0x...",
//   "neutron_addr": "neutron1..."
// }
pub fn get_witnesses(args: Value) -> anyhow::Result<Vec<Witness>> {
    abi::log!(
        "received a proof request with arguments {}",
        serde_json::to_string(&args).unwrap_or_default()
    )?;

    let block =
        abi::get_latest_block(DOMAIN)?.ok_or_else(|| anyhow::anyhow!("no valid domain block"))?;

    let root = block.root;
    let block = format!("{:#x}", block.number);

    // unpack the addresses. ideally we would pass a pubkey here
    // and derive them
    let eth_addr = args["eth_addr"].as_str().unwrap();

    let proof = abi::alchemy(
        NETWORK,
        "eth_getProof",
        &json!([
            eth_addr,
            [], // no storage slots to prove, just need eth balance
            block,
        ]),
    )?;

    abi::log!(
        "received proof response from alchemy: {}",
        serde_json::to_string(&proof).unwrap_or_default()
    )?;

    let proof: EIP1186AccountProofResponse = serde_json::from_value(proof)?;

    abi::log!(
        "{}",
        serde_json::to_string(&json!({
            "account": eth_addr,
            "proof": proof,
            "block": block,
            "root": format!("0x{}", hex::encode(root)),
        }))
        .unwrap_or_default()
    )?;

    let proof = serde_json::to_vec(&proof)?;

    // generate the neutron addr witness
    let neutron_addr = args["neutron_addr"].as_str().unwrap().to_string();

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
    ]
    .to_vec();

    Ok(witnesses)
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
