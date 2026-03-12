use anyhow::Result;
use rmcp::model::CallToolResult;
use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

use crate::backends::subxt_backend;
use crate::server::PolkadotMcp;
use crate::types::{error_result, text_result};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ChainInfoParams {
    /// Which chain to query: 'relay' (default), 'asset-hub', 'bridge-hub', 'people', 'collectives', or 'coretime'.
    #[serde(default = "default_chain")]
    pub chain: String,
}

fn default_chain() -> String {
    "relay".to_string()
}

pub async fn chain_info(server: &PolkadotMcp, params: ChainInfoParams) -> Result<CallToolResult> {
    let (chain_name, config) = match server.pool.network.resolve_chain(&params.chain) {
        Ok((name, config)) => (name.to_string(), config.clone()),
        Err(e) => return Ok(error_result(&e.to_string())),
    };

    let api = match server.pool.get(&chain_name).await {
        Ok(api) => api,
        Err(e) => {
            return Ok(error_result(&format!(
                "Failed to connect to {}: {}",
                chain_name, e
            )));
        }
    };

    let block_number = subxt_backend::current_block_number(&api).await?;
    let runtime = api.runtime_version();

    let mut output = String::new();
    output.push_str(&format!("Chain: {}\n", chain_name));
    output.push_str(&format!("Network: {}\n", server.pool.network.name()));
    output.push_str(&format!("Token: {} ({} decimals)\n", config.token_symbol, config.token_decimals));
    output.push_str(&format!("SS58 Prefix: {}\n", config.ss58_prefix));
    output.push_str(&format!("Current Block: #{}\n", block_number));
    output.push_str(&format!("Spec Version: {}\n", runtime.spec_version));
    output.push_str(&format!("Transaction Version: {}", runtime.transaction_version));

    Ok(text_result(&output))
}
