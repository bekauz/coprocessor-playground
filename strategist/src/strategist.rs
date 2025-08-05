use async_trait::async_trait;
use common::ZK_MINT_CW20_LABEL;
use cw20::{BalanceResponse, Cw20QueryMsg};
use log::info;
use serde_json::json;
use valence_coordinator_sdk::coordinator::ValenceCoordinator;
use valence_domain_clients::{
    coprocessor::base_client::{Base64, CoprocessorBaseClient, Proof},
    cosmos::{grpc_client::GrpcSigningClient, wasm_client::WasmClient},
};

use crate::strategy::Strategy;

const STRATEGIST_LOG_TARGET: &str = "STRATEGIST";

// implement the ValenceWorker trait for the Strategy struct.
// This trait defines the main loop of the strategy and inherits
// the default implementation for spawning the worker.
#[async_trait]
impl ValenceCoordinator for Strategy {
    fn get_name(&self) -> String {
        format!("Valence X-Vault: {}", self.label)
    }

    async fn cycle(&mut self) -> anyhow::Result<()> {
        info!(target: STRATEGIST_LOG_TARGET, "{}: Starting cycle...", self.get_name());

        let usdc_erc20_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";

        let eth_addr = "0x8d41bb082C6050893d1eC113A104cc4C087F2a2a";
        let ntrn_addr = self
            .neutron_client
            .get_signing_client()
            .await?
            .address
            .to_string();

        let proof_request = json!({
          "erc20": usdc_erc20_addr,
          "eth_addr": eth_addr,
          "neutron_addr": ntrn_addr
        });

        info!(target: STRATEGIST_LOG_TARGET, "posting proof request: {:?}", proof_request);

        let resp = self
            .coprocessor_client
            .prove(&self.neutron_cfg.coprocessor_app_id, &proof_request)
            .await?;

        info!(target: STRATEGIST_LOG_TARGET, "received zkp: {:?}", resp);

        // extract the program and domain parameters by decoding the zkp
        let program_proof = decode(resp.program)?;
        let domain_proof = decode(resp.domain)?;

        let cw20_balance: BalanceResponse = self
            .neutron_client
            .query_contract_state(&self.neutron_cfg.cw20, Cw20QueryMsg::Balance {
                address: ntrn_addr.to_string(),
            })
            .await?;
        info!(target: STRATEGIST_LOG_TARGET, "cw20 balance pre-proof: {:?}", cw20_balance);

        info!(target: STRATEGIST_LOG_TARGET, "posting zkp to the authorizations contract");
        // execute the zk authorization. this will perform the verification
        // and, if successful, push the msg to the processor
        valence_coordinator_sdk::core::cw::post_zkp_on_chain(
            &self.neutron_client,
            &self.neutron_cfg.authorizations,
            ZK_MINT_CW20_LABEL,
            program_proof,
            domain_proof,
        )
        .await?;

        info!(target: STRATEGIST_LOG_TARGET, "ticking the processor...");
        // tick the processor
        valence_coordinator_sdk::core::cw::tick(&self.neutron_client, &self.neutron_cfg.processor)
            .await?;

        let cw20_balance: BalanceResponse = self
            .neutron_client
            .query_contract_state(&self.neutron_cfg.cw20, Cw20QueryMsg::Balance {
                address: ntrn_addr,
            })
            .await?;
        info!(target: STRATEGIST_LOG_TARGET, "cw20 balance pre-proof: {:?}", cw20_balance);

        Ok(())
    }
}

fn decode(a: Proof) -> anyhow::Result<(Vec<u8>, Vec<u8>)> {
    let proof = Base64::decode(&a.proof)?;
    let inputs = Base64::decode(&a.inputs)?;

    Ok((proof, inputs))
}
