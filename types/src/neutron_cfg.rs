use serde::{Deserialize, Serialize};
use valence_strategist_utils::worker::ValenceWorkerTomlSerde;

pub const REGULAR_MINT_CW20_LABEL: &str = "mint_cw20";
pub const ZK_MINT_CW20_LABEL: &str = "zk_mint_cw20";

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

impl ValenceWorkerTomlSerde for NeutronStrategyConfig {}
