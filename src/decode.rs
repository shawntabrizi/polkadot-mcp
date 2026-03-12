#![allow(dead_code)]
//! Dynamic value decoding: convert subxt's DecodedValueThunk into human-readable output.
//!
//! This is the hardest module in the codebase. subxt dynamic mode returns
//! `scale_value::Value` types that need to be navigated and formatted
//! contextually (balances as DOT, block numbers as time estimates, etc.).

use subxt::dynamic::{At, DecodedValue};
use subxt::ext::scale_value::{Composite, ValueDef};

use crate::network::ChainConfig;
use crate::types::format_balance;

/// Extract a u128 from a dynamic value, returning 0 if not found.
pub fn value_as_u128(value: &DecodedValue) -> u128 {
    value.as_u128().unwrap_or(0)
}

/// Extract a string from a dynamic value.
pub fn value_as_string(value: &DecodedValue) -> Option<String> {
    value.as_str().map(|s| s.to_string())
}

/// Check if a dynamic value is a specific enum variant.
pub fn is_variant(value: &DecodedValue, variant_name: &str) -> bool {
    match &value.value {
        ValueDef::Variant(v) => v.name == variant_name,
        _ => false,
    }
}

/// Get the variant name and fields from a dynamic value.
pub fn as_variant(value: &DecodedValue) -> Option<(&str, &Composite<u32>)> {
    match &value.value {
        ValueDef::Variant(v) => Some((&v.name, &v.values)),
        _ => None,
    }
}

/// Format a balance field from a dynamic value using chain config.
pub fn format_balance_field(
    value: &DecodedValue,
    field_name: &str,
    config: &ChainConfig,
) -> String {
    let planck = value
        .at(field_name)
        .map(value_as_u128)
        .unwrap_or(0);
    format_balance(planck, config.token_decimals, &config.token_symbol)
}

/// Decode lock ID bytes into a human-readable name.
/// Lock IDs are 8-byte arrays that often contain ASCII text.
pub fn decode_lock_id(id_value: &DecodedValue) -> String {
    // Lock IDs are typically [u8; 8] with ASCII names like "staking " or "pyconvot"
    // Try to extract as bytes and convert to string
    // TODO: Implement proper byte array extraction from dynamic value
    format!("{:?}", id_value)
}

/// Map common lock ID strings to human-readable names.
pub fn lock_id_to_name(raw: &str) -> &str {
    match raw.trim() {
        "staking" | "staking " => "Staking",
        "pyconvot" => "Governance (conviction voting)",
        "vesting " | "vesting" => "Vesting",
        "democrac" => "Democracy (legacy)",
        "phrelect" => "Phragmen election",
        other => other,
    }
}
