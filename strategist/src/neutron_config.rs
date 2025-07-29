use serde::{Deserialize, Serialize};
use valence_strategist_utils::worker::ValenceWorkerTomlSerde;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeutronStrategyConfig {
    // node connection info
    pub grpc_url: String,
    pub grpc_port: String,
    pub chain_id: String,

    /// authorizations module
    pub authorizations: String,
    /// processor coupled with the authorizations
    pub processor: String,
}

impl ValenceWorkerTomlSerde for NeutronStrategyConfig {}
