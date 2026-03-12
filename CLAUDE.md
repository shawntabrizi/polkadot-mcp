# CLAUDE.md

This file provides guidance to AI coding assistants working on this project.

## Project Overview

`polkadot-mcp` is a Rust MCP server that lets AI agents interact with Polkadot/Substrate chains. It connects to multiple networks (Polkadot, Kusama, Westend, Paseo) and their system parachains simultaneously, exposing high-level tools for querying state, decoding data, and submitting transactions.

Built on `subxt` (dynamic mode) + `rmcp` (Anthropic's Rust MCP SDK).

## Build & Run

```bash
cargo build                          # Debug build
cargo build --release                # Release build
cargo test                           # Run tests
cargo clippy -- -D warnings          # Lint
cargo fmt --check                    # Format check

# Run the MCP server (stdio transport) — all networks available
cargo run

# Test with MCP Inspector
npx @modelcontextprotocol/inspector cargo run
```

## Architecture

### Key Concepts

- **Network**: A group of related chains (relay + system parachains). Presets: Polkadot, Kusama, Westend, Paseo.
- **All networks loaded at startup**: The server holds all four networks. Tools accept a `network` param to select which one.
- **ChainPool**: Shared lazy-connection pool across all networks. Caches `OnlineClient<PolkadotConfig>` by chain name.
- **Chain routing via params**: Tools accept `network` (default: "polkadot") and `chain` (default: "relay") params. Domain-specific tools may hardcode the chain (e.g. fellowship always queries collectives).
- **Backends**: `subxt` for live state, Subscan API for historical/indexed data. Tools pick the right backend.

### Networks & Chains

Each network has these chains (all system parachains):

| Chain Alias    | Purpose                                      |
|----------------|----------------------------------------------|
| `relay`        | Core chain — governance, staking, balances    |
| `asset-hub`    | Asset/token management                        |
| `bridge-hub`   | Cross-chain bridging                          |
| `people`       | Decentralized identity                        |
| `collectives`  | Fellowship, DAOs (not available on Kusama)    |
| `coretime`     | Blockspace allocation                         |

Networks: `polkadot`, `kusama`, `westend`, `paseo`

### Module Map

```
src/
├── main.rs          # Entry: signer loading, server start
├── server.rs        # PolkadotMcp struct, #[tool_router] + #[tool_handler] impls
├── network.rs       # Network + ChainConfig structs, presets for all networks
├── pool.rs          # ChainPool with lazy connection + caching (shared across networks)
├── signer.rs        # Load keypair from POLKADOT_SIGNER_URI env var
├── decode.rs        # subxt Value → human-readable (balance formatting, enum naming)
├── types.rs         # parse_ss58(), format_balance(), text_result(), error_result()
├── tools/           # One file per domain
│   ├── account.rs   # get_balances (implemented)
│   ├── fellowship.rs # TODO: fellowship_status, fellowship_salary, demotion_risk
│   ├── governance.rs # TODO: referenda_active, referendum_detail, vote, delegate
│   ├── staking.rs   # TODO: staking_status, staking_rewards, nomination_pools
│   └── chain.rs     # chain_info (implemented)
└── backends/
    ├── subxt_backend.rs  # All subxt dynamic storage/tx helpers
    └── subscan.rs        # Subscan REST API client
```

### How subxt Dynamic Queries Work

```rust
// Build a storage address from strings (no compile-time types)
let addr = subxt::dynamic::storage("System", "Account", vec![
    Value::from_bytes(account_id_bytes),
]);

// Fetch from chain
let result = api.storage().at_latest().await?.fetch(&addr).await?;

// Decode the dynamic value — returns Value<u32> (DecodedValue)
if let Some(value) = result {
    let decoded = value.to_value()?;
    // Navigate: decoded.at("field_name") or decoded.at(0) (requires `use subxt::dynamic::At`)
}
```

For transactions:
```rust
let tx = subxt::dynamic::tx("ConvictionVoting", "vote", vec![
    Value::u128(poll_index),
    vote_value,
]);

// Dry-run first, then submit
let signed = api.tx().create_signed(&tx, &signer, Default::default()).await?;
signed.dry_run(None).await?;
signed.submit_and_watch().await?;
```

### How rmcp Tools Work

Tools use three macros: `#[tool]` on methods, `#[tool_router]` on the impl block, and `#[tool_handler]` on `impl ServerHandler`.

```rust
use rmcp::schemars::{self, JsonSchema};  // Must use rmcp's re-exported schemars (v1), not schemars v0.8

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MyParams {
    /// Description shown to the AI for this parameter
    pub address: String,
    #[serde(default = "default_network")]
    pub network: String,
    #[serde(default = "default_chain")]
    pub chain: String,
}

// In server.rs — tool method in the #[tool_router] impl block:
#[tool(description = "One-line description the AI reads to decide when to call this tool.")]
async fn my_tool(
    &self,
    Parameters(params): Parameters<my_module::MyParams>,
) -> Result<CallToolResult, ErrorData> {
    my_module::my_tool(self, params)
        .await
        .map_err(|e| ErrorData::internal_error(e.to_string(), None))
}
```

The `#[tool_handler(router = "self.tool_router")]` on `impl ServerHandler` generates `list_tools` and `call_tool` methods that delegate to the router.

## Critical Rules

### MCP stdio Transport
- **NEVER write to stdout** except MCP protocol messages. All logging goes to stderr.
- Use `tracing_subscriber::fmt().with_writer(std::io::stderr)` in main.

### Balance Formatting
- DOT/PAS have 10 decimals (1 DOT = 10_000_000_000 planck)
- KSM/WND have 12 decimals (1 KSM = 1_000_000_000_000 planck)
- Always use `ChainConfig.token_decimals` — never hardcode decimals.
- Display at most 4 decimal places for readability.

### SS58 Addresses
- Parse with `bs58` crate (manual base58 decode).
- Accept any valid SS58 address — the prefix just helps with display.

### Schemars Version
- rmcp 0.16 re-exports `schemars` v1. Our Cargo.toml has `schemars` v0.8.
- Tool param structs MUST use `rmcp::schemars::JsonSchema`, not `schemars::JsonSchema`.
- Import as: `use rmcp::schemars::{self, JsonSchema};`

### Tool Descriptions
- Tool descriptions are read by the AI to decide when to call each tool. Make them specific.
- Always list the available `network` and `chain` param values in the description.
- For write tools, include: "IMPORTANT: This submits a real transaction. Confirm with the user first."

### Error Handling
- Return errors as `CallToolResult { is_error: Some(true), content: [text] }`, not panics.
- Use `error_result()` helper from `types.rs`.
- Include actionable context: "Account not found on polkadot. Try specifying chain: 'asset-hub'."

### Dynamic Value Decoding
- This is the hardest part of the codebase. subxt returns `DecodedValueThunk` / `Value<u32>` types.
- Use the `At` trait (import `subxt::dynamic::At`) for `.at("field_name")` navigation.
- Enum variants: pattern match on `ValueDef::Variant(v)` — there is no `.as_variant()` method.
- Always handle missing fields gracefully — chain metadata can change between runtime versions.

## On-Chain Storage Quick Reference

### Fellowship (Collectives chain)
- Rank: `FellowshipCollective.Members[account]` → `{ rank }`
- Activity: `CoreFellowship.Member[account]` → `{ is_active, last_promotion, last_proof }`
- Demotion params: `CoreFellowship.Params` → `{ demotion_period[], min_promotion_period }`
- Salary claim: `FellowshipSalary.Claimant[account]` → `{ last_active, status }`
- Salary cycle: `FellowshipSalary.Status` → `{ cycle, cycle_start, budget }`

### Governance (Relay chain)
- Referendum: `Referenda.ReferendumInfoFor[index]` → `Ongoing{...}` or terminal
- Votes: `ConvictionVoting.VotingFor[account][track]` → `Casting{votes}` or `Delegating{...}`
- Locks: `ConvictionVoting.ClassLocksFor[account]` → `Vec<(class, balance)>`

### Staking (Relay chain)
- Bonded: `Staking.Bonded[stash]` → controller
- Ledger: `Staking.Ledger[controller]` → `{ stash, total, active, unlocking }`
- Nominations: `Staking.Nominators[account]` → `{ targets, submitted_in }`
- Pool member: `NominationPools.PoolMembers[account]` → `{ pool_id, points }`

### Account (Any chain)
- Balance: `System.Account[account]` → `{ nonce, data: { free, reserved, frozen } }`
- Locks: `Balances.Locks[account]` → `Vec<{ id, amount, reasons }>`
- Freezes: `Balances.Freezes[account]` → `Vec<(FreezeId, Balance)>`
- Holds: `Balances.Holds[account]` → `Vec<(HoldReason, Balance)>`

## Testing

- **Unit tests**: Test `types.rs` formatting and value navigation.
- **MCP Inspector**: `npx @modelcontextprotocol/inspector cargo run` to debug tool call/response flow.
- **Manual testing**: Add to Claude Desktop config, ask natural questions, iterate on tool descriptions.

## Dependencies

| Crate | Purpose |
|---|---|
| `rmcp` (features: server, macros, transport-io) | MCP SDK |
| `subxt` | Substrate chain client (dynamic mode) |
| `subxt-signer` (features: sr25519) | Transaction signing |
| `tokio` (features: full) | Async runtime |
| `serde`, `serde_json` | Serialization |
| `schemars` | JSON Schema for tool params (but use rmcp's re-export!) |
| `bs58` | SS58 address decoding |
| `reqwest` (features: json) | HTTP client for Subscan API |
| `tracing`, `tracing-subscriber` | Logging (to stderr only!) |
| `anyhow`, `thiserror` | Error handling |
| `futures` | `join_all` for parallel queries |
