use std::{fs, path::PathBuf};

use types::neutron_cfg::NeutronStrategyConfig;

use crate::inputs::OUTPUTS_DIR;

pub fn run(cd: PathBuf, neutron_cfg: NeutronStrategyConfig) -> anyhow::Result<()> {
    println!("writing outputs...");

    // Save the Neutron Strategy Config to a toml file
    let neutron_cfg_toml =
        toml::to_string(&neutron_cfg).expect("Failed to serialize Neutron Strategy Config");

    let target_path = cd.join(format!("{OUTPUTS_DIR}/neutron_strategy_config.toml"));
    println!("writing neutron_strategy_config.toml to: {target_path:?}");

    fs::write(target_path, neutron_cfg_toml)
        .expect("Failed to write Neutron Strategy Config to file");

    Ok(())
}
