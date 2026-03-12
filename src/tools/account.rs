use anyhow::Result;
use rmcp::model::CallToolResult;
use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;
use subxt::dynamic::At;
use subxt::ext::scale_value::{Composite, ValueDef};

use crate::backends::subxt_backend;
use crate::decode::{self, value_as_u128};
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

// ---------------------------------------------------------------------------
// account_locks
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AccountLocksParams {
    /// SS58 address to query.
    pub address: String,
    /// Network to query: 'polkadot' (default), 'kusama', 'westend', or 'paseo'.
    #[serde(default = "default_network")]
    pub network: String,
    /// Chain within the network: 'relay' (default), 'asset-hub', 'bridge-hub', 'people', 'collectives', or 'coretime'.
    #[serde(default = "default_chain")]
    pub chain: String,
}

pub async fn account_locks(
    server: &PolkadotMcp,
    params: AccountLocksParams,
) -> Result<CallToolResult> {
    let account_id = match parse_ss58(&params.address) {
        Ok(id) => id,
        Err(e) => return Ok(error_result(&format!("Invalid address: {}", e))),
    };

    let config = match server.resolve(&params.network, &params.chain) {
        Ok(c) => c,
        Err(e) => return Ok(error_result(&e.to_string())),
    };

    let api = match server.pool.get(&config).await {
        Ok(a) => a,
        Err(e) => {
            return Ok(error_result(&format!(
                "Failed to connect to {}: {}",
                config.name, e
            )));
        }
    };

    let sym = &config.token_symbol;
    let dec = config.token_decimals;

    let mut output = String::new();
    output.push_str(&format!("Account: {}\n", params.address));
    output.push_str(&format!("Chain: {}\n", config.name));

    // Fetch Locks
    let locks = subxt_backend::fetch_storage(
        &api,
        "Balances",
        "Locks",
        vec![subxt_backend::account_value(&account_id)],
    )
    .await?;

    if let Some(locks_value) = locks {
        let items = extract_vec_items(&locks_value);
        if items.is_empty() {
            output.push_str("\nLocks: none\n");
        } else {
            output.push_str(&format!("\nLocks ({}):\n", items.len()));
            for item in &items {
                let id = item
                    .at("id")
                    .map(decode::decode_lock_id)
                    .unwrap_or_else(|| "?".to_string());
                let amount = item.at("amount").map(value_as_u128).unwrap_or(0);
                let reasons = item
                    .at("reasons")
                    .map(decode::format_value)
                    .unwrap_or_else(|| "?".to_string());
                output.push_str(&format!(
                    "  {} — {} ({})\n",
                    id,
                    format_balance(amount, dec, sym),
                    reasons
                ));
            }
        }
    } else {
        output.push_str("\nLocks: none\n");
    }

    // Fetch Freezes
    let freezes = subxt_backend::fetch_storage(
        &api,
        "Balances",
        "Freezes",
        vec![subxt_backend::account_value(&account_id)],
    )
    .await?;

    if let Some(freezes_value) = freezes {
        let items = extract_vec_items(&freezes_value);
        if items.is_empty() {
            output.push_str("\nFreezes: none\n");
        } else {
            output.push_str(&format!("\nFreezes ({}):\n", items.len()));
            for item in &items {
                let id = item
                    .at("id")
                    .map(decode::format_value)
                    .unwrap_or_else(|| "?".to_string());
                let amount = item.at("amount").map(value_as_u128).unwrap_or(0);
                output.push_str(&format!(
                    "  {} — {}\n",
                    id,
                    format_balance(amount, dec, sym)
                ));
            }
        }
    } else {
        output.push_str("\nFreezes: none\n");
    }

    // Fetch Holds
    let holds = subxt_backend::fetch_storage(
        &api,
        "Balances",
        "Holds",
        vec![subxt_backend::account_value(&account_id)],
    )
    .await?;

    if let Some(holds_value) = holds {
        let items = extract_vec_items(&holds_value);
        if items.is_empty() {
            output.push_str("\nHolds: none");
        } else {
            output.push_str(&format!("\nHolds ({}):\n", items.len()));
            for item in &items {
                let id = item
                    .at("id")
                    .map(decode::format_value)
                    .unwrap_or_else(|| "?".to_string());
                let amount = item.at("amount").map(value_as_u128).unwrap_or(0);
                output.push_str(&format!(
                    "  {} — {}\n",
                    id,
                    format_balance(amount, dec, sym)
                ));
            }
        }
    } else {
        output.push_str("\nHolds: none");
    }

    Ok(text_result(&output))
}

/// Extract items from a Vec-like dynamic value (Composite::Unnamed).
fn extract_vec_items(
    value: &subxt::dynamic::DecodedValue,
) -> Vec<&subxt::dynamic::DecodedValue> {
    match &value.value {
        ValueDef::Composite(Composite::Unnamed(items)) => items.iter().collect(),
        _ => vec![],
    }
}
