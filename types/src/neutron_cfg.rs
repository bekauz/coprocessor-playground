use serde::{Deserialize, Serialize};

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
