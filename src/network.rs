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
    pub asset_hub: ChainConfig,
    pub bridge_hub: ChainConfig,
    pub people: ChainConfig,
    pub collectives: Option<ChainConfig>,
    pub coretime: ChainConfig,
}

impl Network {
    /// Get network name (relay chain name).
    pub fn name(&self) -> &str {
        &self.relay.name
    }

    /// All chains in this network.
    fn all_chains(&self) -> Vec<&ChainConfig> {
        let mut chains = vec![
            &self.relay,
            &self.asset_hub,
            &self.bridge_hub,
            &self.people,
            &self.coretime,
        ];
        if let Some(ref c) = self.collectives {
            chains.push(c);
        }
        chains
    }

    /// Look up a chain config by name.
    pub fn config_for(&self, chain_name: &str) -> Result<&ChainConfig> {
        self.all_chains()
            .into_iter()
            .find(|c| c.name == chain_name)
            .ok_or_else(|| {
                let names: Vec<&str> = self.all_chains().iter().map(|c| c.name.as_str()).collect();
                anyhow!("Unknown chain '{}'. Available: {}", chain_name, names.join(", "))
            })
    }

    /// Resolve a short chain alias (used by tools) to the full chain name.
    /// Accepts: "relay", "asset-hub", "bridge-hub", "people", "collectives", "coretime",
    /// or the full chain name (e.g. "polkadot-asset-hub").
    pub fn resolve_chain(&self, alias: &str) -> Result<(&str, &ChainConfig)> {
        let config = match alias {
            "relay" | "" => &self.relay,
            "asset-hub" => &self.asset_hub,
            "bridge-hub" => &self.bridge_hub,
            "people" => &self.people,
            "collectives" => {
                return self
                    .collectives
                    .as_ref()
                    .map(|c| (c.name.as_str(), c))
                    .ok_or_else(|| anyhow!("Collectives chain is not available on this network"));
            }
            "coretime" => &self.coretime,
            full_name => {
                return self
                    .config_for(full_name)
                    .map(|c| (c.name.as_str(), c));
            }
        };
        Ok((&config.name, config))
    }

    /// All chain names in this network.
    pub fn chain_names(&self) -> Vec<&str> {
        self.all_chains().iter().map(|c| c.name.as_str()).collect()
    }

    /// Load network from environment variables.
    ///
    /// POLKADOT_NETWORK: "polkadot" (default), "kusama", "westend", "paseo"
    /// Override individual endpoints:
    ///   POLKADOT_RELAY_URL, POLKADOT_ASSET_HUB_URL, POLKADOT_BRIDGE_HUB_URL,
    ///   POLKADOT_PEOPLE_URL, POLKADOT_COLLECTIVES_URL, POLKADOT_CORETIME_URL
    pub fn from_env() -> Result<Self> {
        let network_name = std::env::var("POLKADOT_NETWORK")
            .unwrap_or_else(|_| "polkadot".to_string());

        let mut network = match network_name.as_str() {
            "polkadot" => Self::polkadot(),
            "kusama" => Self::kusama(),
            "westend" => Self::westend(),
            "paseo" => Self::paseo(),
            other => return Err(anyhow!(
                "Unknown network '{}'. Use: polkadot, kusama, westend, paseo", other
            )),
        };

        // Allow endpoint overrides
        if let Ok(url) = std::env::var("POLKADOT_RELAY_URL") {
            network.relay.endpoint = url;
        }
        if let Ok(url) = std::env::var("POLKADOT_ASSET_HUB_URL") {
            network.asset_hub.endpoint = url;
        }
        if let Ok(url) = std::env::var("POLKADOT_BRIDGE_HUB_URL") {
            network.bridge_hub.endpoint = url;
        }
        if let Ok(url) = std::env::var("POLKADOT_PEOPLE_URL") {
            network.people.endpoint = url;
        }
        if let Ok(url) = std::env::var("POLKADOT_COLLECTIVES_URL") {
            if let Some(ref mut c) = network.collectives {
                c.endpoint = url;
            }
        }
        if let Ok(url) = std::env::var("POLKADOT_CORETIME_URL") {
            network.coretime.endpoint = url;
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
            asset_hub: ChainConfig {
                name: "polkadot-asset-hub".into(),
                endpoint: "wss://polkadot-asset-hub-rpc.polkadot.io".into(),
                token_symbol: "DOT".into(),
                token_decimals: 10,
                ss58_prefix: 0,
            },
            bridge_hub: ChainConfig {
                name: "polkadot-bridge-hub".into(),
                endpoint: "wss://polkadot-bridge-hub-rpc.polkadot.io".into(),
                token_symbol: "DOT".into(),
                token_decimals: 10,
                ss58_prefix: 0,
            },
            people: ChainConfig {
                name: "polkadot-people".into(),
                endpoint: "wss://polkadot-people-rpc.polkadot.io".into(),
                token_symbol: "DOT".into(),
                token_decimals: 10,
                ss58_prefix: 0,
            },
            collectives: Some(ChainConfig {
                name: "polkadot-collectives".into(),
                endpoint: "wss://polkadot-collectives-rpc.polkadot.io".into(),
                token_symbol: "DOT".into(),
                token_decimals: 10,
                ss58_prefix: 0,
            }),
            coretime: ChainConfig {
                name: "polkadot-coretime".into(),
                endpoint: "wss://polkadot-coretime-rpc.polkadot.io".into(),
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
            asset_hub: ChainConfig {
                name: "kusama-asset-hub".into(),
                endpoint: "wss://kusama-asset-hub-rpc.polkadot.io".into(),
                token_symbol: "KSM".into(),
                token_decimals: 12,
                ss58_prefix: 2,
            },
            bridge_hub: ChainConfig {
                name: "kusama-bridge-hub".into(),
                endpoint: "wss://kusama-bridge-hub-rpc.polkadot.io".into(),
                token_symbol: "KSM".into(),
                token_decimals: 12,
                ss58_prefix: 2,
            },
            people: ChainConfig {
                name: "kusama-people".into(),
                endpoint: "wss://kusama-people-rpc.polkadot.io".into(),
                token_symbol: "KSM".into(),
                token_decimals: 12,
                ss58_prefix: 2,
            },
            collectives: None,
            coretime: ChainConfig {
                name: "kusama-coretime".into(),
                endpoint: "wss://kusama-coretime-rpc.polkadot.io".into(),
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
            asset_hub: ChainConfig {
                name: "westend-asset-hub".into(),
                endpoint: "wss://westend-asset-hub-rpc.polkadot.io".into(),
                token_symbol: "WND".into(),
                token_decimals: 12,
                ss58_prefix: 42,
            },
            bridge_hub: ChainConfig {
                name: "westend-bridge-hub".into(),
                endpoint: "wss://westend-bridge-hub-rpc.polkadot.io".into(),
                token_symbol: "WND".into(),
                token_decimals: 12,
                ss58_prefix: 42,
            },
            people: ChainConfig {
                name: "westend-people".into(),
                endpoint: "wss://westend-people-rpc.polkadot.io".into(),
                token_symbol: "WND".into(),
                token_decimals: 12,
                ss58_prefix: 42,
            },
            collectives: Some(ChainConfig {
                name: "westend-collectives".into(),
                endpoint: "wss://westend-collectives-rpc.polkadot.io".into(),
                token_symbol: "WND".into(),
                token_decimals: 12,
                ss58_prefix: 42,
            }),
            coretime: ChainConfig {
                name: "westend-coretime".into(),
                endpoint: "wss://westend-coretime-rpc.polkadot.io".into(),
                token_symbol: "WND".into(),
                token_decimals: 12,
                ss58_prefix: 42,
            },
        }
    }

    pub fn paseo() -> Self {
        Self {
            relay: ChainConfig {
                name: "paseo".into(),
                endpoint: "wss://paseo.ibp.network".into(),
                token_symbol: "PAS".into(),
                token_decimals: 10,
                ss58_prefix: 42,
            },
            asset_hub: ChainConfig {
                name: "paseo-asset-hub".into(),
                endpoint: "wss://asset-hub-paseo.ibp.network".into(),
                token_symbol: "PAS".into(),
                token_decimals: 10,
                ss58_prefix: 42,
            },
            bridge_hub: ChainConfig {
                name: "paseo-bridge-hub".into(),
                endpoint: "wss://bridge-hub-paseo.ibp.network".into(),
                token_symbol: "PAS".into(),
                token_decimals: 10,
                ss58_prefix: 42,
            },
            people: ChainConfig {
                name: "paseo-people".into(),
                endpoint: "wss://people-paseo.ibp.network".into(),
                token_symbol: "PAS".into(),
                token_decimals: 10,
                ss58_prefix: 42,
            },
            collectives: Some(ChainConfig {
                name: "paseo-collectives".into(),
                endpoint: "wss://collectives-paseo.ibp.network".into(),
                token_symbol: "PAS".into(),
                token_decimals: 10,
                ss58_prefix: 42,
            }),
            coretime: ChainConfig {
                name: "paseo-coretime".into(),
                endpoint: "wss://coretime-paseo.ibp.network".into(),
                token_symbol: "PAS".into(),
                token_decimals: 10,
                ss58_prefix: 42,
            },
        }
    }
}
