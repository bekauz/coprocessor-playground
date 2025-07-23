use deploy::{
    INPUTS_DIR,
    inputs::input_type::{NeutronInputs, PATH_NEUTRON_CODE_IDS, UploadedContracts},
};
use std::{env, error::Error, fs};
use valence_domain_clients::{
    clients::neutron::NeutronClient, cosmos::grpc_client::GrpcSigningClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    let mnemonic = env::var("MNEMONIC").expect("mnemonic must be provided");

    let current_dir = env::current_dir()?;

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

    Ok(())
}
