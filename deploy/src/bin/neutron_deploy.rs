use cw20::MinterResponse;
use deploy::{
    INPUTS_DIR,
    inputs::{
        OUTPUTS_DIR, VALENCE_NEUTRON_VERIFICATION_GATEWAY,
        input_type::{NeutronInputs, PATH_NEUTRON_CODE_IDS, UploadedContracts},
    },
};
use std::{env, error::Error, fs, time::SystemTime};
use types::neutron_cfg::NeutronStrategyConfig;
use valence_domain_clients::{
    clients::neutron::NeutronClient,
    cosmos::{base_client::BaseClient, grpc_client::GrpcSigningClient, wasm_client::WasmClient},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    let mnemonic = env::var("MNEMONIC").expect("mnemonic must be provided");

    let current_dir = env::current_dir()?;
    println!("cd: {current_dir:?}");

    let parameters = fs::read_to_string(current_dir.join(format!("{INPUTS_DIR}/neutron.toml")))
        .expect("Failed to read file");

    let input_params: NeutronInputs =
        toml::from_str(&parameters).expect("failed to parse neutron toml inputs");

    println!("input params: {:?}", input_params);

    // Read code IDS from the code ids file
    let code_ids_content = fs::read_to_string(current_dir.join(PATH_NEUTRON_CODE_IDS))
        .expect("Failed to read code ids file");
    let uploaded_contracts: UploadedContracts =
        toml::from_str(&code_ids_content).expect("Failed to parse code ids");

    println!("code ids: {:?}", uploaded_contracts);

    let neutron_client = NeutronClient::new(
        &input_params.grpc_url,
        &input_params.grpc_port,
        &mnemonic,
        &input_params.chain_id,
    )
    .await?;

    let my_address = neutron_client
        .get_signing_client()
        .await?
        .address
        .to_string();

    println!("runner address: {my_address}");

    let code_id_authorization = *uploaded_contracts.code_ids.get("authorization").unwrap();
    let code_id_processor = *uploaded_contracts.code_ids.get("processor").unwrap();
    let code_id_cw20 = *uploaded_contracts.code_ids.get("cw20_base").unwrap();

    let now = SystemTime::now();
    let salt_raw = now
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs()
        .to_string();
    let salt = hex::encode(salt_raw.as_bytes());

    let predicted_processor_address = neutron_client
        .predict_instantiate2_addr(code_id_processor, salt.clone(), my_address.clone())
        .await?
        .address;

    // Owner will initially be the deploy address and eventually will be transferred to the owned address
    let authorization_instantiate_msg = valence_authorization_utils::msg::InstantiateMsg {
        owner: my_address.to_string(),
        sub_owners: vec![],
        processor: predicted_processor_address.clone(),
    };

    let authorization_address = neutron_client
        .instantiate2(
            code_id_authorization,
            "authorization".to_string(),
            authorization_instantiate_msg,
            Some(my_address.to_string()),
            salt.clone(),
        )
        .await?;
    println!("Authorization instantiated: {authorization_address}");

    let processor_instantiate_msg = valence_processor_utils::msg::InstantiateMsg {
        authorization_contract: authorization_address.clone(),
        polytone_contracts: None,
    };

    let processor_address = neutron_client
        .instantiate2(
            code_id_processor,
            "processor".to_string(),
            processor_instantiate_msg,
            Some(my_address.to_string()),
            salt.clone(),
        )
        .await?;
    println!("Processor instantiated: {processor_address}");

    // Set the verification gateway address on the authorization contract
    let set_verification_gateway_msg =
        valence_authorization_utils::msg::ExecuteMsg::PermissionedAction(
            valence_authorization_utils::msg::PermissionedMsg::SetVerificationGateway {
                verification_gateway: VALENCE_NEUTRON_VERIFICATION_GATEWAY.to_string(),
            },
        );

    let set_verification_gateway_rx = neutron_client
        .execute_wasm(
            &authorization_address,
            set_verification_gateway_msg,
            vec![],
            None,
        )
        .await?;

    neutron_client
        .poll_for_tx(&set_verification_gateway_rx.hash)
        .await?;

    println!("Set verification gateway address to {VALENCE_NEUTRON_VERIFICATION_GATEWAY}");

    let cw20_init_msg = cw20_base::msg::InstantiateMsg {
        name: "test_playground".to_string(),
        symbol: "CWBASETEST".to_string(),
        decimals: 18,
        initial_balances: vec![],
        mint: Some(MinterResponse {
            minter: processor_address.to_string(),
            cap: None,
        }),
        marketing: None,
    };

    let cw20_addr = neutron_client
        .instantiate(
            code_id_cw20,
            "mirror_cw20".to_string(),
            cw20_init_msg,
            Some(my_address),
        )
        .await?;

    println!("CW20 Instantiated: {cw20_addr}");

    let cargo_valence_app = cargo_valence::App::default();

    let controller_path = "./circuits/circuit_a/controller";
    let circuit = "valence-coprocessor-app-circuit";
    let circuit_deployment_response =
        cargo_valence_app.deploy_circuit(Some(controller_path), circuit)?;

    let controller_id = match circuit_deployment_response.as_object() {
        Some(obj) => {
            let controller_id = obj.get("controller");
            match controller_id {
                Some(id) => id.to_string(),
                None => {
                    println!("failed to get controller from response object: {:?}", obj);
                    "deployment_error".to_string()
                }
            }
        }
        None => {
            println!(
                "failed to deploy the circuit: {:?}",
                circuit_deployment_response
            );
            "deployment_error".to_string()
        }
    };

    println!("controller_id: {controller_id}");

    let neutron_cfg = NeutronStrategyConfig {
        grpc_url: input_params.grpc_url,
        grpc_port: input_params.grpc_port,
        chain_id: input_params.chain_id,
        authorizations: authorization_address,
        processor: processor_address,
        cw20: cw20_addr,
        coprocessor_app_id: controller_id,
    };

    println!("Neutron Strategy Config created successfully");

    // Save the Neutron Strategy Config to a toml file
    let neutron_cfg_toml =
        toml::to_string(&neutron_cfg).expect("Failed to serialize Neutron Strategy Config");

    let target_path = current_dir.join(format!("{OUTPUTS_DIR}/neutron_strategy_config.toml"));
    println!("writing neutron_strategy_config.toml to: {target_path:?}");

    fs::write(target_path, neutron_cfg_toml)
        .expect("Failed to write Neutron Strategy Config to file");

    Ok(())
}
