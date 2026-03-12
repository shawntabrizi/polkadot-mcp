use anyhow::Result;
use rmcp::model::CallToolResult;
use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

use crate::server::PolkadotMcp;
use crate::types::{error_result, text_result};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListPalletsParams {
    /// Network to query: 'polkadot' (default), 'kusama', 'westend', or 'paseo'.
    #[serde(default = "default_network")]
    pub network: String,
    /// Chain within the network: 'relay' (default), 'asset-hub', 'bridge-hub', 'people', 'collectives', or 'coretime'.
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PalletInfoParams {
    /// Name of the pallet to inspect (e.g. "Staking", "Balances", "System").
    pub pallet_name: String,
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

pub async fn list_pallets(
    server: &PolkadotMcp,
    params: ListPalletsParams,
) -> Result<CallToolResult> {
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

    let metadata = api.metadata();
    let mut pallets: Vec<_> = metadata.pallets().collect();
    pallets.sort_by_key(|p| p.index());

    let mut output = String::new();
    output.push_str(&format!(
        "Pallets on {} ({} total):\n\n",
        config.name,
        pallets.len()
    ));
    output.push_str(&format!(
        "{:<4} {:<30} {:>6} {:>8} {:>7} {:>7} {:>6}\n",
        "Idx", "Pallet", "Calls", "Storage", "Events", "Errors", "Const"
    ));
    output.push_str(&format!("{}\n", "-".repeat(72)));

    for pallet in &pallets {
        let n_calls = pallet
            .call_variants()
            .map(|v| v.len())
            .unwrap_or(0);
        let n_storage = pallet
            .storage()
            .map(|s| s.entries().len())
            .unwrap_or(0);
        let n_events = pallet
            .event_variants()
            .map(|v| v.len())
            .unwrap_or(0);
        let n_errors = pallet
            .error_variants()
            .map(|v| v.len())
            .unwrap_or(0);
        let n_constants = pallet.constants().count();

        output.push_str(&format!(
            "{:<4} {:<30} {:>6} {:>8} {:>7} {:>7} {:>6}\n",
            pallet.index(),
            pallet.name(),
            n_calls,
            n_storage,
            n_events,
            n_errors,
            n_constants
        ));
    }

    Ok(text_result(&output))
}

pub async fn pallet_info(
    server: &PolkadotMcp,
    params: PalletInfoParams,
) -> Result<CallToolResult> {
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

    let metadata = api.metadata();
    let pallet = match metadata.pallet_by_name(&params.pallet_name) {
        Some(p) => p,
        None => {
            // Try case-insensitive match
            match metadata
                .pallets()
                .find(|p| p.name().eq_ignore_ascii_case(&params.pallet_name))
            {
                Some(p) => p,
                None => {
                    return Ok(error_result(&format!(
                        "Pallet '{}' not found on {}. Use list_pallets to see available pallets.",
                        params.pallet_name, config.name
                    )));
                }
            }
        }
    };

    let mut output = String::new();
    output.push_str(&format!(
        "Pallet: {} (index {})\n",
        pallet.name(),
        pallet.index()
    ));
    output.push_str(&format!("Chain: {}\n", config.name));

    // Documentation
    let docs = pallet.docs();
    if !docs.is_empty() {
        output.push_str("\nDocumentation:\n");
        for (i, line) in docs.iter().enumerate() {
            if i >= 5 {
                output.push_str("  ...(truncated)\n");
                break;
            }
            output.push_str(&format!("  {}\n", line));
        }
    }

    // Calls
    if let Some(calls) = pallet.call_variants() {
        output.push_str(&format!("\nCalls ({}):\n", calls.len()));
        for call in calls {
            let param_names: Vec<&str> = call
                .fields
                .iter()
                .filter_map(|f| f.name.as_deref())
                .collect();
            if param_names.is_empty() {
                output.push_str(&format!("  {}()\n", call.name));
            } else {
                output.push_str(&format!("  {}({})\n", call.name, param_names.join(", ")));
            }
        }
    }

    // Storage
    if let Some(storage) = pallet.storage() {
        let entries = storage.entries();
        output.push_str(&format!("\nStorage ({}):\n", entries.len()));
        for entry in entries {
            output.push_str(&format!("  {}\n", entry.name()));
        }
    }

    // Events
    if let Some(events) = pallet.event_variants() {
        output.push_str(&format!("\nEvents ({}):\n", events.len()));
        for event in events {
            output.push_str(&format!("  {}\n", event.name));
        }
    }

    // Errors
    if let Some(errors) = pallet.error_variants() {
        output.push_str(&format!("\nErrors ({}):\n", errors.len()));
        for error in errors {
            output.push_str(&format!("  {}\n", error.name));
        }
    }

    // Constants
    let constants: Vec<_> = pallet.constants().collect();
    if !constants.is_empty() {
        output.push_str(&format!("\nConstants ({}):\n", constants.len()));
        for constant in &constants {
            output.push_str(&format!("  {}\n", constant.name()));
        }
    }

    Ok(text_result(&output))
}
