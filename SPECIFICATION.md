# polkadot-mcp — Specification

## Overview

`polkadot-mcp` is a Rust MCP (Model Context Protocol) server that lets any AI agent interact with Polkadot, Kusama, Westend, Paseo, and other Substrate-based chains. It replaces browser-based UIs (Polkadot-JS Apps) with AI-native interfaces: the AI becomes the UI.

A single MCP server instance has access to **all networks simultaneously** (Polkadot, Kusama, Westend, Paseo) — each with their relay chain and system parachains (Asset Hub, Bridge Hub, People, Collectives, Coretime). Tools accept `network` and `chain` parameters to route queries, so the AI can query any chain without restarting the server. The server exposes high-level, intent-based tools that any MCP-compatible client (Claude, ChatGPT, Cursor, VS Code, custom agents) can use.

## Tech Stack

| Component | Choice | Why |
|---|---|---|
| Language | Rust (2021 edition) | Single binary, no runtime deps, same language as Polkadot SDK |
| Chain client | `subxt` (dynamic mode) | No codegen needed, works with any Substrate chain at runtime |
| Signing | `subxt-signer` | sr25519/ecdsa, BIP-39 phrases, `//Dev` URI derivation |
| MCP SDK | `rmcp` | Official Rust MCP SDK. `#[tool]`, `#[tool_router]`, `#[tool_handler]` macros, stdio transport |
| Async | `tokio` | Required by both subxt and rmcp |
| Serialization | `serde`, `serde_json` | Tool params and return values |
| Schema | `schemars` | Auto-generates JSON Schema for MCP tool parameters |
| HTTP client | `reqwest` | For block explorer API backends (Subscan, etc.) |
| Logging | `tracing` | MUST write to stderr (stdout is the MCP stdio transport) |

## Architecture

### Crate Structure

```
polkadot-mcp/
├── Cargo.toml
├── Cargo.lock
├── CLAUDE.md                # Agent instructions for building this project
├── SPECIFICATION.md         # This file
├── README.md
├── src/
│   ├── main.rs              # Entry point: parse env, build server, start stdio
│   ├── server.rs            # PolkadotMcp struct, #[tool_router] + #[tool_handler] impl
│   ├── network.rs           # Network, ChainConfig, chain presets
│   ├── pool.rs              # ChainPool: lazy connection manager
│   ├── signer.rs            # Key management from env var / URI
│   ├── decode.rs            # Dynamic Value → human-readable conversion
│   ├── types.rs             # Shared types, helper fns (parse_ss58, format_balance)
│   ├── tools/
│   │   ├── mod.rs           # Re-exports all tool modules
│   │   ├── account.rs       # account_info, account_balances, account_locks, unlock_frozen
│   │   ├── fellowship.rs    # fellowship_status, fellowship_salary, demotion_risk
│   │   ├── governance.rs    # referenda_active, referendum_detail, vote, delegate
│   │   ├── staking.rs       # staking_status, staking_rewards, nomination_pools
│   │   └── chain.rs         # chain_info, block_info, decode_extrinsic, query_storage
│   └── backends/
│       ├── mod.rs
│       ├── subxt.rs          # Live on-chain state queries via subxt dynamic
│       └── subscan.rs        # Indexed/historical data via Subscan REST API
└── tests/
    └── integration.rs        # Integration tests against Westend testnet
```

### Multi-Network, Multi-Chain Design

A single server instance manages **all networks simultaneously**. At startup, it creates a `HashMap<String, Network>` with entries for Polkadot, Kusama, Westend, and Paseo. Each **Network** is a group of related chains (relay + system parachains). A shared **ChainPool** lazily creates and caches one `OnlineClient<PolkadotConfig>` per chain across all networks.

```
PolkadotMcp
├── networks["polkadot"]  → Network { relay, asset_hub, bridge_hub, people, collectives, coretime }
├── networks["kusama"]    → Network { relay, asset_hub, bridge_hub, people, coretime }  (no collectives)
├── networks["westend"]   → Network { relay, asset_hub, bridge_hub, people, collectives, coretime }
├── networks["paseo"]     → Network { relay, asset_hub, bridge_hub, people, collectives, coretime }
└── pool                  → ChainPool (shared, caches connections by chain name)
```

**Tools accept `network` and `chain` parameters** to route queries. The server resolves `(network, chain)` → `ChainConfig` → cached connection.

| Tool | Default network | Default chain | Routing |
|---|---|---|---|
| `chain_info` | polkadot | relay | `server.resolve(&network, &chain)` |
| `get_balances` | polkadot | relay | `server.resolve(&network, &chain)` |
| `fellowship_status` | polkadot | collectives | Hardcoded chain, network param |
| `referenda_active` | polkadot | relay | Hardcoded chain, network param |
| `query_storage` | polkadot | (required) | Both params explicit |

Each `ChainConfig` carries metadata (token symbol, decimals, SS58 prefix) so tools format output correctly regardless of network.

Note: Kusama does not have a Collectives chain, so `collectives` is `Option<ChainConfig>`.

### Connection Lifecycle

1. Server starts → creates all 4 `Network` presets + empty shared `ChainPool` (no connections yet)
2. First tool call for e.g. Polkadot relay → `pool.get(&config)` opens WebSocket, downloads metadata (~200KB–1MB), caches client by chain name
3. First tool call for e.g. Kusama asset-hub → same lazy init, separate cached client
4. Subsequent calls to the same chain → reuse cached `OnlineClient` (cheap clone via `Arc` internally)
5. Connections are cached across all networks in the same pool

### Backend Strategy

Tools use the best backend for each query:

| Data type | Backend | Why |
|---|---|---|
| Live chain state (balances, locks, staking, fellowship) | `subxt` | Real-time, trustless |
| Transaction history, transfer lists | Subscan API | Requires indexing, not in runtime state |
| Reward history (per-era payouts) | Subscan API | Historical aggregation |
| Static reference data (track names, conviction table) | Baked into binary | Never changes, no RPC needed |

The `backends/` module abstracts this. Tools call backend functions, not raw subxt/reqwest.

## Tool Specifications

See [TOOLS.md](TOOLS.md) for the full tool catalog organized by category.

## On-Chain Storage Reference

### Fellowship (Collectives chain)

| Pallet | Storage | Key | Returns |
|---|---|---|---|
| `FellowshipCollective` | `Members` | AccountId | `MemberRecord { rank }` |
| `FellowshipCollective` | `MemberCount` | Rank | u32 |
| `CoreFellowship` | `Member` | AccountId | `{ is_active, last_promotion, last_proof }` |
| `CoreFellowship` | `MemberEvidence` | AccountId | `(Wish, Evidence)` |
| `CoreFellowship` | `Params` | — | `{ demotion_period[], min_promotion_period, offboard_timeout }` |
| `FellowshipSalary` | `Claimant` | AccountId | `{ last_active, status }` |
| `FellowshipSalary` | `Status` | — | `{ cycle, cycle_start, budget }` |

**Demotion:** `current_block - last_proof > demotion_period[rank]` → eligible via `core_fellowship.bump(who)`

**Salary:** Can claim if `last_active != current_cycle && status != Attempted`. Call `register()` then `claim()`.

### OpenGov (Relay chain)

| Pallet | Storage | Key | Returns |
|---|---|---|---|
| `Referenda` | `ReferendumInfoFor` | ReferendumIndex | `Ongoing{...}` or terminal state |
| `Referenda` | `ReferendumCount` | — | u32 |
| `ConvictionVoting` | `VotingFor` | (AccountId, TrackId) | `Casting{votes}` or `Delegating{target}` |
| `ConvictionVoting` | `ClassLocksFor` | AccountId | `Vec<(Class, Balance)>` |

**Conviction:** None=0.1x (no lock), 1x (7d), 2x (14d), 3x (28d), 4x (56d), 5x (112d), 6x (224d)

**Unlock:** `conviction_voting.unlock(class, target)` for expired locks.

### Staking (Relay chain)

| Pallet | Storage | Key | Returns |
|---|---|---|---|
| `Staking` | `Bonded` | StashId | ControllerId |
| `Staking` | `Ledger` | ControllerId | `{ stash, total, active, unlocking }` |
| `Staking` | `Nominators` | AccountId | `{ targets, submitted_in }` |
| `Staking` | `CurrentEra` | — | EraIndex |
| `NominationPools` | `PoolMembers` | AccountId | `{ pool_id, points, unbonding_eras }` |
| `NominationPools` | `BondedPools` | PoolId | `{ commission, member_counter, points, roles, state }` |

### Account Basics (Any chain)

| Pallet | Storage | Key | Returns |
|---|---|---|---|
| `System` | `Account` | AccountId | `{ nonce, data: { free, reserved, frozen } }` |
| `Balances` | `Locks` | AccountId | `Vec<{ id, amount, reasons }>` (deprecated but still present) |
| `Balances` | `Freezes` | AccountId | `Vec<(FreezeId, Balance)>` |
| `Balances` | `Holds` | AccountId | `Vec<(HoldReason, Balance)>` |
| `Vesting` | `Vesting` | AccountId | `Vec<{ locked, per_block, starting_block }>` |
| `Proxy` | `Proxies` | AccountId | `(Vec<ProxyDef>, deposit)` |

## Configuration

### Environment Variables

| Variable | Required | Default | Description |
|---|---|---|---|
| `POLKADOT_SIGNER_URI` | No | (none, read-only) | Signer key URI. e.g. `//Alice` or mnemonic phrase |
| `SUBSCAN_API_KEY` | No | (none) | Subscan API key for historical data queries |

All networks (Polkadot, Kusama, Westend, Paseo) are available simultaneously — no `POLKADOT_NETWORK` env var needed. Tools accept a `network` parameter to select which network to query.

### MCP Client Configuration

```json
{
  "mcpServers": {
    "polkadot": {
      "command": "polkadot-mcp",
      "env": {
        "POLKADOT_SIGNER_URI": "bottom drive obey lake curtain smoke basket hold race lonely fit walk//polkadot"
      }
    }
  }
}
```

Read-only (no signer, no transaction tools):
```json
{
  "mcpServers": {
    "polkadot": {
      "command": "polkadot-mcp"
    }
  }
}
```

## Implementation Phases

### Phase 1: Foundation (read-only)
- `main.rs`, `server.rs` with rmcp stdio setup
- `network.rs` with Polkadot/Kusama/Westend presets
- `pool.rs` with lazy ChainPool
- `decode.rs` with balance formatting, enum variant naming
- Tools: `chain_info`, `account_info`, `account_balances`, `query_storage`
- Test with Claude Desktop against Polkadot mainnet

### Phase 2: Fellowship & Governance reads
- Tools: `fellowship_status`, `fellowship_salary`, `fellowship_demotion_risk`, `fellowship_members`
- Tools: `referenda_active`, `referendum_detail`, `my_votes`, `delegation_status`
- Multi-chain queries (relay + collectives in parallel)

### Phase 3: Staking reads + Subscan backend
- Tools: `staking_status`, `staking_rewards`, `nomination_pools`
- `backends/subscan.rs` for transaction history and reward history
- Tools: `account_transfers`, `account_locks`

### Phase 4: Transaction submission
- `signer.rs` key management
- Tools: `vote`, `delegate`, `unlock_frozen`, `fellowship_claim_salary`, `claim_rewards`
- Dry-run before every broadcast
- Tool descriptions instruct AI to confirm with user before signing

### Phase 5: Polish & Ship
- Error messages optimized for AI recovery
- Tool descriptions refined from real usage
- MCP server instructions (context) with Polkadot domain knowledge
- README with installation and usage examples
- Publish to crates.io and MCP registry

## Network Presets

### Polkadot

| Chain | Endpoint | Token | Decimals | SS58 |
|---|---|---|---|---|
| Relay | `wss://rpc.polkadot.io` | DOT | 10 | 0 |
| Asset Hub | `wss://polkadot-asset-hub-rpc.polkadot.io` | DOT | 10 | 0 |
| Bridge Hub | `wss://polkadot-bridge-hub-rpc.polkadot.io` | DOT | 10 | 0 |
| People | `wss://polkadot-people-rpc.polkadot.io` | DOT | 10 | 0 |
| Collectives | `wss://polkadot-collectives-rpc.polkadot.io` | DOT | 10 | 0 |
| Coretime | `wss://polkadot-coretime-rpc.polkadot.io` | DOT | 10 | 0 |

### Kusama

| Chain | Endpoint | Token | Decimals | SS58 |
|---|---|---|---|---|
| Relay | `wss://kusama-rpc.polkadot.io` | KSM | 12 | 2 |
| Asset Hub | `wss://kusama-asset-hub-rpc.polkadot.io` | KSM | 12 | 2 |
| Bridge Hub | `wss://kusama-bridge-hub-rpc.polkadot.io` | KSM | 12 | 2 |
| People | `wss://kusama-people-rpc.polkadot.io` | KSM | 12 | 2 |
| Collectives | *(none)* | — | — | — |
| Coretime | `wss://kusama-coretime-rpc.polkadot.io` | KSM | 12 | 2 |

### Westend (testnet)

| Chain | Endpoint | Token | Decimals | SS58 |
|---|---|---|---|---|
| Relay | `wss://westend-rpc.polkadot.io` | WND | 12 | 42 |
| Asset Hub | `wss://westend-asset-hub-rpc.polkadot.io` | WND | 12 | 42 |
| Bridge Hub | `wss://westend-bridge-hub-rpc.polkadot.io` | WND | 12 | 42 |
| People | `wss://westend-people-rpc.polkadot.io` | WND | 12 | 42 |
| Collectives | `wss://westend-collectives-rpc.polkadot.io` | WND | 12 | 42 |
| Coretime | `wss://westend-coretime-rpc.polkadot.io` | WND | 12 | 42 |

### Paseo (testnet)

| Chain | Endpoint | Token | Decimals | SS58 |
|---|---|---|---|---|
| Relay | `wss://paseo.ibp.network` | PAS | 10 | 42 |
| Asset Hub | `wss://asset-hub-paseo.ibp.network` | PAS | 10 | 42 |
| Bridge Hub | `wss://bridge-hub-paseo.ibp.network` | PAS | 10 | 42 |
| People | `wss://people-paseo.ibp.network` | PAS | 10 | 42 |
| Collectives | `wss://collectives-paseo.ibp.network` | PAS | 10 | 42 |
| Coretime | `wss://coretime-paseo.ibp.network` | PAS | 10 | 42 |
