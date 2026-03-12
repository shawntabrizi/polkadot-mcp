use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ServerCapabilities, ServerInfo};
use rmcp::{ErrorData, ServerHandler, tool, tool_handler, tool_router};

use crate::network::{ChainConfig, Network};
use crate::pool::ChainPool;
use crate::tools::{account, chain};

/// The MCP server. Holds all network configs and a shared connection pool.
#[derive(Clone)]
pub struct PolkadotMcp {
    pub networks: Arc<HashMap<String, Network>>,
    pub pool: Arc<ChainPool>,
    pub signer: Option<Arc<subxt_signer::sr25519::Keypair>>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl PolkadotMcp {
    #[tool(description = "Get chain information: network name, token symbol, decimals, \
        current block number, and runtime version. \
        Use 'network' param for: 'polkadot' (default), 'kusama', 'westend', 'paseo'. \
        Use 'chain' param for: 'relay' (default), 'asset-hub', 'bridge-hub', 'people', 'collectives', 'coretime'.")]
    async fn chain_info(
        &self,
        Parameters(params): Parameters<chain::ChainInfoParams>,
    ) -> Result<CallToolResult, ErrorData> {
        chain::chain_info(self, params)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))
    }

    #[tool(description = "Get account balances: free, reserved, frozen, and transferable \
        balance for an SS58 address. \
        Use 'network' param for: 'polkadot' (default), 'kusama', 'westend', 'paseo'. \
        Use 'chain' param for: 'relay' (default), 'asset-hub', 'bridge-hub', 'people', 'collectives', 'coretime'.")]
    async fn get_balances(
        &self,
        Parameters(params): Parameters<account::GetBalancesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        account::get_balances(self, params)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))
    }
}

impl PolkadotMcp {
    pub fn new(signer: Option<subxt_signer::sr25519::Keypair>) -> Self {
        let mut networks = HashMap::new();
        networks.insert("polkadot".to_string(), Network::polkadot());
        networks.insert("kusama".to_string(), Network::kusama());
        networks.insert("westend".to_string(), Network::westend());
        networks.insert("paseo".to_string(), Network::paseo());

        Self {
            networks: Arc::new(networks),
            pool: ChainPool::new(),
            signer: signer.map(Arc::new),
            tool_router: Self::tool_router(),
        }
    }

    /// Resolve a (network, chain) pair to the chain config.
    /// network defaults to "polkadot", chain defaults to "relay".
    pub fn resolve(&self, network: &str, chain: &str) -> Result<ChainConfig> {
        let network_name = if network.is_empty() { "polkadot" } else { network };
        let net = self.networks.get(network_name).ok_or_else(|| {
            let available: Vec<&str> = self.networks.keys().map(|s| s.as_str()).collect();
            anyhow!(
                "Unknown network '{}'. Available: {}",
                network_name,
                available.join(", ")
            )
        })?;
        let (_name, config) = net.resolve_chain(chain)?;
        Ok(config.clone())
    }
}

#[tool_handler(router = "self.tool_router")]
impl ServerHandler for PolkadotMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Polkadot MCP server. Query any Polkadot ecosystem chain: \
                 Polkadot, Kusama, Westend, Paseo — plus their system parachains \
                 (Asset Hub, Bridge Hub, People, Collectives, Coretime). \
                 Use the 'network' parameter to select the network and 'chain' \
                 to select a specific parachain."
                    .to_string(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
