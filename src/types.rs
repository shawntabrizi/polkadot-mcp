use anyhow::{anyhow, Result};
use rmcp::model::CallToolResult;
use rmcp::model::Content;

use crate::network::ChainConfig;

/// Format a balance from planck to human-readable with token symbol.
/// Uses the chain's decimals for correct formatting.
///
/// # Examples
/// ```
/// format_balance(15_000_000_000, 10, "DOT") // "1.5000 DOT"
/// format_balance(1_500_000_000_000, 12, "KSM") // "1.5000 KSM"
/// ```
pub fn format_balance(planck: u128, decimals: u8, symbol: &str) -> String {
    let divisor = 10u128.pow(decimals as u32);
    let whole = planck / divisor;
    let frac = planck % divisor;

    // Show 4 decimal places
    let frac_divisor = 10u128.pow(decimals.saturating_sub(4) as u32);
    let frac_display = if frac_divisor > 0 {
        frac / frac_divisor
    } else {
        frac
    };

    format!("{}.{:04} {}", whole, frac_display, symbol)
}

/// Format a balance using a ChainConfig for decimals and symbol.
#[allow(dead_code)]
pub fn format_chain_balance(planck: u128, config: &ChainConfig) -> String {
    format_balance(planck, config.token_decimals, &config.token_symbol)
}

/// Parse an SS58 address string to raw 32-byte AccountId.
///
/// Accepts any valid SS58 address regardless of prefix.
/// Returns the raw 32-byte account ID.
pub fn parse_ss58(address: &str) -> Result<[u8; 32]> {
    // SS58 is base58check encoded: [prefix_bytes | account_bytes | checksum]
    // For simplicity, we'll use a basic implementation.
    // In production, use the `sp-core` crate or `ss58-registry`.
    //
    // TODO: Replace with proper SS58 decoding (sp-core or manual base58).
    // For now, this is a placeholder that needs a real implementation.
    let decoded = bs58::decode(address)
        .into_vec()
        .map_err(|e| anyhow!("Invalid SS58 address '{}': {}", address, e))?;

    // SS58 format: [prefix(1-2 bytes)] [account(32 bytes)] [checksum(2 bytes)]
    // Simple prefix (0-63): 1 byte
    // Full prefix (64-16383): 2 bytes
    let prefix_len = if decoded.first().is_some_and(|&b| b < 64) {
        1
    } else {
        2
    };

    if decoded.len() < prefix_len + 32 + 2 {
        return Err(anyhow!("SS58 address too short: '{}'", address));
    }

    let mut account = [0u8; 32];
    account.copy_from_slice(&decoded[prefix_len..prefix_len + 32]);
    Ok(account)
}

/// Build a successful text response for an MCP tool.
pub fn text_result(text: &str) -> CallToolResult {
    CallToolResult {
        content: vec![Content::text(text.to_string())],
        is_error: Some(false),
        meta: None,
        structured_content: None,
    }
}

/// Build an error response for an MCP tool.
pub fn error_result(text: &str) -> CallToolResult {
    CallToolResult {
        content: vec![Content::text(text.to_string())],
        is_error: Some(true),
        meta: None,
        structured_content: None,
    }
}

/// Format a block number as approximate wall-clock time.
/// Assumes 6-second block times (standard for Polkadot/Kusama).
#[allow(dead_code)]
pub fn blocks_to_duration(blocks: u64) -> String {
    let seconds = blocks * 6;
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("~{}m", seconds / 60)
    } else if seconds < 86400 {
        format!("~{}h", seconds / 3600)
    } else {
        format!("~{}d", seconds / 86400)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_balance_dot() {
        assert_eq!(format_balance(15_000_000_000, 10, "DOT"), "1.5000 DOT");
        assert_eq!(format_balance(10_000_000_000, 10, "DOT"), "1.0000 DOT");
        assert_eq!(format_balance(0, 10, "DOT"), "0.0000 DOT");
        assert_eq!(
            format_balance(123_456_789_012_345, 10, "DOT"),
            "12345.6789 DOT"
        );
    }

    #[test]
    fn test_format_balance_ksm() {
        assert_eq!(
            format_balance(1_500_000_000_000, 12, "KSM"),
            "1.5000 KSM"
        );
    }

    #[test]
    fn test_blocks_to_duration() {
        assert_eq!(blocks_to_duration(1), "6s");
        assert_eq!(blocks_to_duration(10), "~1m");
        assert_eq!(blocks_to_duration(600), "~1h");
        assert_eq!(blocks_to_duration(14400), "~1d");
    }
}
