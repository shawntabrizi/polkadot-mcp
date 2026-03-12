# polkadot-mcp

MCP server for AI-native interaction with Polkadot, Kusama, and Substrate chains.

Replace browser-based UIs with natural language. Ask your AI "what's my fellowship status?" or "why is my DOT frozen?" and get real answers backed by live on-chain data.

## Quick Start

```bash
# Build
cargo build --release

# Add to Claude Desktop config (~/.config/claude/claude_desktop_config.json)
```

```json
{
  "mcpServers": {
    "polkadot": {
      "command": "/path/to/polkadot-mcp"
    }
  }
}
```

Restart Claude Desktop. Ask: *"What's the balance of 5GrwvaEF..."*

## What Can It Do?

| Domain | Example Questions |
|---|---|
| **Account** | "What's my DOT balance?" · "Why is my balance frozen?" · "Show my recent transactions" |
| **Fellowship** | "Do I need to claim my salary?" · "Am I at risk of demotion?" · "What's my rank?" |
| **Governance** | "What's being voted on?" · "Vote aye on ref 1234" · "Who am I delegating to?" |
| **Staking** | "How much have I earned staking?" · "How's my nomination pool?" · "Claim my rewards" |

## Configuration

### Environment Variables

| Variable | Default | Description |
|---|---|---|
| `POLKADOT_NETWORK` | `polkadot` | Network: `polkadot`, `kusama`, `westend` |
| `POLKADOT_SIGNER_URI` | *(none)* | Signer for transactions. Omit for read-only. |
| `SUBSCAN_API_KEY` | *(none)* | Subscan API key for historical data |

### Transaction Support

By default, the server runs in **read-only mode**. To enable transaction tools (vote, claim salary, unlock funds), set a signer:

```json
{
  "mcpServers": {
    "polkadot": {
      "command": "/path/to/polkadot-mcp",
      "env": {
        "POLKADOT_SIGNER_URI": "bottom drive obey lake curtain smoke basket hold race lonely fit walk//polkadot"
      }
    }
  }
}
```

### Multiple Networks

Run separate instances for Polkadot and Kusama:

```json
{
  "mcpServers": {
    "polkadot": {
      "command": "/path/to/polkadot-mcp",
      "env": { "POLKADOT_NETWORK": "polkadot" }
    },
    "kusama": {
      "command": "/path/to/polkadot-mcp",
      "env": { "POLKADOT_NETWORK": "kusama" }
    }
  }
}
```

## Architecture

Built on [subxt](https://github.com/paritytech/subxt) (dynamic mode) and [rmcp](https://crates.io/crates/rmcp) (Anthropic's Rust MCP SDK).

One server connects to multiple chains simultaneously (relay, Collectives, Asset Hub). Tools automatically route to the right chain — the AI never needs to think about which RPC endpoint to use.

```
polkadot-mcp
├── ChainPool          # Lazy-connected clients for relay, collectives, asset-hub
├── tools/
│   ├── account.rs     # Balance, locks, transfers
│   ├── fellowship.rs  # Rank, salary, demotion     → Collectives chain
│   ├── governance.rs  # Referenda, voting           → Relay chain
│   ├── staking.rs     # Staking, pools, rewards     → Relay chain
│   └── chain.rs       # Generic queries             → Any chain
└── backends/
    ├── subxt.rs       # Live chain state
    └── subscan.rs     # Historical/indexed data
```

## Development

```bash
cargo build                          # Build
cargo test                           # Test
cargo clippy -- -D warnings          # Lint
POLKADOT_NETWORK=westend cargo run   # Run against testnet

# Debug with MCP Inspector
npx @anthropic-ai/mcp-inspector cargo run
```

## License

Apache-2.0
