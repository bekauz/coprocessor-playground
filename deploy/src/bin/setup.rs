use std::env;

use deploy::steps::{
    deploy_coprocessor_app, instantiate_contracts, read_input, setup_authorizations, write_output,
};
use types::neutron_cfg::NeutronStrategyConfig;
use valence_domain_clients::clients::neutron::NeutronClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let mnemonic = env::var("MNEMONIC")?;
    let current_dir = env::current_dir()?;

    let neutron_inputs = read_input::run(current_dir.clone())?;

    let neutron_client = NeutronClient::new(
        &neutron_inputs.grpc_url,
        &neutron_inputs.grpc_port,
        &mnemonic,
        &neutron_inputs.chain_id,
    )
    .await?;

    let instantiation_outputs =
        instantiate_contracts::run(&neutron_client, neutron_inputs.code_ids).await?;

    let coprocessor_app_id =
        deploy_coprocessor_app::run(current_dir.clone(), &instantiation_outputs.cw20)?;

    let neutron_strategy_config = NeutronStrategyConfig {
        grpc_url: neutron_inputs.grpc_url,
        grpc_port: neutron_inputs.grpc_port,
        chain_id: neutron_inputs.chain_id,
        authorizations: instantiation_outputs.authorizations,
        processor: instantiation_outputs.processor,
        cw20: instantiation_outputs.cw20,
        coprocessor_app_id,
    };

    println!("neutron strategy config: {:?}", neutron_strategy_config);

    setup_authorizations::run(&neutron_client, &neutron_strategy_config).await?;

    write_output::run(current_dir, neutron_strategy_config)?;

    Ok(())
}
