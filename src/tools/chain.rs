use anyhow::Result;
use rmcp::model::CallToolResult;
use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

use crate::backends::subxt_backend;
use crate::server::PolkadotMcp;
use crate::types::{error_result, text_result};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ChainInfoParams {
    /// Network to query: 'polkadot' (default), 'kusama', 'westend', or 'paseo'.
    #[serde(default = "default_network")]
    pub network: String,
    /// Chain within the network: 'relay' (default), 'asset-hub', 'bridge-hub', 'people', 'collectives', or 'coretime'.
    #[serde(default = "default_chain")]
    pub chain: String,
}

fn default_network() -> String {
    "polkadot".to_string()
}

fn default_chain() -> String {
    "relay".to_string()
}

pub async fn chain_info(server: &PolkadotMcp, params: ChainInfoParams) -> Result<CallToolResult> {
    let config = match server.resolve(&params.network, &params.chain) {
        Ok(config) => config,
        Err(e) => return Ok(error_result(&e.to_string())),
    };

    let api = match server.pool.get(&config).await {
        Ok(api) => api,
        Err(e) => {
            return Ok(error_result(&format!(
                "Failed to connect to {}: {}",
                config.name, e
            )));
        }
    };

    let block_number = subxt_backend::current_block_number(&api).await?;
    let runtime = api.runtime_version();

    let network_name = if params.network.is_empty() {
        "polkadot"
    } else {
        &params.network
    };

    let chain_type = if params.chain == "relay" || params.chain.is_empty() {
        "Relay Chain"
    } else {
        "System Parachain"
    };

    let mut output = String::new();
    output.push_str(&format!("Chain: {}\n", config.name));
    output.push_str(&format!("Type: {}\n", chain_type));
    output.push_str(&format!("Network: {}\n", network_name));
    output.push_str(&format!("Token: {} ({} decimals)\n", config.token_symbol, config.token_decimals));
    output.push_str(&format!("SS58 Prefix: {}\n", config.ss58_prefix));
    output.push_str(&format!("Current Block: #{}\n", block_number));
    output.push_str(&format!("Spec Version: {}\n", runtime.spec_version));
    output.push_str(&format!("Transaction Version: {}", runtime.transaction_version));

    Ok(text_result(&output))
}
