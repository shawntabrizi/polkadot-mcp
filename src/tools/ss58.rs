use anyhow::Result;
use rmcp::model::CallToolResult;
use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;
use sp_core::crypto::{AccountId32, Ss58AddressFormat, Ss58Codec};

use crate::types::{error_result, text_result};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct Ss58DecodeParams {
    /// SS58 address to decode (e.g. "15oF4uVJwmo4TdGW7VfQxNLavjCXviqWrztPu9T1PLww5M9Q").
    pub address: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct Ss58EncodeParams {
    /// Raw 32-byte public key as hex string (with or without 0x prefix).
    pub public_key_hex: String,
    /// SS58 prefix (0=Polkadot, 2=Kusama, 42=Generic/Westend/Paseo).
    #[serde(default = "default_prefix")]
    pub prefix: u16,
}

fn default_prefix() -> u16 {
    42
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct Ss58ConvertParams {
    /// SS58 address to convert.
    pub address: String,
    /// Target SS58 prefix (0=Polkadot, 2=Kusama, 42=Generic/Westend/Paseo).
    pub target_prefix: u16,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct Ss58ValidateParams {
    /// SS58 address to validate.
    pub address: String,
}

fn prefix_to_network(prefix: u16) -> &'static str {
    match prefix {
        0 => "Polkadot",
        2 => "Kusama",
        42 => "Generic Substrate (Westend, Paseo, etc.)",
        _ => "Unknown",
    }
}

fn hex_to_bytes32(hex: &str) -> Result<[u8; 32]> {
    let hex = hex.strip_prefix("0x").unwrap_or(hex);
    if hex.len() != 64 {
        return Err(anyhow::anyhow!(
            "Expected 64 hex characters (32 bytes), got {}",
            hex.len()
        ));
    }
    let bytes = hex::decode(hex)?;
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(arr)
}

pub async fn ss58_decode(params: Ss58DecodeParams) -> Result<CallToolResult> {
    let (account, format) = match AccountId32::from_ss58check_with_version(&params.address) {
        Ok(result) => result,
        Err(e) => return Ok(error_result(&format!("Invalid SS58 address: {:?}", e))),
    };

    let prefix: u16 = format.into();
    let raw: &[u8; 32] = account.as_ref();
    let hex = hex::encode(raw);

    let mut output = String::new();
    output.push_str(&format!("Address: {}\n", params.address));
    output.push_str(&format!("Public Key (hex): 0x{}\n", hex));
    output.push_str(&format!("SS58 Prefix: {}\n", prefix));
    output.push_str(&format!("Network: {}", prefix_to_network(prefix)));

    Ok(text_result(&output))
}

pub async fn ss58_encode(params: Ss58EncodeParams) -> Result<CallToolResult> {
    let bytes = match hex_to_bytes32(&params.public_key_hex) {
        Ok(b) => b,
        Err(e) => return Ok(error_result(&format!("Invalid public key hex: {}", e))),
    };

    let account = AccountId32::from(bytes);
    let format = Ss58AddressFormat::custom(params.prefix);
    let address = account.to_ss58check_with_version(format);

    let hex = params
        .public_key_hex
        .strip_prefix("0x")
        .unwrap_or(&params.public_key_hex);

    let mut output = String::new();
    output.push_str(&format!("Public Key: 0x{}\n", hex));
    output.push_str(&format!(
        "SS58 Prefix: {} ({})\n",
        params.prefix,
        prefix_to_network(params.prefix)
    ));
    output.push_str(&format!("Address: {}", address));

    Ok(text_result(&output))
}

pub async fn ss58_convert(params: Ss58ConvertParams) -> Result<CallToolResult> {
    let (account, source_format) = match AccountId32::from_ss58check_with_version(&params.address) {
        Ok(result) => result,
        Err(e) => return Ok(error_result(&format!("Invalid SS58 address: {:?}", e))),
    };

    let source_prefix: u16 = source_format.into();
    let target_format = Ss58AddressFormat::custom(params.target_prefix);
    let new_address = account.to_ss58check_with_version(target_format);
    let raw: &[u8; 32] = account.as_ref();
    let hex = hex::encode(raw);

    let mut output = String::new();
    output.push_str(&format!(
        "Source: {} (prefix {}; {})\n",
        params.address,
        source_prefix,
        prefix_to_network(source_prefix)
    ));
    output.push_str(&format!(
        "Target: {} (prefix {}; {})\n",
        new_address,
        params.target_prefix,
        prefix_to_network(params.target_prefix)
    ));
    output.push_str(&format!("Public Key: 0x{}\n", hex));
    output.push_str("Same account on both networks: yes");

    Ok(text_result(&output))
}

pub async fn ss58_validate(params: Ss58ValidateParams) -> Result<CallToolResult> {
    match AccountId32::from_ss58check_with_version(&params.address) {
        Ok((account, format)) => {
            let prefix: u16 = format.into();
            let raw: &[u8; 32] = account.as_ref();
            let hex = hex::encode(raw);

            let mut output = String::new();
            output.push_str(&format!("Address: {}\n", params.address));
            output.push_str("Valid: true\n");
            output.push_str(&format!("SS58 Prefix: {}\n", prefix));
            output.push_str(&format!("Network: {}\n", prefix_to_network(prefix)));
            output.push_str(&format!("Public Key: 0x{}", hex));

            Ok(text_result(&output))
        }
        Err(e) => {
            let mut output = String::new();
            output.push_str(&format!("Address: {}\n", params.address));
            output.push_str("Valid: false\n");
            output.push_str(&format!("Error: {:?}", e));

            Ok(text_result(&output))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Alice's well-known public key (from sp_keyring)
    const ALICE_HEX: &str = "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";

    /// Compute Alice's SS58 address for a given prefix from the known public key.
    fn alice_address(prefix: u16) -> String {
        let bytes = hex::decode(ALICE_HEX).unwrap();
        let account = AccountId32::new(bytes.try_into().unwrap());
        account.to_ss58check_with_version(Ss58AddressFormat::custom(prefix))
    }

    fn extract_text(result: &CallToolResult) -> &str {
        match &result.content[0].raw {
            rmcp::model::RawContent::Text(t) => &t.text,
            _ => panic!("expected text content"),
        }
    }

    #[tokio::test]
    async fn test_ss58_decode_polkadot() {
        let addr = alice_address(0);
        let result = ss58_decode(Ss58DecodeParams {
            address: addr,
        })
        .await
        .unwrap();

        let text = extract_text(&result);
        assert!(text.contains(&format!("0x{}", ALICE_HEX)));
        assert!(text.contains("SS58 Prefix: 0"));
        assert!(text.contains("Network: Polkadot"));
    }

    #[tokio::test]
    async fn test_ss58_decode_kusama() {
        let addr = alice_address(2);
        let result = ss58_decode(Ss58DecodeParams {
            address: addr,
        })
        .await
        .unwrap();

        let text = extract_text(&result);
        assert!(text.contains(&format!("0x{}", ALICE_HEX)));
        assert!(text.contains("SS58 Prefix: 2"));
        assert!(text.contains("Network: Kusama"));
    }

    #[tokio::test]
    async fn test_ss58_encode_polkadot() {
        let expected = alice_address(0);
        let result = ss58_encode(Ss58EncodeParams {
            public_key_hex: format!("0x{}", ALICE_HEX),
            prefix: 0,
        })
        .await
        .unwrap();

        let text = extract_text(&result);
        assert!(text.contains(&expected));
    }

    #[tokio::test]
    async fn test_ss58_encode_kusama() {
        let expected = alice_address(2);
        let result = ss58_encode(Ss58EncodeParams {
            public_key_hex: ALICE_HEX.to_string(),
            prefix: 2,
        })
        .await
        .unwrap();

        let text = extract_text(&result);
        assert!(text.contains(&expected));
    }

    #[tokio::test]
    async fn test_ss58_convert_polkadot_to_kusama() {
        let polkadot_addr = alice_address(0);
        let kusama_addr = alice_address(2);
        let result = ss58_convert(Ss58ConvertParams {
            address: polkadot_addr,
            target_prefix: 2,
        })
        .await
        .unwrap();

        let text = extract_text(&result);
        assert!(text.contains(&kusama_addr));
        assert!(text.contains(&format!("0x{}", ALICE_HEX)));
        assert!(text.contains("Same account on both networks: yes"));
    }

    #[tokio::test]
    async fn test_ss58_convert_kusama_to_generic() {
        let kusama_addr = alice_address(2);
        let generic_addr = alice_address(42);
        let result = ss58_convert(Ss58ConvertParams {
            address: kusama_addr,
            target_prefix: 42,
        })
        .await
        .unwrap();

        let text = extract_text(&result);
        assert!(text.contains(&generic_addr));
    }

    #[tokio::test]
    async fn test_ss58_validate_valid() {
        let addr = alice_address(0);
        let result = ss58_validate(Ss58ValidateParams {
            address: addr,
        })
        .await
        .unwrap();

        let text = extract_text(&result);
        assert!(text.contains("Valid: true"));
        assert!(text.contains("SS58 Prefix: 0"));
    }

    #[tokio::test]
    async fn test_ss58_validate_invalid() {
        let result = ss58_validate(Ss58ValidateParams {
            address: "notavalidaddress".to_string(),
        })
        .await
        .unwrap();

        let text = extract_text(&result);
        assert!(text.contains("Valid: false"));
    }

    #[tokio::test]
    async fn test_ss58_decode_invalid_address() {
        let result = ss58_decode(Ss58DecodeParams {
            address: "garbage".to_string(),
        })
        .await
        .unwrap();

        assert_eq!(result.is_error, Some(true));
    }

    #[tokio::test]
    async fn test_ss58_encode_invalid_hex() {
        let result = ss58_encode(Ss58EncodeParams {
            public_key_hex: "not_hex".to_string(),
            prefix: 0,
        })
        .await
        .unwrap();

        assert_eq!(result.is_error, Some(true));
    }

    #[tokio::test]
    async fn test_ss58_encode_short_hex() {
        let result = ss58_encode(Ss58EncodeParams {
            public_key_hex: "0xdead".to_string(),
            prefix: 0,
        })
        .await
        .unwrap();

        assert_eq!(result.is_error, Some(true));
    }

    #[tokio::test]
    async fn test_roundtrip_decode_encode() {
        let polkadot_addr = alice_address(0);

        // Decode to get the public key
        let decode_result = ss58_decode(Ss58DecodeParams {
            address: polkadot_addr.clone(),
        })
        .await
        .unwrap();
        let decode_text = extract_text(&decode_result);
        assert!(decode_text.contains(&format!("0x{}", ALICE_HEX)));

        // Re-encode with same prefix should give back the same address
        let encode_result = ss58_encode(Ss58EncodeParams {
            public_key_hex: format!("0x{}", ALICE_HEX),
            prefix: 0,
        })
        .await
        .unwrap();
        let encode_text = extract_text(&encode_result);
        assert!(encode_text.contains(&polkadot_addr));
    }
}
