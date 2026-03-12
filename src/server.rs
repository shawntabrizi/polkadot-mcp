use std::sync::Arc;

use rmcp::model::ServerInfo;
use rmcp::{ServerHandler, tool_router};

use crate::network::Network;
use crate::pool::ChainPool;

/// The MCP server. Holds the chain connection pool and optional signer.
/// Tools are defined in the `tools/` modules and registered via `#[tool_box]`.
#[derive(Clone)]
pub struct PolkadotMcp {
    pub pool: Arc<ChainPool>,
    pub signer: Option<Arc<subxt_signer::sr25519::Keypair>>,
}

#[tool_router]
impl PolkadotMcp {
    // Tools are implemented in tools/*.rs and called via #[tool] attribute.
    // The #[tool_box] macro on this impl block collects all #[tool] methods
    // and registers them with the MCP server.
    //
    // Tool methods are defined in:
    //   - tools/chain.rs     (chain_info, query_storage, ...)
    //   - tools/account.rs   (account_info, account_balances, ...)
    //   - tools/fellowship.rs (fellowship_status, ...)
    //   - tools/governance.rs (referenda_active, vote, ...)
    //   - tools/staking.rs   (staking_status, ...)
    //
    // TODO: Register tool methods here as they are implemented.
}

impl PolkadotMcp {
    pub fn new(
        network: Network,
        signer: Option<subxt_signer::sr25519::Keypair>,
    ) -> Self {
        Self {
            pool: ChainPool::new(network),
            signer: signer.map(Arc::new),
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
            ..Default::default()
        }
    }
}
