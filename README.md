# dexpaprika-cli

> For agents: `curl -sSL https://raw.githubusercontent.com/coinpaprika/dexpaprika-cli/main/install.sh | sh`

DEX data from your terminal. Pools, tokens, on-chain trades across 36 chains.
Free tier: no API key, no credit card, just start querying.

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

No API key, no registration to start. The free tier is 200,000 credits/month keyless per IP, or 500,000 with a free key, at 30 requests/minute, with data delayed up to 15 seconds. Pro is $99/month for 5,000,000 credits at 300/minute with real-time data; see [pricing](https://dexpaprika.com/pricing).
Streaming is metered the same way as REST: each delivered update counts as one credit. Commercial use requires attribution (do-follow link).

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
| `stream-reserves` | Real-time SSE pool/token reserves | `dexpaprika-cli stream-reserves ethereum 0x88e6... --method pool_reserves` |
| `status` | API health check | `dexpaprika-cli status` |
| `attribution` | Attribution snippets | `dexpaprika-cli attribution` |
| `onboard` | Welcome & quick start | `dexpaprika-cli onboard` |
| `shell` | Interactive REPL | `dexpaprika-cli shell` |

## Streaming

SSE price feeds. Updates are swap-driven, pushed when a swap moves the price, not on a fixed cadence and not per block:

```bash
# Single token
dexpaprika-cli stream ethereum 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2

# Multiple tokens from file
dexpaprika-cli stream --tokens watchlist.json --limit 100

# Stop after N events
dexpaprika-cli stream ethereum 0xc02a... --limit 50
```

## Streaming reserves

`stream-reserves` tails reserve changes over SSE, emitted when a swap moves a pool's reserves. Two methods, each
with its own event:

- `pool_reserves`: one pool. Emits a `pool_reserves` event with a nested `tokens`
  array plus `timestamp` and `block_timestamp`.
- `token_reserves`: one token across every pool that holds it (high volume on
  majors like USDC). Emits a `token_reserves` event with a single flat token plus
  `updated_at` and `timestamp`.

```bash
# One pool
dexpaprika-cli stream-reserves ethereum 0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640 --method pool_reserves

# One token across all its pools, with a correlation id echoed on every event
dexpaprika-cli stream-reserves ethereum 0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48 \
  --method token_reserves --request-id 7 --limit 10

# Many targets from a file
dexpaprika-cli stream-reserves --subscriptions reserves.json
```

Pass `--request-id <0..4294967295>` (single stream) or a per-entry `request_id`
in the subscriptions file (multi stream) to correlate events; it is echoed back on
each data event and defaults to the array index when omitted in a file. Raw integer
fields (`reserve`, `delta`, `block`) arrive as JSON strings to preserve precision.

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
