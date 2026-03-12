use anyhow::{anyhow, Result};

/// Metadata for a single chain within a network.
#[derive(Debug, Clone)]
pub struct ChainConfig {
    pub name: String,
    pub endpoint: String,
    pub token_symbol: String,
    pub token_decimals: u8,
    pub ss58_prefix: u16,
}

/// A network is a group of related chains (relay + system parachains).
/// Tools query the network, not individual chains.
#[derive(Debug, Clone)]
pub struct Network {
    pub relay: ChainConfig,
    pub collectives: ChainConfig,
    pub asset_hub: ChainConfig,
    // Future: bridge_hub, coretime, people, etc.
}

impl Network {
    /// Get network name (relay chain name).
    pub fn name(&self) -> &str {
        &self.relay.name
    }

    /// Look up a chain config by name.
    pub fn config_for(&self, chain_name: &str) -> Result<&ChainConfig> {
        if chain_name == self.relay.name {
            Ok(&self.relay)
        } else if chain_name == self.collectives.name {
            Ok(&self.collectives)
        } else if chain_name == self.asset_hub.name {
            Ok(&self.asset_hub)
        } else {
            Err(anyhow!(
                "Unknown chain '{}'. Available: {}, {}, {}",
                chain_name,
                self.relay.name,
                self.collectives.name,
                self.asset_hub.name,
            ))
        }
    }

    /// All chain names in this network.
    pub fn chain_names(&self) -> Vec<&str> {
        vec![&self.relay.name, &self.collectives.name, &self.asset_hub.name]
    }

    /// Load network from environment variables.
    ///
    /// POLKADOT_NETWORK: "polkadot" (default), "kusama", "westend"
    /// Override individual endpoints:
    ///   POLKADOT_RELAY_URL, POLKADOT_COLLECTIVES_URL, POLKADOT_ASSET_HUB_URL
    pub fn from_env() -> Result<Self> {
        let network_name = std::env::var("POLKADOT_NETWORK")
            .unwrap_or_else(|_| "polkadot".to_string());

        let mut network = match network_name.as_str() {
            "polkadot" => Self::polkadot(),
            "kusama" => Self::kusama(),
            "westend" => Self::westend(),
            other => return Err(anyhow!(
                "Unknown network '{}'. Use: polkadot, kusama, westend", other
            )),
        };

        // Allow endpoint overrides
        if let Ok(url) = std::env::var("POLKADOT_RELAY_URL") {
            network.relay.endpoint = url;
        }
        if let Ok(url) = std::env::var("POLKADOT_COLLECTIVES_URL") {
            network.collectives.endpoint = url;
        }
        if let Ok(url) = std::env::var("POLKADOT_ASSET_HUB_URL") {
            network.asset_hub.endpoint = url;
        }

        Ok(network)
    }

    pub fn polkadot() -> Self {
        Self {
            relay: ChainConfig {
                name: "polkadot".into(),
                endpoint: "wss://rpc.polkadot.io".into(),
                token_symbol: "DOT".into(),
                token_decimals: 10,
                ss58_prefix: 0,
            },
            collectives: ChainConfig {
                name: "polkadot-collectives".into(),
                endpoint: "wss://polkadot-collectives-rpc.polkadot.io".into(),
                token_symbol: "DOT".into(),
                token_decimals: 10,
                ss58_prefix: 0,
            },
            asset_hub: ChainConfig {
                name: "polkadot-asset-hub".into(),
                endpoint: "wss://polkadot-asset-hub-rpc.polkadot.io".into(),
                token_symbol: "DOT".into(),
                token_decimals: 10,
                ss58_prefix: 0,
            },
        }
    }

    pub fn kusama() -> Self {
        Self {
            relay: ChainConfig {
                name: "kusama".into(),
                endpoint: "wss://kusama-rpc.polkadot.io".into(),
                token_symbol: "KSM".into(),
                token_decimals: 12,
                ss58_prefix: 2,
            },
            collectives: ChainConfig {
                name: "kusama-collectives".into(),
                endpoint: "wss://kusama-collectives-rpc.polkadot.io".into(),
                token_symbol: "KSM".into(),
                token_decimals: 12,
                ss58_prefix: 2,
            },
            asset_hub: ChainConfig {
                name: "kusama-asset-hub".into(),
                endpoint: "wss://kusama-asset-hub-rpc.polkadot.io".into(),
                token_symbol: "KSM".into(),
                token_decimals: 12,
                ss58_prefix: 2,
            },
        }
    }

    pub fn westend() -> Self {
        Self {
            relay: ChainConfig {
                name: "westend".into(),
                endpoint: "wss://westend-rpc.polkadot.io".into(),
                token_symbol: "WND".into(),
                token_decimals: 12,
                ss58_prefix: 42,
            },
            collectives: ChainConfig {
                name: "westend-collectives".into(),
                endpoint: "wss://westend-collectives-rpc.polkadot.io".into(),
                token_symbol: "WND".into(),
                token_decimals: 12,
                ss58_prefix: 42,
            },
            asset_hub: ChainConfig {
                name: "westend-asset-hub".into(),
                endpoint: "wss://westend-asset-hub-rpc.polkadot.io".into(),
                token_symbol: "WND".into(),
                token_decimals: 12,
                ss58_prefix: 42,
            },
        }
    }
}
