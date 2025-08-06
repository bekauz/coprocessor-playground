use serde::{Deserialize, Serialize};

pub const REGULAR_MINT_CW20_LABEL: &str = "mint_cw20";
pub const ZK_MINT_CW20_LABEL: &str = "zk_mint_cw20";

pub const INPUTS_DIR: &str = "deploy/src/inputs";
pub const OUTPUTS_DIR: &str = "artifacts/deploy";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeutronStrategyConfig {
    // node info
    pub grpc_url: String,
    pub grpc_port: String,
    pub chain_id: String,

    // contracts
    pub authorizations: String,
    pub processor: String,
    pub cw20: String,

    // coprocessor app id
    pub coprocessor_app_id: String,
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct CircuitInputs {
//     pub erc20: String,
//     pub eth_addr: String,
//     pub neutron_addr: String,
// }
