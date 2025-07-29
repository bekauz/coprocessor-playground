use std::env;

use dotenv::dotenv;
use log::{info, warn};
use strategist::{neutron_config::NeutronStrategyConfig, strategy::Strategy};
use valence_strategist_utils::worker::ValenceWorker;
use valence_strategist_utils::worker::ValenceWorkerTomlSerde;

const RUNNER: &str = "runner";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // load environment variables
    dotenv().ok();

    // initialize the logger
    env_logger::init();

    info!(target: RUNNER, "starting the strategist runner");

    let neutron_cfg_path = env::var("NEUTRON_CFG_PATH")?;

    info!(target: RUNNER, "Using configuration files:");
    info!(target: RUNNER, "  Neutron: {neutron_cfg_path}");

    let neutron_cfg = NeutronStrategyConfig::from_file(neutron_cfg_path)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let strategy = Strategy::new(neutron_cfg).await?;

    info!(target: RUNNER, "strategy initialized");
    info!(target: RUNNER, "starting the strategist");

    let strategist_join_handle = strategy.start();

    // join here will wait for the strategist thread to finish which should never happen in practice since it runs an infinite stayalive loop
    match strategist_join_handle.join() {
        Ok(t) => warn!(target: RUNNER, "strategist thread completed: {t:?}"),
        Err(e) => warn!(target: RUNNER, "strategist thread completed with error: {e:?}"),
    }

    Ok(())
}
