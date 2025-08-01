use async_trait::async_trait;
use cw20::{AllAccountsResponse, Cw20QueryMsg};
use log::info;
use serde_json::json;
use types::neutron_cfg::ZK_MINT_CW20_LABEL;
use valence_coordinator_sdk::coordinator::ValenceCoordinator;
use valence_domain_clients::{
    coprocessor::base_client::{Base64, CoprocessorBaseClient, Proof},
    cosmos::{grpc_client::GrpcSigningClient, wasm_client::WasmClient},
};

use crate::strategy::Strategy;

// implement the ValenceWorker trait for the Strategy struct.
// This trait defines the main loop of the strategy and inherits
// the default implementation for spawning the worker.
#[async_trait]
impl ValenceCoordinator for Strategy {
    fn get_name(&self) -> String {
        format!("Valence X-Vault: {}", self.label)
    }

    async fn cycle(&mut self) -> anyhow::Result<()> {
        info!(target: "STRATEGIST", "{}: Starting cycle...", self.get_name());

        let eth_addr = "0x8d41bb082C6050893d1eC113A104cc4C087F2a2a";
        let ntrn_addr = self
            .neutron_client
            .get_signing_client()
            .await?
            .address
            .to_string();

        let proof_request = json!({
          "eth_addr": eth_addr,
          "neutron_addr": ntrn_addr
        });

        info!(target: "STRATEGIST", "posting proof request: {:?}", proof_request);

        let resp = self
            .coprocessor_client
            .prove(&self.neutron_cfg.coprocessor_app_id, &proof_request)
            .await?;

        info!(target: "STRATEGIST", "received zkp: {:?}", resp);

        // extract the program and domain parameters by decoding the zkp
        let program_proof = decode(resp.program)?;
        let domain_proof = decode(resp.domain)?;

        let cw20_accounts_response: AllAccountsResponse = self
            .neutron_client
            .query_contract_state(
                &self.neutron_cfg.cw20,
                Cw20QueryMsg::AllAccounts {
                    start_after: None,
                    limit: None,
                },
            )
            .await?;
        info!(target: "STRATEGIST", "cw20 accounts resp: {:?}", cw20_accounts_response);

        info!(target: "STRATEGIST", "posting zkp to the authorizations contract");
        // execute the zk authorization. this will perform the verification
        // and, if successful, push the msg to the processor
        valence_coordinator_sdk::core::cw::post_zkp_on_chain(
            &self.neutron_client,
            &self.neutron_cfg.authorizations,
            ZK_MINT_CW20_LABEL,
            program_proof,
            domain_proof,
        ).await?;

        info!(target: "STRATEGIST", "ticking the processor...");
        // tick the processor
        valence_coordinator_sdk::core::cw::tick(
            &self.neutron_client,
            &self.neutron_cfg.processor,
        ).await?;

        let cw20_accounts_response: AllAccountsResponse = self
            .neutron_client
            .query_contract_state(
                &self.neutron_cfg.cw20,
                Cw20QueryMsg::AllAccounts {
                    start_after: None,
                    limit: None,
                },
            )
            .await?;
        info!(target: "STRATEGIST", "cw20 accounts resp: {:?}", cw20_accounts_response);

        Ok(())
    }
}

fn decode(a: Proof) -> anyhow::Result<(Vec<u8>, Vec<u8>)> {
    let proof = Base64::decode(&a.proof)?;
    let inputs = Base64::decode(&a.inputs)?;

    Ok((proof, inputs))
}
