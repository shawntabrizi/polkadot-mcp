use anyhow::Result;
use rmcp::model::CallToolResult;
use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

use crate::decode;
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
    let pallet = match find_pallet(&metadata, &params.pallet_name) {
        Some(p) => p,
        None => {
            return Ok(error_result(&format!(
                "Pallet '{}' not found on {}. Use list_pallets to see available pallets.",
                params.pallet_name, config.name
            )));
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

// ---------------------------------------------------------------------------
// list_storage
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListStorageParams {
    /// Name of the pallet to list storage entries for (e.g. "System", "Balances", "Staking").
    pub pallet_name: String,
    /// Network to query: 'polkadot' (default), 'kusama', 'westend', or 'paseo'.
    #[serde(default = "default_network")]
    pub network: String,
    /// Chain within the network: 'relay' (default), 'asset-hub', 'bridge-hub', 'people', 'collectives', or 'coretime'.
    #[serde(default = "default_chain")]
    pub chain: String,
}

pub async fn list_storage(
    server: &PolkadotMcp,
    params: ListStorageParams,
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
    let pallet = match find_pallet(&metadata, &params.pallet_name) {
        Some(p) => p,
        None => {
            return Ok(error_result(&format!(
                "Pallet '{}' not found on {}. Use list_pallets to see available pallets.",
                params.pallet_name, config.name
            )));
        }
    };

    let storage = match pallet.storage() {
        Some(s) => s,
        None => {
            return Ok(error_result(&format!(
                "Pallet '{}' has no storage entries on {}.",
                pallet.name(),
                config.name
            )));
        }
    };

    let entries = storage.entries();
    let types = metadata.types();

    let mut output = String::new();
    output.push_str(&format!(
        "Storage entries for {} on {} ({} entries):\n\n",
        pallet.name(),
        config.name,
        entries.len()
    ));

    for entry in entries {
        let entry_type = entry.entry_type();
        let value_type = decode::type_to_string(entry_type.value_ty(), types);
        let modifier = match entry.modifier() {
            subxt::metadata::types::StorageEntryModifier::Optional => "Optional",
            subxt::metadata::types::StorageEntryModifier::Default => "Default",
        };

        match entry_type {
            subxt::metadata::types::StorageEntryType::Plain(_) => {
                output.push_str(&format!(
                    "  {} → {}  ({})\n",
                    entry.name(),
                    value_type,
                    modifier,
                ));
            }
            subxt::metadata::types::StorageEntryType::Map {
                hashers,
                key_ty,
                ..
            } => {
                let key_type = decode::type_to_string(*key_ty, types);
                let hasher_strs: Vec<&str> = hashers.iter().map(format_hasher).collect();
                output.push_str(&format!(
                    "  {}({}) → {}  [{}]  ({})\n",
                    entry.name(),
                    key_type,
                    value_type,
                    hasher_strs.join(", "),
                    modifier,
                ));
            }
        }
    }

    Ok(text_result(&output))
}

fn format_hasher(h: &subxt::metadata::types::StorageHasher) -> &'static str {
    use subxt::metadata::types::StorageHasher;
    match h {
        StorageHasher::Blake2_128 => "Blake2_128",
        StorageHasher::Blake2_256 => "Blake2_256",
        StorageHasher::Blake2_128Concat => "Blake2_128Concat",
        StorageHasher::Twox128 => "Twox128",
        StorageHasher::Twox256 => "Twox256",
        StorageHasher::Twox64Concat => "Twox64Concat",
        StorageHasher::Identity => "Identity",
    }
}

/// Case-insensitive pallet lookup helper.
fn find_pallet<'a>(
    metadata: &'a subxt::metadata::types::Metadata,
    name: &str,
) -> Option<subxt::metadata::types::PalletMetadata<'a>> {
    metadata.pallet_by_name(name).or_else(|| {
        metadata
            .pallets()
            .find(|p| p.name().eq_ignore_ascii_case(name))
    })
}

// ---------------------------------------------------------------------------
// constant_value
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ConstantValueParams {
    /// Pallet name (e.g. "Balances", "Staking").
    pub pallet_name: String,
    /// Constant name (e.g. "ExistentialDeposit", "BondingDuration").
    pub constant_name: String,
    /// Network to query: 'polkadot' (default), 'kusama', 'westend', or 'paseo'.
    #[serde(default = "default_network")]
    pub network: String,
    /// Chain within the network: 'relay' (default), 'asset-hub', 'bridge-hub', 'people', 'collectives', or 'coretime'.
    #[serde(default = "default_chain")]
    pub chain: String,
}

pub async fn constant_value(
    server: &PolkadotMcp,
    params: ConstantValueParams,
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

    // Use subxt dynamic constant query
    let addr = subxt::dynamic::constant(&params.pallet_name, &params.constant_name);
    let value = match api.constants().at(&addr) {
        Ok(v) => v,
        Err(e) => {
            return Ok(error_result(&format!(
                "Constant '{}.{}' not found on {}: {}",
                params.pallet_name, params.constant_name, config.name, e
            )));
        }
    };

    let decoded = value.to_value()?;

    // Get type info for display
    let metadata = api.metadata();
    let type_name = metadata
        .pallet_by_name(&params.pallet_name)
        .and_then(|p| p.constant_by_name(&params.constant_name))
        .map(|c| decode::type_to_string(c.ty(), metadata.types()))
        .unwrap_or_else(|| "unknown".to_string());

    let mut output = String::new();
    output.push_str(&format!(
        "Constant: {}.{}\n",
        params.pallet_name, params.constant_name
    ));
    output.push_str(&format!("Chain: {}\n", config.name));
    output.push_str(&format!("Type: {}\n", type_name));
    output.push_str(&format!("Value: {}", decode::format_value(&decoded)));

    Ok(text_result(&output))
}
