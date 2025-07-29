use std::error::Error;

use async_trait::async_trait;
use cosmwasm_std::Binary;
use cw20::{AllAccountsResponse, Cw20QueryMsg};
use log::info;
use serde_json::json;
use types::neutron_cfg::ZK_MINT_CW20_LABEL;
use valence_domain_clients::{
    coprocessor::base_client::{Base64, CoprocessorBaseClient, Proof},
    cosmos::{base_client::BaseClient, grpc_client::GrpcSigningClient, wasm_client::WasmClient},
};
use valence_strategist_utils::worker::ValenceWorker;

use crate::strategy::Strategy;

// implement the ValenceWorker trait for the Strategy struct.
// This trait defines the main loop of the strategy and inherits
// the default implementation for spawning the worker.
#[async_trait]
impl ValenceWorker for Strategy {
    fn get_name(&self) -> String {
        format!("Valence X-Vault: {}", self.label)
    }

    async fn cycle(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
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
        let (proof_program, inputs_program) = decode(resp.program)?;
        let (proof_domain, inputs_domain) = decode(resp.domain)?;

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

        // construct the zk authorization registration message
        let execute_zk_authorization_msg =
            valence_authorization_utils::msg::PermissionlessMsg::ExecuteZkAuthorization {
                label: ZK_MINT_CW20_LABEL.to_string(),
                message: Binary::from(inputs_program),
                proof: Binary::from(proof_program),
                domain_message: Binary::from(inputs_domain),
                domain_proof: Binary::from(proof_domain),
            };

        // execute the zk authorization. this will perform the verification
        // and, if successful, push the msg to the processor
        info!(target: "STRATEGIST", "executing zk authorization");

        let tx_resp = self
            .neutron_client
            .execute_wasm(
                &self.neutron_cfg.authorizations,
                valence_authorization_utils::msg::ExecuteMsg::PermissionlessAction(
                    execute_zk_authorization_msg,
                ),
                vec![],
                None,
            )
            .await?;

        // poll for inclusion to avoid account sequence mismatch errors
        self.neutron_client.poll_for_tx(&tx_resp.hash).await?;

        info!(target: "STRATEGIST", "tickticktick");
        let tx_resp = self
            .neutron_client
            .execute_wasm(
                &self.neutron_cfg.processor,
                valence_processor_utils::msg::ExecuteMsg::PermissionlessAction(
                    valence_processor_utils::msg::PermissionlessMsg::Tick {},
                ),
                vec![],
                None,
            )
            .await?;

        self.neutron_client.poll_for_tx(&tx_resp.hash).await?;

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
