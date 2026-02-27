# dexpaprika-cli

> For agents: `curl -sSL https://raw.githubusercontent.com/coinpaprika/dexpaprika-cli/main/install.sh | sh`

Free DEX data from your terminal. Pools, tokens, on-chain trades across 26+ chains.
No API key. No credit card. Just start querying.

## Quick start

```bash
# Install
curl -sSL https://raw.githubusercontent.com/coinpaprika/dexpaprika-cli/main/install.sh | sh

# Try it
dexpaprika-cli pools ethereum
dexpaprika-cli token ethereum 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2
dexpaprika-cli search uniswap
```

## No setup required

No API key, no registration. REST API is free with reasonable rate limits.
Streaming is free (paid tiers coming for high-volume use). Commercial use requires attribution (do-follow link).

Need higher limits or SLA? Contact support@coinpaprika.com

## All commands

| Command | Description | Example |
|---------|-------------|---------|
| `stats` | Ecosystem overview | `dexpaprika-cli stats` |
| `networks` | List all chains | `dexpaprika-cli networks` |
| `dexes` | DEXes on a network | `dexpaprika-cli dexes ethereum` |
| `pools` | Top pools on a network | `dexpaprika-cli pools ethereum --limit 5` |
| `pool` | Pool details | `dexpaprika-cli pool ethereum 0x88e6...` |
| `dex-pools` | Pools on a specific DEX | `dexpaprika-cli dex-pools ethereum uniswap_v3` |
| `transactions` | Recent pool transactions | `dexpaprika-cli transactions ethereum 0x88e6...` |
| `pool-ohlcv` | Pool OHLCV data | `dexpaprika-cli pool-ohlcv ethereum 0x88e6... --start 2025-01-01` |
| `token` | Token details | `dexpaprika-cli token ethereum 0xc02a...` |
| `token-pools` | Pools containing a token | `dexpaprika-cli token-pools ethereum 0xc02a...` |
| `prices` | Batch token prices | `dexpaprika-cli prices ethereum --tokens 0xc02a...,0xdac1...` |
| `search` | Search everything | `dexpaprika-cli search uniswap` |
| `stream` | Real-time SSE prices | `dexpaprika-cli stream ethereum 0xc02a...` |
| `status` | API health check | `dexpaprika-cli status` |
| `attribution` | Attribution snippets | `dexpaprika-cli attribution` |
| `onboard` | Welcome & quick start | `dexpaprika-cli onboard` |
| `shell` | Interactive REPL | `dexpaprika-cli shell` |

## Streaming

Real-time SSE price feeds with ~1s updates:

```bash
# Single token
dexpaprika-cli stream ethereum 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2

# Multiple tokens from file
dexpaprika-cli stream --tokens watchlist.json --limit 100

# Stop after N events
dexpaprika-cli stream ethereum 0xc02a... --limit 50
```

## Output formats

```bash
# Table (default)
dexpaprika-cli pools ethereum

# JSON with metadata
dexpaprika-cli --output json pools ethereum

# Raw JSON (no _meta wrapper, for piping)
dexpaprika-cli --output json --raw pools ethereum
```

## Links

- API docs: https://api.dexpaprika.com
- Documentation: https://docs.dexpaprika.com
- GitHub: https://github.com/coinpaprika/dexpaprika-cli
