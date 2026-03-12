use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use subxt::{OnlineClient, PolkadotConfig};
use tokio::sync::RwLock;

use crate::network::{ChainConfig, Network};

/// Connection pool managing lazy-connected subxt clients for multiple chains.
///
/// Each chain gets one `OnlineClient<PolkadotConfig>`, created on first use
/// and cached for the lifetime of the server. Connections are cheap to clone
/// (Arc internally) but expensive to create (WebSocket + metadata download).
pub struct ChainPool {
    pub network: Network,
    clients: RwLock<HashMap<String, OnlineClient<PolkadotConfig>>>,
}

impl ChainPool {
    pub fn new(network: Network) -> Arc<Self> {
        Arc::new(Self {
            network,
            clients: RwLock::new(HashMap::new()),
        })
    }

    /// Get client for a named chain. Lazy-connects on first call.
    pub async fn get(&self, chain_name: &str) -> Result<OnlineClient<PolkadotConfig>> {
        // Check cache first (read lock — cheap, concurrent)
        if let Some(client) = self.clients.read().await.get(chain_name) {
            return Ok(client.clone());
        }

        // Not cached — connect (write lock — exclusive)
        let config = self.network.config_for(chain_name)?;
        tracing::info!(chain = chain_name, endpoint = %config.endpoint, "Connecting to chain");

        let client = OnlineClient::<PolkadotConfig>::from_url(&config.endpoint).await?;
        self.clients
            .write()
            .await
            .insert(chain_name.to_string(), client.clone());

        tracing::info!(chain = chain_name, "Connected");
        Ok(client)
    }

    /// Get config for a named chain.
    pub fn config(&self, chain_name: &str) -> Result<&ChainConfig> {
        self.network.config_for(chain_name)
    }

    // --- Shorthand accessors for domain tools ---

    /// Relay chain client.
    pub async fn relay(&self) -> Result<OnlineClient<PolkadotConfig>> {
        self.get(&self.network.relay.name).await
    }

    /// Relay chain config.
    pub fn relay_config(&self) -> &ChainConfig {
        &self.network.relay
    }

    /// Asset Hub parachain client.
    pub async fn asset_hub(&self) -> Result<OnlineClient<PolkadotConfig>> {
        self.get(&self.network.asset_hub.name).await
    }

    /// Asset Hub chain config.
    pub fn asset_hub_config(&self) -> &ChainConfig {
        &self.network.asset_hub
    }

    /// Bridge Hub parachain client.
    pub async fn bridge_hub(&self) -> Result<OnlineClient<PolkadotConfig>> {
        self.get(&self.network.bridge_hub.name).await
    }

    /// Bridge Hub chain config.
    pub fn bridge_hub_config(&self) -> &ChainConfig {
        &self.network.bridge_hub
    }

    /// People chain client.
    pub async fn people(&self) -> Result<OnlineClient<PolkadotConfig>> {
        self.get(&self.network.people.name).await
    }

    /// People chain config.
    pub fn people_config(&self) -> &ChainConfig {
        &self.network.people
    }

    /// Collectives parachain client (fellowship, salary).
    /// Returns an error if the network has no Collectives chain (e.g. Kusama).
    pub async fn collectives(&self) -> Result<OnlineClient<PolkadotConfig>> {
        let config = self.network.collectives.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Collectives chain is not available on this network"))?;
        self.get(&config.name).await
    }

    /// Collectives chain config (None on networks without Collectives, e.g. Kusama).
    pub fn collectives_config(&self) -> Option<&ChainConfig> {
        self.network.collectives.as_ref()
    }

    /// Coretime parachain client (blockspace allocation).
    pub async fn coretime(&self) -> Result<OnlineClient<PolkadotConfig>> {
        self.get(&self.network.coretime.name).await
    }

    /// Coretime chain config.
    pub fn coretime_config(&self) -> &ChainConfig {
        &self.network.coretime
    }

    /// Get relay + collectives clients in parallel (for fellowship queries that
    /// need both chains).
    pub async fn relay_and_collectives(
        &self,
    ) -> Result<(OnlineClient<PolkadotConfig>, OnlineClient<PolkadotConfig>)> {
        let (relay, collectives) = tokio::try_join!(self.relay(), self.collectives())?;
        Ok((relay, collectives))
    }

    /// Get relay + asset hub clients in parallel (for cross-chain balance queries).
    pub async fn relay_and_asset_hub(
        &self,
    ) -> Result<(OnlineClient<PolkadotConfig>, OnlineClient<PolkadotConfig>)> {
        let (relay, asset_hub) = tokio::try_join!(self.relay(), self.asset_hub())?;
        Ok((relay, asset_hub))
    }
}
