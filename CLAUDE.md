# CLAUDE.md

This file provides guidance to AI coding assistants working on this project.

## Project Overview

`polkadot-mcp` is a Rust MCP server that lets AI agents interact with Polkadot/Substrate chains. It connects to multiple chains simultaneously (relay, Collectives, Asset Hub) and exposes high-level tools for querying state, decoding data, and submitting transactions.

Built on `subxt` (dynamic mode) + `rmcp` (Anthropic's Rust MCP SDK).

## Build & Run

```bash
cargo build                          # Debug build
cargo build --release                # Release build
cargo test                           # Run tests
cargo clippy -- -D warnings          # Lint
cargo fmt --check                    # Format check

# Run the MCP server (stdio transport)
cargo run

# Run with environment config
POLKADOT_NETWORK=polkadot cargo run
POLKADOT_NETWORK=westend cargo run   # Use testnet

# Test with MCP Inspector
npx @anthropic-ai/mcp-inspector cargo run
```

## Architecture

### Key Concepts

- **Network**: A group of related chains (relay + system parachains). Preset configs exist for Polkadot, Kusama, Westend.
- **ChainPool**: Manages lazy-connected `OnlineClient<PolkadotConfig>` instances, one per chain. Cached after first connection.
- **Tools own their chain routing**: `fellowship_status` always queries Collectives. `referenda_active` always queries relay. The AI doesn't pick chains.
- **Backends**: `subxt` for live state, Subscan API for historical/indexed data. Tools pick the right backend.

### Module Map

```
src/
├── main.rs          # Entry: env parsing, Network selection, server start
├── server.rs        # PolkadotMcp struct with #[tool_box] impl
├── network.rs       # Network + ChainConfig structs, presets
├── pool.rs          # ChainPool with lazy connection + caching
├── signer.rs        # Load keypair from POLKADOT_SIGNER_URI env var
├── decode.rs        # subxt Value → human-readable (balance formatting, enum naming)
├── types.rs         # parse_ss58(), format_balance(), format_blocks_as_time()
├── tools/           # One file per domain, each contains #[tool] methods
│   ├── account.rs   # account_info, account_balances, account_locks, unlock_frozen
│   ├── fellowship.rs # fellowship_status, fellowship_salary, demotion_risk
│   ├── governance.rs # referenda_active, referendum_detail, vote, delegate
│   ├── staking.rs   # staking_status, staking_rewards, nomination_pools
│   └── chain.rs     # chain_info, block_info, decode_extrinsic, query_storage
└── backends/
    ├── subxt.rs     # All subxt dynamic storage/tx helpers
    └── subscan.rs   # Subscan REST API client
```

### How subxt Dynamic Queries Work

```rust
// Build a storage address from strings (no compile-time types)
let addr = subxt::dynamic::storage("System", "Account", vec![
    Value::from_bytes(account_id_bytes),
]);

// Fetch from chain
let result = api.storage().at_latest().await?.fetch(&addr).await?;

// Decode the dynamic value
if let Some(value) = result {
    let decoded = value.to_value()?;
    // Navigate: decoded.at("field_name") or decoded.at(0)
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

```rust
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct MyParams {
    /// Description shown to the AI for this parameter
    pub address: String,
}

#[tool(description = "One-line description the AI reads to decide when to call this tool.")]
async fn my_tool(
    &self,
    #[tool(params)] params: MyParams,
) -> Result<CallToolResult, McpError> {
    // Query chain...
    Ok(CallToolResult {
        content: vec![Content::text("Human-readable response")],
        is_error: false,
    })
}
```

## Critical Rules

### MCP stdio Transport
- **NEVER write to stdout** except MCP protocol messages. All logging goes to stderr.
- Use `tracing_subscriber::fmt().with_writer(std::io::stderr)` in main.

### Balance Formatting
- DOT has 10 decimals (1 DOT = 10_000_000_000 planck)
- KSM has 12 decimals (1 KSM = 1_000_000_000_000 planck)
- Always use `ChainConfig.token_decimals` — never hardcode decimals.
- Display at most 4 decimal places for readability.

### SS58 Addresses
- Parse with `sp-core` or manual base58 decode.
- Validate the SS58 prefix matches the target chain.
- Accept any valid SS58 address — the prefix just helps with display.

### Tool Descriptions
- Tool descriptions are read by the AI to decide when to call each tool. Make them specific.
- Include what question the tool answers: "Check if fellowship salary is claimable this cycle."
- For write tools, include: "IMPORTANT: This submits a real transaction. Confirm with the user first."

### Error Handling
- Return errors as `CallToolResult { is_error: true, content: [text] }`, not panics.
- Include actionable context: "Account not found on Polkadot relay chain. Try specifying chain: 'asset-hub'."
- For dispatch errors, decode using metadata: `pallet.error_name — description`.

### Dynamic Value Decoding
- This is the hardest part of the codebase. subxt returns `DecodedValueThunk` / `Value` types.
- Navigate with `.at("field_name")` for named fields, `.at(0)` for tuple positions.
- Enum variants: check `.as_variant()` for `(name, fields)`.
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

- **Unit tests**: Mock subxt responses, test `decode.rs` formatting and value navigation.
- **Integration tests**: Connect to Westend testnet (`POLKADOT_NETWORK=westend`), run real queries.
- **MCP Inspector**: `npx @anthropic-ai/mcp-inspector cargo run` to debug tool call/response flow.
- **Manual testing**: Add to Claude Desktop config, ask natural questions, iterate on tool descriptions.

## Dependencies

| Crate | Purpose |
|---|---|
| `rmcp` (features: server, macros, transport-io) | MCP SDK |
| `subxt` | Substrate chain client (dynamic mode) |
| `subxt-signer` (features: sr25519) | Transaction signing |
| `tokio` (features: full) | Async runtime |
| `serde`, `serde_json` | Serialization |
| `schemars` | JSON Schema for tool params |
| `reqwest` (features: json) | HTTP client for Subscan API |
| `tracing`, `tracing-subscriber` | Logging (to stderr only!) |
| `anyhow`, `thiserror` | Error handling |
| `futures` | `join_all` for parallel queries |
