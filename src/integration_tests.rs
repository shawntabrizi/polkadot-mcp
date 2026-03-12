//! Integration tests that run against Westend testnet.
//!
//! These tests are marked `#[ignore]` so they don't run with `cargo test`.
//! Run them explicitly with: `cargo test -- --ignored`
//!
//! They require network access to wss://westend-rpc.polkadot.io

use crate::server::PolkadotMcp;
use crate::tools::{account, chain, metadata};
use rmcp::model::RawContent;

/// Extract text content from a CallToolResult.
fn extract_text(result: &rmcp::model::CallToolResult) -> &str {
    match &result.content[0].raw {
        RawContent::Text(t) => &t.text,
        _ => panic!("expected text content"),
    }
}

/// Build a PolkadotMcp server for testing (no signer).
fn test_server() -> PolkadotMcp {
    PolkadotMcp::new(None)
}

/// Alice's public key (well-known dev account, funded on Westend).
const ALICE_HEX: &str = "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";

/// Compute Alice's Westend address (prefix 42).
fn alice_westend() -> String {
    use sp_core::crypto::{AccountId32, Ss58AddressFormat, Ss58Codec};
    let bytes = hex::decode(ALICE_HEX).unwrap();
    let account = AccountId32::new(bytes.try_into().unwrap());
    account.to_ss58check_with_version(Ss58AddressFormat::custom(42))
}

// --- Chain Utilities ---

#[tokio::test]
#[ignore]
async fn test_chain_info_westend() {
    let server = test_server();
    let result = chain::chain_info(
        &server,
        chain::ChainInfoParams {
            network: "westend".into(),
            chain: "relay".into(),
        },
    )
    .await
    .unwrap();

    let text = extract_text(&result);
    assert!(text.contains("westend"), "should contain chain name");
    assert!(text.contains("WND"), "should contain token symbol");
    assert!(text.contains("12 decimals"), "WND has 12 decimals");
    assert!(text.contains("Current Block: #"), "should show block number");
}

#[tokio::test]
#[ignore]
async fn test_chain_info_westend_asset_hub() {
    let server = test_server();
    let result = chain::chain_info(
        &server,
        chain::ChainInfoParams {
            network: "westend".into(),
            chain: "asset-hub".into(),
        },
    )
    .await
    .unwrap();

    let text = extract_text(&result);
    assert!(text.contains("westend-asset-hub"));
    assert!(text.contains("WND"));
}

// --- Account & Balance ---

#[tokio::test]
#[ignore]
async fn test_get_balances_westend() {
    let server = test_server();
    let result = account::get_balances(
        &server,
        account::GetBalancesParams {
            address: alice_westend(),
            network: "westend".into(),
            chain: "relay".into(),
        },
    )
    .await
    .unwrap();

    let text = extract_text(&result);
    assert!(
        text.contains("Balances:") || text.contains("not found"),
        "should return balances or not-found message"
    );
    // If Alice exists on Westend, we should see WND
    if text.contains("Balances:") {
        assert!(text.contains("WND"), "balance should be in WND");
        assert!(text.contains("Free:"), "should show free balance");
        assert!(text.contains("Transferable:"), "should show transferable");
    }
}

#[tokio::test]
#[ignore]
async fn test_get_balances_nonexistent_account() {
    let server = test_server();
    // Use a zero account that almost certainly has no balance
    let zero_addr = {
        use sp_core::crypto::{AccountId32, Ss58AddressFormat, Ss58Codec};
        let account = AccountId32::new([0u8; 32]);
        account.to_ss58check_with_version(Ss58AddressFormat::custom(42))
    };

    let result = account::get_balances(
        &server,
        account::GetBalancesParams {
            address: zero_addr,
            network: "westend".into(),
            chain: "relay".into(),
        },
    )
    .await
    .unwrap();

    // Should return a valid response (not panic), either not-found or zero balances
    let text = extract_text(&result);
    assert!(
        text.contains("not found") || text.contains("0.0000 WND"),
        "zero account should show not-found or zero balance"
    );
}

// --- Metadata Introspection ---

#[tokio::test]
#[ignore]
async fn test_list_pallets_westend() {
    let server = test_server();
    let result = metadata::list_pallets(
        &server,
        metadata::ListPalletsParams {
            network: "westend".into(),
            chain: "relay".into(),
        },
    )
    .await
    .unwrap();

    let text = extract_text(&result);
    assert!(text.contains("System"), "should list System pallet");
    assert!(text.contains("Balances"), "should list Balances pallet");
    assert!(text.contains("Staking"), "should list Staking pallet");
    assert!(
        text.contains("total)"),
        "should show total pallet count"
    );
}

#[tokio::test]
#[ignore]
async fn test_pallet_info_system() {
    let server = test_server();
    let result = metadata::pallet_info(
        &server,
        metadata::PalletInfoParams {
            pallet_name: "System".into(),
            network: "westend".into(),
            chain: "relay".into(),
        },
    )
    .await
    .unwrap();

    let text = extract_text(&result);
    assert!(text.contains("Pallet: System"), "should show pallet name");
    assert!(text.contains("Calls ("), "should list calls");
    assert!(text.contains("Storage ("), "should list storage");
    assert!(text.contains("Account"), "System should have Account storage");
    assert!(text.contains("Events ("), "should list events");
}

#[tokio::test]
#[ignore]
async fn test_pallet_info_staking() {
    let server = test_server();
    let result = metadata::pallet_info(
        &server,
        metadata::PalletInfoParams {
            pallet_name: "Staking".into(),
            network: "westend".into(),
            chain: "relay".into(),
        },
    )
    .await
    .unwrap();

    let text = extract_text(&result);
    assert!(text.contains("Pallet: Staking"));
    assert!(text.contains("nominate"), "Staking should have nominate call");
    assert!(text.contains("bond"), "Staking should have bond call");
    assert!(text.contains("Bonded"), "Staking should have Bonded storage");
}

#[tokio::test]
#[ignore]
async fn test_pallet_info_nonexistent() {
    let server = test_server();
    let result = metadata::pallet_info(
        &server,
        metadata::PalletInfoParams {
            pallet_name: "DoesNotExist".into(),
            network: "westend".into(),
            chain: "relay".into(),
        },
    )
    .await
    .unwrap();

    assert_eq!(result.is_error, Some(true));
    let text = extract_text(&result);
    assert!(text.contains("not found"));
}

#[tokio::test]
#[ignore]
async fn test_pallet_info_case_insensitive() {
    let server = test_server();
    let result = metadata::pallet_info(
        &server,
        metadata::PalletInfoParams {
            pallet_name: "system".into(), // lowercase
            network: "westend".into(),
            chain: "relay".into(),
        },
    )
    .await
    .unwrap();

    let text = extract_text(&result);
    assert!(text.contains("Pallet: System"), "case-insensitive lookup should work");
}

// --- Constant Value ---

#[tokio::test]
#[ignore]
async fn test_constant_value_existential_deposit() {
    let server = test_server();
    let result = metadata::constant_value(
        &server,
        metadata::ConstantValueParams {
            pallet_name: "Balances".into(),
            constant_name: "ExistentialDeposit".into(),
            network: "westend".into(),
            chain: "relay".into(),
        },
    )
    .await
    .unwrap();

    let text = extract_text(&result);
    assert!(text.contains("Balances.ExistentialDeposit"));
    assert!(text.contains("Value:"), "should show the decoded value");
    // ExistentialDeposit is a u128 — the value should be a number
    assert!(text.contains("Type:"), "should show the type");
}

#[tokio::test]
#[ignore]
async fn test_constant_value_bonding_duration() {
    let server = test_server();
    let result = metadata::constant_value(
        &server,
        metadata::ConstantValueParams {
            pallet_name: "Staking".into(),
            constant_name: "BondingDuration".into(),
            network: "westend".into(),
            chain: "relay".into(),
        },
    )
    .await
    .unwrap();

    let text = extract_text(&result);
    assert!(text.contains("Staking.BondingDuration"));
    assert!(text.contains("Value:"));
}

#[tokio::test]
#[ignore]
async fn test_constant_value_nonexistent() {
    let server = test_server();
    let result = metadata::constant_value(
        &server,
        metadata::ConstantValueParams {
            pallet_name: "Balances".into(),
            constant_name: "DoesNotExist".into(),
            network: "westend".into(),
            chain: "relay".into(),
        },
    )
    .await
    .unwrap();

    assert_eq!(result.is_error, Some(true));
}

// --- Account Locks ---

#[tokio::test]
#[ignore]
async fn test_account_locks_westend() {
    let server = test_server();
    let result = account::account_locks(
        &server,
        account::AccountLocksParams {
            address: alice_westend(),
            network: "westend".into(),
            chain: "relay".into(),
        },
    )
    .await
    .unwrap();

    let text = extract_text(&result);
    assert!(text.contains("Account:"), "should show account address");
    assert!(text.contains("Chain: westend"), "should show chain");
    // Should have Locks, Freezes, and Holds sections (even if "none")
    assert!(
        text.contains("Locks:") || text.contains("Locks ("),
        "should show locks section"
    );
    assert!(
        text.contains("Freezes:") || text.contains("Freezes ("),
        "should show freezes section"
    );
    assert!(
        text.contains("Holds:") || text.contains("Holds ("),
        "should show holds section"
    );
}
