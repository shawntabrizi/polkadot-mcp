use anyhow::Result;
use std::str::FromStr;
use subxt_signer::sr25519::Keypair;
use subxt_signer::SecretUri;

/// Load a signer keypair from the POLKADOT_SIGNER_URI environment variable.
///
/// Supports:
///   - Dev accounts: "//Alice", "//Bob", etc.
///   - Seed phrases: "bottom drive obey lake curtain smoke basket hold race lonely fit walk"
///   - Seed phrases with derivation: "phrase//hard/soft"
///
/// Returns None if the env var is not set (server runs in read-only mode).
pub fn load_from_env() -> Result<Option<Keypair>> {
    let uri = match std::env::var("POLKADOT_SIGNER_URI") {
        Ok(uri) => uri,
        Err(_) => return Ok(None),
    };

    if uri.is_empty() {
        return Ok(None);
    }

    let secret_uri = SecretUri::from_str(&uri)
        .map_err(|e| anyhow::anyhow!("Failed to parse POLKADOT_SIGNER_URI: {}", e))?;
    let keypair = Keypair::from_uri(&secret_uri)
        .map_err(|e| anyhow::anyhow!("Failed to create keypair from POLKADOT_SIGNER_URI: {}", e))?;

    Ok(Some(keypair))
}
