# polkadot-mcp — Specification

## Overview

`polkadot-mcp` is a Rust MCP (Model Context Protocol) server that lets any AI agent interact with Polkadot, Kusama, and other Substrate-based chains. It replaces browser-based UIs (Polkadot-JS Apps) with AI-native interfaces: the AI becomes the UI.

A single MCP server instance connects to multiple chains simultaneously (relay chain, Collectives, Asset Hub) and exposes high-level, intent-based tools that any MCP-compatible client (Claude, ChatGPT, Cursor, VS Code, custom agents) can use.

## Tech Stack

| Component | Choice | Why |
|---|---|---|
| Language | Rust (2021 edition) | Single binary, no runtime deps, same language as Polkadot SDK |
| Chain client | `subxt` (dynamic mode) | No codegen needed, works with any Substrate chain at runtime |
| Signing | `subxt-signer` | sr25519/ecdsa, BIP-39 phrases, `//Dev` URI derivation |
| MCP SDK | `rmcp` | Official Rust MCP SDK from Anthropic. `#[tool]` macro, stdio transport |
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
│   ├── server.rs            # PolkadotMcp struct, #[tool_box] impl
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

### Multi-Chain Design

A single server instance manages a **Network** — a group of related chains (relay + system parachains). The server holds a **ChainPool** that lazily creates and caches one `OnlineClient<PolkadotConfig>` per chain.

```
Network (e.g., Polkadot)
├── relay:       wss://rpc.polkadot.io
├── collectives: wss://polkadot-collectives-rpc.polkadot.io
└── asset_hub:   wss://polkadot-asset-hub-rpc.polkadot.io
```

**Tools own their chain routing.** Domain-specific tools hardcode which chain they query. The AI never thinks about which RPC endpoint to hit.

| Tool | Chain(s) | Routing |
|---|---|---|
| `fellowship_status` | Collectives | Hardcoded: `self.pool.collectives()` |
| `referenda_active` | Relay | Hardcoded: `self.pool.relay()` |
| `account_info` | Relay (default) | Param override: `chain: "asset-hub"` |
| `total_balance` | Relay + Asset Hub | Parallel fan-out via `tokio::try_join!` |
| `query_storage` | Any | Explicit `chain` parameter required |

Each `ChainConfig` carries metadata (token symbol, decimals, SS58 prefix) so tools format output correctly regardless of network.

### Connection Lifecycle

1. Server starts → creates `ChainPool` with `Network` config (endpoints only, no connections yet)
2. First tool call touching relay → `pool.relay()` opens WebSocket, downloads metadata (~200KB–1MB), caches client
3. First tool call touching collectives → same lazy init
4. Subsequent calls → reuse cached `OnlineClient` (cheap clone via `Arc` internally)
5. Connection drops → `unstable-reconnecting-rpc-client` feature handles automatic reconnect

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

### Account & Balance

#### `account_info`
- **Params:** `address: String`, `chain: Option<String>` (default: relay)
- **Queries:** `System.Account`, `Balances.Locks`, `Balances.Freezes`, `Balances.Holds`
- **Returns:** Free, reserved, frozen, transferable balance. Lock breakdown with reasons.
- **Backend:** subxt

#### `account_balances`
- **Params:** `addresses: Vec<String>`, `chain: Option<String>` (default: relay, "all" for relay+asset-hub)
- **Queries:** Same as `account_info`, batched across addresses. Parallel via `join_all`.
- **Returns:** Balance summary per address. If chain="all", shows per-chain breakdown + total.
- **Backend:** subxt

#### `account_transfers`
- **Params:** `address: String`, `limit: Option<u32>` (default: 20)
- **Returns:** Recent transfers with timestamp, direction, counterparty, amount.
- **Backend:** Subscan API (`/api/v2/scan/transfers`)

#### `account_locks`
- **Params:** `address: String`
- **Queries:** `Balances.Locks`, `Balances.Freezes`, `ConvictionVoting.ClassLocksFor`, `Vesting.Vesting`
- **Returns:** Every lock/freeze with reason, amount, and unlock condition/time.
- **Backend:** subxt

#### `unlock_frozen`
- **Params:** `address: String`
- **Queries:** Checks all expired conviction locks and vesting schedules
- **Action:** Builds batch of `conviction_voting.unlock(class, target)` and/or `vesting.vest()` calls
- **Backend:** subxt (read + write)

### Fellowship (Polkadot Collectives chain)

#### `fellowship_status`
- **Params:** `address: String`
- **Queries (all on collectives chain):**
  1. `FellowshipCollective.Members[address]` → rank
  2. `CoreFellowship.Member[address]` → `{ is_active, last_promotion, last_proof }`
  3. `CoreFellowship.Params` → `{ demotion_period[], min_promotion_period }`
  4. `FellowshipSalary.Claimant[address]` → `{ last_active, status }`
  5. `FellowshipSalary.Status` → `{ cycle, cycle_start, budget }`
  6. Current block number (for time calculations)
- **Returns:** Rank, activity status, demotion risk (blocks until eligible for demotion), salary claim status, time since last promotion.
- **Backend:** subxt

#### `fellowship_salary`
- **Params:** `address: String`
- **Returns:** Whether salary is claimable this cycle, amount based on rank, claim instructions.
- **Logic:** Claimable if `last_active != current_cycle && status != Attempted`. Must `register()` then `claim()`.
- **Backend:** subxt

#### `fellowship_claim_salary`
- **Params:** `address: String`
- **Action:** Builds `salary.register()` + `salary.claim()` extrinsics (may need batch).
- **Backend:** subxt (write)

#### `fellowship_demotion_risk`
- **Params:** `address: String`
- **Returns:** Blocks remaining until demotion eligible. What qualifies as activity proof. How to submit evidence.
- **Logic:** `demotion_period[rank] - (current_block - last_proof)`. If negative, demotion is overdue.
- **Backend:** subxt

#### `fellowship_members`
- **Params:** `min_rank: Option<u16>` (default: 0)
- **Queries:** Iterate `FellowshipCollective.Members` entries, filter by rank.
- **Returns:** List of members with rank, address, identity (if available on relay chain).
- **Backend:** subxt

### Governance (OpenGov)

#### `referenda_active`
- **Params:** (none, or `track: Option<u16>` to filter)
- **Queries:** `Referenda.ReferendumCount`, then iterate recent `Referenda.ReferendumInfoFor`, filter for `Ongoing` variant.
- **Returns:** List of active referenda with: index, track name, proposer, tally (aye/nay/support %), status (preparing/deciding/confirming), time remaining.
- **Backend:** subxt

#### `referendum_detail`
- **Params:** `index: u32`
- **Queries:** `Referenda.ReferendumInfoFor[index]`
- **Returns:** Full detail: track, origin, proposal (decoded if possible), tally, timeline, deposit info.
- **Backend:** subxt

#### `vote`
- **Params:** `referendum_index: u32`, `aye: bool`, `conviction: u8` (0-6), `balance: u128`
- **Action:** Builds `ConvictionVoting.vote(poll_index, AccountVote::Standard { vote, balance })`. Dry-runs first.
- **Backend:** subxt (write)

#### `delegate`
- **Params:** `target: String`, `conviction: u8`, `balance: u128`, `track: Option<u16>` (None = all tracks)
- **Action:** Builds `ConvictionVoting.delegate(class, to, conviction, balance)` for specified or all tracks.
- **Backend:** subxt (write)

#### `my_votes`
- **Params:** `address: String`
- **Queries:** `ConvictionVoting.VotingFor[address][class]` for all 15 tracks, `ConvictionVoting.ClassLocksFor[address]`
- **Returns:** Active votes per track with conviction and lock expiry. Delegations. Total locked balance.
- **Backend:** subxt

### Staking

#### `staking_status`
- **Params:** `address: String`
- **Queries:** `Staking.Bonded[address]`, `Staking.Ledger[controller]`, `Staking.Nominators[address]`, `NominationPools.PoolMembers[address]`
- **Returns:** Bonded amount, active nominations, pending unbonds, pool membership if any.
- **Backend:** subxt

#### `staking_rewards`
- **Params:** `address: String`, `eras: Option<u32>` (default: 10)
- **Returns:** Pending unclaimed rewards + historical payouts per era.
- **Backend:** subxt for pending (`Staking.Ledger` claimed rewards), Subscan for history

#### `nomination_pools`
- **Params:** `address: String`
- **Queries:** `NominationPools.PoolMembers[address]`, `NominationPools.BondedPools[pool_id]`, `NominationPools.RewardPools[pool_id]`
- **Returns:** Pool name, your stake, pending rewards, pool state, member count.
- **Backend:** subxt

### Chain Utilities

#### `chain_info`
- **Params:** `chain: Option<String>` (default: relay)
- **Returns:** Chain name, runtime version, current block, current era, epoch, token info.
- **Backend:** subxt

#### `block_info`
- **Params:** `block: String` (number or hash), `chain: Option<String>`
- **Returns:** Block details with extrinsics and events decoded to human-readable.
- **Backend:** subxt

#### `decode_extrinsic`
- **Params:** `hex: String`, `chain: Option<String>`
- **Returns:** Human-readable decoding of the call data.
- **Backend:** subxt (metadata-based decoding)

#### `query_storage`
- **Params:** `chain: String`, `pallet: String`, `entry: String`, `keys: Option<Vec<String>>`
- **Returns:** Raw decoded value from any storage item on any chain.
- **Backend:** subxt

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
| `POLKADOT_NETWORK` | No | `polkadot` | Network preset: `polkadot`, `kusama`, `westend` |
| `POLKADOT_RELAY_URL` | No | (from preset) | Override relay chain RPC endpoint |
| `POLKADOT_COLLECTIVES_URL` | No | (from preset) | Override Collectives RPC endpoint |
| `POLKADOT_ASSET_HUB_URL` | No | (from preset) | Override Asset Hub RPC endpoint |
| `POLKADOT_SIGNER_URI` | No | (none, read-only) | Signer key URI. e.g. `//Alice` or mnemonic phrase |
| `SUBSCAN_API_KEY` | No | (none) | Subscan API key for historical data queries |

### MCP Client Configuration

```json
{
  "mcpServers": {
    "polkadot": {
      "command": "polkadot-mcp",
      "env": {
        "POLKADOT_NETWORK": "polkadot",
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
| Collectives | `wss://polkadot-collectives-rpc.polkadot.io` | DOT | 10 | 0 |
| Asset Hub | `wss://polkadot-asset-hub-rpc.polkadot.io` | DOT | 10 | 0 |

### Kusama

| Chain | Endpoint | Token | Decimals | SS58 |
|---|---|---|---|---|
| Relay | `wss://kusama-rpc.polkadot.io` | KSM | 12 | 2 |
| Collectives | `wss://kusama-collectives-rpc.polkadot.io` | KSM | 12 | 2 |
| Asset Hub | `wss://kusama-asset-hub-rpc.polkadot.io` | KSM | 12 | 2 |

### Westend (testnet)

| Chain | Endpoint | Token | Decimals | SS58 |
|---|---|---|---|---|
| Relay | `wss://westend-rpc.polkadot.io` | WND | 12 | 42 |
| Collectives | `wss://westend-collectives-rpc.polkadot.io` | WND | 12 | 42 |
| Asset Hub | `wss://westend-asset-hub-rpc.polkadot.io` | WND | 12 | 42 |
