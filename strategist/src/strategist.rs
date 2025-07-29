use std::error::Error;

use async_trait::async_trait;
use log::info;
use valence_strategist_utils::worker::ValenceWorker;

use crate::strategy::Strategy;

// implement the ValenceWorker trait for the Strategy struct.
// This trait defines the main loop of the strategy and inherits
// the default implementation for spawning the worker.
#[async_trait]
impl ValenceWorker for Strategy {
    fn get_name(&self) -> String {
        format!("Valence X-Vault: {}", self.label)
    }

    async fn cycle(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!(target: "WORKER", "{}: Starting cycle...", self.get_name());

        Ok(())
    }
}
