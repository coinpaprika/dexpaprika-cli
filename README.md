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

No API key, no registration to start. The free tier is keyless, with data delayed up to 15 seconds, and a free key raises both the monthly quota and the per-minute rate. Pro is $99/month at 300/minute with real-time data. Monthly quotas change, so read the current figures from [pricing](https://dexpaprika.com/api/pricing).
Streaming is metered the same way as REST: each delivered update counts as one credit. Commercial use requires attribution (do-follow link).

Need higher limits or SLA? Contact support@coinpaprika.com

## Optional API key

**The CLI works without a key and always will.** Nothing above needs one.

A free key raises the monthly credit allowance. It does **not** raise the
per-minute limit, which is the same on both free tiers, so reach for one when you
are running out of monthly credits rather than hitting rate limits. Current
figures: [rate limits](https://docs.dexpaprika.com/knowledge-base/rate-limits).

```bash
dexpaprika-cli config set-key api_YOUR_KEY   # validates against the API, then stores it
dexpaprika-cli config show                   # which key is in use and what the API makes of it
dexpaprika-cli config delete                 # forget it and go back to keyless
```

Or set `DEXPAPRIKA_API_KEY`, or pass `--api-key` on any command. Precedence is
flag, then environment, then the stored config, then keyless.

**Paste the key on its own. There is no `Bearer` prefix**, and no other scheme
word: the API checksums the raw header, so a scheme word returns 401.

`config set-key` checks the key against `/usage` before storing it, and refuses
to save one the API rejects. That is deliberate: on the data endpoints a key the
API cannot read is ignored rather than rejected, returning `200` with real data
while quietly serving you the keyless tier, so a broken key otherwise looks
exactly like a working one. `/usage` is the only endpoint that reports the truth.

The stored file is `~/.dexpaprika/config.json`, created `0600` in a `0700`
directory.

## All commands

| Command | Description | Example |
|---------|-------------|---------|
| `stats` | Ecosystem overview | `dexpaprika-cli stats` |
| `networks` | List all chains | `dexpaprika-cli networks` |
| `dexes` | DEXes on a network | `dexpaprika-cli dexes ethereum` |
| `pools` | Top pools on a network | `dexpaprika-cli pools ethereum --limit 5` |
| `pool-filter` | Filter pools by volume, liquidity, txns, price change | `dexpaprika-cli pool-filter ethereum --price-change-24h-max -20` |
| `pool` | Pool details | `dexpaprika-cli pool ethereum 0x88e6...` |
| `dex-pools` | Pools on a specific DEX | `dexpaprika-cli dex-pools ethereum uniswap_v3 --limit 5` |
| `transactions` | Recent pool transactions | `dexpaprika-cli transactions ethereum 0x88e6...` |
| `pool-ohlcv` | Pool OHLCV data | `dexpaprika-cli pool-ohlcv ethereum 0x88e6... --start 2025-01-01` |
| `token` | Token details | `dexpaprika-cli token ethereum 0xc02a...` |
| `token-pools` | Pools containing a token | `dexpaprika-cli token-pools ethereum 0xc02a...` |
| `prices` | Batch token prices | `dexpaprika-cli prices ethereum --tokens 0xc02a...,0xdac1...` |
| `search` | Search everything | `dexpaprika-cli search uniswap` |
| `stream` | SSE token price stream | `dexpaprika-cli stream ethereum 0xc02a...` |
| `stream-reserves` | SSE pool/token reserve stream | `dexpaprika-cli stream-reserves ethereum 0x88e6... --method pool_reserves` |
| `status` | API health check | `dexpaprika-cli status` |
| `attribution` | Attribution snippets | `dexpaprika-cli attribution` |
| `onboard` | Welcome & quick start | `dexpaprika-cli onboard` |
| `shell` | Interactive REPL | `dexpaprika-cli shell` |

## DEX pools

`dex-pools` lists the pools of one DEX. The DEX id is the positional argument, exactly as
before, and you can get the valid ids from `dexpaprika-cli dexes <network>`. Pass the id
column, which is matched case-insensitively. Passing a display name like "Uniswap V3"
returns an empty list rather than an error, so an empty result usually means a name went in
where an id belonged.

```bash
# First page
dexpaprika-cli dex-pools ethereum uniswap_v3 --limit 5

# Next page: pass the next_cursor printed under the table
dexpaprika-cli dex-pools ethereum uniswap_v3 --limit 5 --cursor eyJjaGFpbiI6ImV0aGVyZXVtIi...
```

Results are cursor-paginated, so there is no `--page` flag on this command. The table prints
`next_cursor` when more results are available; feed it back through `--cursor`.

## Price change windows

`pool-filter` bounds four price-change windows, and both `pools` and `pool-filter` can sort by any of them. Values are percentages, so a max of -20 reads as "down 20% or more":

```bash
# Pools down 20% or more over 24h
dexpaprika-cli pool-filter ethereum --price-change-24h-max -20 --limit 5

# Pools up 50% or more in the last hour, sorted by that same hour
dexpaprika-cli pool-filter ethereum --price-change-1h-min 50 --sort-by price_change_percentage_1h

# The short windows on the sort side
dexpaprika-cli --output json pools ethereum --order-by price_change_percentage_5m --sort desc
```

The eight bounds are `--price-change-{24h,6h,1h,5m}-{min,max}`. Tables carry the 24h change, so ask for `--output json` when you want the 6h, 1h and 5m numbers back.

Only the 6h, 1h and 5m windows are pools-only. The token endpoint rejects those three as sort fields and quietly ignores them as bounds, which is the nastier half: an ignored bound comes back `200` with the unfiltered page. The 24h window works on both sides, so `top-tokens` sorts by it and `filter-tokens` takes `--price-change-24h-min` and `--price-change-24h-max`.

## Streaming

SSE price feeds. Updates are swap-driven, pushed when a swap moves the price, not on a fixed cadence and not per block.

Keyless streaming covers 36 showcase tokens, one per chain. A free API key opens streaming for any token. Either way you get up to 10 concurrent streams per IP and 25 subscriptions per POST connection. If a keyless stream connects but only ever delivers `ping` frames, the token you asked for is not one of the showcase 36:

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
