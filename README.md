# polkadot-mcp

MCP server for AI-native interaction with Polkadot, Kusama, Westend, Paseo, and their system parachains.

Replace browser-based UIs with natural language. Ask your AI "what's my fellowship status?" or "why is my DOT frozen?" and get real answers backed by live on-chain data.

## Install

```bash
cargo install polkadot-mcp
```

Requires Rust. If you don't have it: [rustup.rs](https://rustup.rs)

## Setup

### Claude Desktop

Add to your config file:

- **macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows:** `%APPDATA%\Claude\claude_desktop_config.json`
- **Linux:** `~/.config/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "polkadot": {
      "command": "polkadot-mcp"
    }
  }
}
```

Restart Claude Desktop. Ask: *"What's the current block on Polkadot?"*

### Claude Code

```bash
claude mcp add polkadot-mcp -- polkadot-mcp
```

### Cursor / VS Code

Add to your MCP settings (`.cursor/mcp.json` or VS Code MCP config):

```json
{
  "mcpServers": {
    "polkadot": {
      "command": "polkadot-mcp"
    }
  }
}
```

### Any MCP Client

`polkadot-mcp` uses stdio transport. Point any MCP-compatible client at the binary with no arguments.

### Verify it works

```bash
# Interactive debugger — see tools, call them, inspect responses
npx @modelcontextprotocol/inspector polkadot-mcp
```

## Supported Networks & Chains

A single server instance has access to **all networks simultaneously**. Tools accept `network` and `chain` parameters to route queries.

| Network | Token | Chains |
|---|---|---|
| **Polkadot** | DOT | relay, asset-hub, bridge-hub, people, collectives, coretime |
| **Kusama** | KSM | relay, asset-hub, bridge-hub, people, coretime |
| **Westend** (testnet) | WND | relay, asset-hub, bridge-hub, people, collectives, coretime |
| **Paseo** (testnet) | PAS | relay, asset-hub, bridge-hub, people, collectives, coretime |

Note: Kusama does not have a Collectives chain.

## What Can It Do?

| Domain | Example Questions |
|---|---|
| **Account** | "What's my DOT balance?" · "Why is my balance frozen?" · "Show my recent transactions" |
| **Fellowship** | "Do I need to claim my salary?" · "Am I at risk of demotion?" · "What's my rank?" |
| **Governance** | "What's being voted on?" · "Vote aye on ref 1234" · "Who am I delegating to?" |
| **Staking** | "How much have I earned staking?" · "How's my nomination pool?" · "Claim my rewards" |

See [TOOLS.md](TOOLS.md) for the full tool catalog (~80 tools across 15 categories).

## Configuration

### Environment Variables

| Variable | Default | Description |
|---|---|---|
| `POLKADOT_SIGNER_URI` | *(none)* | Signer for transactions. Omit for read-only. |
| `SUBSCAN_API_KEY` | *(none)* | Subscan API key for historical data |

All networks are loaded at startup — no need for a network env var.

### Transaction Support

By default, the server runs in **read-only mode**. To enable transaction tools (vote, claim salary, unlock funds), set a signer:

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

## Architecture

Built on [subxt](https://github.com/paritytech/subxt) (dynamic mode) and [rmcp](https://crates.io/crates/rmcp) (Rust MCP SDK).

One server manages all networks simultaneously with a shared connection pool. Connections are created lazily on first use and cached. Tools accept `network` (polkadot/kusama/westend/paseo) and `chain` (relay/asset-hub/bridge-hub/people/collectives/coretime) parameters.

```
polkadot-mcp
├── PolkadotMcp        # Server: holds all networks + shared pool
├── networks/          # Polkadot, Kusama, Westend, Paseo presets
├── ChainPool          # Lazy-connected clients, cached by chain name
├── tools/
│   ├── account.rs     # Balance, locks, transfers
│   ├── fellowship.rs  # Rank, salary, demotion     → Collectives chain
│   ├── governance.rs  # Referenda, voting           → Relay chain
│   ├── staking.rs     # Staking, pools, rewards     → Relay chain
│   └── chain.rs       # Chain info, generic queries  → Any chain
└── backends/
    ├── subxt.rs       # Live chain state
    └── subscan.rs     # Historical/indexed data
```

## Development

```bash
cargo build                          # Build
cargo test                           # Test
cargo clippy -- -D warnings          # Lint
cargo run                            # Run (all networks available)

# Debug with MCP Inspector
npx @modelcontextprotocol/inspector cargo run
```

## License

Apache-2.0
