use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use subxt::{OnlineClient, PolkadotConfig};
use tokio::sync::RwLock;

use crate::network::ChainConfig;

/// Connection pool managing lazy-connected subxt clients for multiple chains
/// across all networks.
///
/// Each chain gets one `OnlineClient<PolkadotConfig>`, created on first use
/// and cached for the lifetime of the server. Connections are cheap to clone
/// (Arc internally) but expensive to create (WebSocket + metadata download).
///
/// Chains are keyed by their globally unique name (e.g. "polkadot", "kusama-asset-hub").
pub struct ChainPool {
    clients: RwLock<HashMap<String, OnlineClient<PolkadotConfig>>>,
}

impl ChainPool {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            clients: RwLock::new(HashMap::new()),
        })
    }

    /// Get client for a chain by config. Lazy-connects on first call, caches by name.
    pub async fn get(&self, config: &ChainConfig) -> Result<OnlineClient<PolkadotConfig>> {
        // Check cache first (read lock — cheap, concurrent)
        if let Some(client) = self.clients.read().await.get(&config.name) {
            return Ok(client.clone());
        }

        // Not cached — connect (write lock — exclusive)
        tracing::info!(chain = %config.name, endpoint = %config.endpoint, "Connecting to chain");

        let client = OnlineClient::<PolkadotConfig>::from_url(&config.endpoint).await?;
        self.clients
            .write()
            .await
            .insert(config.name.clone(), client.clone());

        tracing::info!(chain = %config.name, "Connected");
        Ok(client)
    }
}
