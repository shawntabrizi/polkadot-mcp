#![allow(dead_code)]
//! subxt backend: live on-chain state queries via subxt dynamic mode.
//!
//! This module provides helper functions for common subxt query patterns.
//! Tools call these instead of constructing raw dynamic queries.

use anyhow::Result;
use subxt::dynamic::Value;
use subxt::{OnlineClient, PolkadotConfig};

/// Fetch a single storage value by pallet, entry, and key.
pub async fn fetch_storage(
    api: &OnlineClient<PolkadotConfig>,
    pallet: &str,
    entry: &str,
    keys: Vec<Value>,
) -> Result<Option<subxt::dynamic::DecodedValue>> {
    let addr = subxt::dynamic::storage(pallet, entry, keys);
    let storage = api.storage().at_latest().await?;
    let result = storage.fetch(&addr).await?;
    match result {
        Some(val) => Ok(Some(val.to_value()?)),
        None => Ok(None),
    }
}

/// Fetch a storage value, returning an error if not found.
pub async fn fetch_storage_required(
    api: &OnlineClient<PolkadotConfig>,
    pallet: &str,
    entry: &str,
    keys: Vec<Value>,
) -> Result<subxt::dynamic::DecodedValue> {
    fetch_storage(api, pallet, entry, keys)
        .await?
        .ok_or_else(|| anyhow::anyhow!("{}.{}: not found", pallet, entry))
}

/// Fetch the current block number.
pub async fn current_block_number(api: &OnlineClient<PolkadotConfig>) -> Result<u64> {
    let block = api.blocks().at_latest().await?;
    Ok(block.number().into())
}

/// Build an AccountId Value from raw 32-byte account ID.
pub fn account_value(account: &[u8; 32]) -> Value {
    Value::from_bytes(account)
}

/// Check if a pallet exists in the chain's metadata.
pub async fn pallet_exists(api: &OnlineClient<PolkadotConfig>, pallet_name: &str) -> bool {
    api.metadata()
        .pallet_by_name(pallet_name)
        .is_some()
}

/// Iterate all entries of a storage map. Returns decoded values.
/// Use `partial_keys` to iterate a subset (e.g. all stakers in an era).
/// Pass empty vec to iterate all entries.
/// Limited to `limit` entries (default 1000) to prevent OOM on large maps.
pub async fn fetch_storage_iter(
    api: &OnlineClient<PolkadotConfig>,
    pallet: &str,
    entry: &str,
    partial_keys: Vec<Value>,
    limit: Option<usize>,
) -> Result<Vec<subxt::dynamic::DecodedValue>> {
    let addr = subxt::dynamic::storage(pallet, entry, partial_keys);
    let storage = api.storage().at_latest().await?;
    let mut iter = storage.iter(addr).await?;
    let mut results = Vec::new();
    let max = limit.unwrap_or(1000);
    while let Some(kv) = iter.next().await {
        let kv = kv?;
        results.push(kv.value.to_value()?);
        if results.len() >= max {
            break;
        }
    }
    Ok(results)
}
