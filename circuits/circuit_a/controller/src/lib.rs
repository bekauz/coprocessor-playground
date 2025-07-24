#![no_std]

extern crate alloc;

use alloc::{format, string::ToString as _, vec, vec::Vec};
use alloy_rpc_types_eth::EIP1186AccountProofResponse;
use serde_json::Value;
use valence_coprocessor::{StateProof, Witness};
use valence_coprocessor_wasm::abi;

const NETWORK: &str = "eth-mainnet";
const DOMAIN: &str = "ethereum-electra-alpha";

pub fn get_witnesses(args: Value) -> anyhow::Result<Vec<Witness>> {
    abi::log!(
        "received a proof request with arguments {}",
        serde_json::to_string(&args).unwrap_or_default()
    )?;

    let block =
        abi::get_latest_block(DOMAIN)?.ok_or_else(|| anyhow::anyhow!("no valid domain block"))?;

    let root = block.root;
    let block = format!("{:#x}", block.number);

    // unpack the destination addr arg
    let neutron_addr = args["neutron_addr"].as_str().unwrap();

    // TODO: find out how to fetch this
    let proof: EIP1186AccountProofResponse = EIP1186AccountProofResponse::default();
    let proof = serde_json::to_vec(&proof)?;

    let ntrn_addr_vec = bincode::serde::encode_to_vec(neutron_addr, bincode::config::standard())?;

    let proof = Witness::StateProof(StateProof {
        domain: DOMAIN.into(),
        root,
        payload: Default::default(),
        proof,
    });
    let neutron_addr_witness = Witness::Data(ntrn_addr_vec);

    Ok(vec![proof, neutron_addr_witness])
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
