use std::sync::Arc;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ServerCapabilities, ServerInfo};
use rmcp::{ErrorData, ServerHandler, tool, tool_router};

use crate::network::Network;
use crate::pool::ChainPool;
use crate::tools::{account, chain};

/// The MCP server. Holds the chain connection pool and optional signer.
#[derive(Clone)]
pub struct PolkadotMcp {
    pub pool: Arc<ChainPool>,
    pub signer: Option<Arc<subxt_signer::sr25519::Keypair>>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl PolkadotMcp {
    #[tool(description = "Get chain information: network name, token symbol, decimals, \
        current block number, and runtime version. Defaults to relay chain. \
        Use 'chain' param for: 'asset-hub', 'bridge-hub', 'people', 'collectives', 'coretime'.")]
    async fn chain_info(
        &self,
        Parameters(params): Parameters<chain::ChainInfoParams>,
    ) -> Result<CallToolResult, ErrorData> {
        chain::chain_info(self, params)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))
    }

    #[tool(description = "Get account balances: free, reserved, frozen, and transferable \
        balance for an SS58 address. Defaults to relay chain. Use 'chain' param for: \
        'asset-hub', 'bridge-hub', 'people', 'collectives', 'coretime'.")]
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
    pub fn new(
        network: Network,
        signer: Option<subxt_signer::sr25519::Keypair>,
    ) -> Self {
        Self {
            pool: ChainPool::new(network),
            signer: signer.map(Arc::new),
            tool_router: Self::tool_router(),
        }
    }
}

impl ServerHandler for PolkadotMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Polkadot MCP server. Provides tools to query and interact with \
                 Polkadot, Kusama, and other Substrate-based blockchains. \
                 Tools cover account balances, fellowship status, governance \
                 (OpenGov), staking, and generic chain queries. \
                 Most tools are read-only. Transaction tools require a signer \
                 to be configured and will note this in their description."
                    .to_string(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
