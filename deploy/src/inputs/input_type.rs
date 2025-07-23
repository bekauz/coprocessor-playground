use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct NeutronInputs {
    pub grpc_url: String,
    pub grpc_port: String,
    pub chain_id: String,
    pub owner: String,
    pub coprocessor_app_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UploadedContracts {
    pub code_ids: HashMap<String, u64>,
}

pub const PATH_NEUTRON_CODE_IDS: &str = "deploy/src/inputs/neutron_code_ids.toml";
