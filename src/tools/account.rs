use anyhow::Result;
use rmcp::model::CallToolResult;
use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;
use subxt::dynamic::At;

use crate::backends::subxt_backend;
use crate::decode::value_as_u128;
use crate::server::PolkadotMcp;
use crate::types::{error_result, format_balance, parse_ss58, text_result};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetBalancesParams {
    /// SS58 address to query (e.g. "15oF4uVJwmo4TdGW7VfQxNLavjCXviqWrztPu9T1PLww5M9Q").
    pub address: String,
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

pub async fn get_balances(
    server: &PolkadotMcp,
    params: GetBalancesParams,
) -> Result<CallToolResult> {
    let account_id = match parse_ss58(&params.address) {
        Ok(id) => id,
        Err(e) => return Ok(error_result(&format!("Invalid address: {}", e))),
    };

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

    let account_value = subxt_backend::account_value(&account_id);
    let result = subxt_backend::fetch_storage(
        &api,
        "System",
        "Account",
        vec![account_value],
    )
    .await?;

    let Some(account_data) = result else {
        return Ok(text_result(&format!(
            "Account {} not found on {}. The account may not exist or has no balance.",
            params.address, config.name
        )));
    };

    // System.Account returns { nonce, consumers, providers, sufficients, data: { free, reserved, frozen, flags } }
    let nonce = account_data.at("nonce").map(value_as_u128).unwrap_or(0);

    let data = account_data.at("data");
    let free = data.and_then(|d| d.at("free")).map(value_as_u128).unwrap_or(0);
    let reserved = data.and_then(|d| d.at("reserved")).map(value_as_u128).unwrap_or(0);
    let frozen = data.and_then(|d| d.at("frozen")).map(value_as_u128).unwrap_or(0);

    // Transferable = free - frozen (but never negative)
    let transferable = free.saturating_sub(frozen);
    let total = free + reserved;

    let sym = &config.token_symbol;
    let dec = config.token_decimals;

    let mut output = String::new();
    output.push_str(&format!("Account: {}\n", params.address));
    output.push_str(&format!("Chain: {}\n", config.name));
    output.push_str(&format!("Nonce: {}\n", nonce));
    output.push_str("\nBalances:\n");
    output.push_str(&format!("  Total:        {}\n", format_balance(total, dec, sym)));
    output.push_str(&format!("  Free:         {}\n", format_balance(free, dec, sym)));
    output.push_str(&format!("  Reserved:     {}\n", format_balance(reserved, dec, sym)));
    output.push_str(&format!("  Frozen:       {}\n", format_balance(frozen, dec, sym)));
    output.push_str(&format!("  Transferable: {}", format_balance(transferable, dec, sym)));

    Ok(text_result(&output))
}
