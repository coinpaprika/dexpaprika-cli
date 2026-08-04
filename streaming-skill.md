# DexPaprika Streaming Skill (CLI)

Two SSE feeds, one transport. Free tier, no API key needed to start.

- `dexpaprika-cli stream …` for live token prices (`/sse/prices`).
- `dexpaprika-cli stream-reserves …` for pool reserves (`/sse/reserves`), emitted when a swap moves a pool's reserves.

**Limits:** 25 subscriptions per POST connection. 10 concurrent SSE streams per IP. A `ping` event lands every 15s.

---

## Option A: CLI (Recommended)

Install:

```bash
curl -sSL https://raw.githubusercontent.com/coinpaprika/dexpaprika-cli/main/install.sh | sh
```

### Stream prices

```bash
# Single token
dexpaprika-cli stream ethereum 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2

# Multiple tokens via watchlist file (JSON array, up to 25 entries)
dexpaprika-cli stream --tokens watchlist.json

# Cap event count for smoke-tests
dexpaprika-cli stream ethereum 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2 --limit 10

# JSON output for piping
dexpaprika-cli -o json stream ethereum 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2 --limit 5
```

Watchlist file format:

```json
[
  {"chain": "ethereum", "address": "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"},
  {"chain": "solana",   "address": "So11111111111111111111111111111111111111112"}
]
```

### Stream reserves

```bash
# Single pool: fires on every reserve change in that pool
dexpaprika-cli stream-reserves ethereum 0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640 --method pool_reserves

# Single token: fires for every pool containing that token (high volume on USDC etc.)
dexpaprika-cli stream-reserves ethereum 0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48 --method token_reserves --limit 10

# Multi-target via subscriptions file (up to 25 entries, can mix methods)
dexpaprika-cli stream-reserves --subscriptions reserves.json
```

Subscriptions file format:

```json
[
  {"chain": "ethereum", "address": "0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640", "method": "pool_reserves"},
  {"chain": "ethereum", "address": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48", "method": "token_reserves"}
]
```

---

## Option B: Direct SSE API

Can't install binaries? Connect to the SSE endpoints directly.

**Base URL:** `https://streaming.dexpaprika.com`

### Stream prices, single token (GET)

```
GET /sse/prices?method=token_price&chain={network}&address={token_address}
```

```python
import json, requests

url = "https://streaming.dexpaprika.com/sse/prices"
params = {"method": "token_price", "chain": "ethereum",
          "address": "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"}

with requests.get(url, params=params, stream=True) as r:
    r.raise_for_status()
    msg_lines = []
    for line in r.iter_lines(decode_unicode=True):
        if line:
            msg_lines.append(line); continue
        # Blank line: dispatch the buffered message.
        event_type, data_str = "message", None
        for ml in msg_lines:
            if ml.startswith("event:"):
                event_type = ml.split(":", 1)[1].strip()
            elif ml.startswith("data:"):
                data_str = ml[5:].lstrip()
        msg_lines = []
        if event_type == "token_price" and data_str:
            d = json.loads(data_str)
            print(f"{d['chain']} {d['address']}: ${d['price']}")
```

### Stream prices, multiple tokens (POST)

```
POST /sse/prices
Accept: text/event-stream
Content-Type: application/json
```

Body: JSON array of `{"chain", "address", "method": "token_price"}` objects, max 25 per connection.

```python
import json, requests

assets = [
  {"chain": "ethereum", "address": "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2", "method": "token_price"},
  {"chain": "solana",   "address": "So11111111111111111111111111111111111111112",  "method": "token_price"},
]

r = requests.post("https://streaming.dexpaprika.com/sse/prices",
    headers={"Accept": "text/event-stream", "Content-Type": "application/json"},
    json=assets, stream=True)
# Same buffer-by-blank-line parser as above; filter on event_type == "token_price".
```

One invalid asset cancels the entire stream with HTTP 400. Validate addresses with REST `/search` first.

### Stream reserves (GET, single)

```
GET /sse/reserves?method=pool_reserves&chain={network}&address={pool_address}
GET /sse/reserves?method=token_reserves&chain={network}&address={token_address}
```

### Stream reserves (POST, multi)

```
POST /sse/reserves
```

Body: JSON array of `{"chain", "address", "method": "pool_reserves"|"token_reserves"}` objects, max 25 per connection. Methods can be mixed.

---

## Event types

| Event | Where | Payload shape |
|---|---|---|
| `token_price` | prices feed | `{address, chain, price, timestamp, timestamp_price, token_price}` |
| `pool_reserves` | reserves feed | `{chain, pool_id, block, previous_block, tokens[], total_reserve_usd, total_delta_usd, timestamp, block_timestamp}` |
| `token_reserves` | reserves feed | `{chain, token_id, reserve, delta, block, price_usd, reserve_usd, delta_usd, updated_at, timestamp}` |
| `ping` | both | `{"time": <unix>}` (every ~15s) |
| `warning` | both | `{"message": "..."}` (non-fatal, e.g. deprecation) |
| `error` | both | `{"message": "..."}` (stream-terminating) |

`token_price` payload (JSON):
```json
{
  "address": "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
  "chain": "ethereum",
  "price": "2145.78",
  "timestamp": 1779110450,
  "timestamp_price": 1779110449,
  "token_price": 1779110449
}
```

`pool_reserves` payload (JSON):
```json
{
  "chain": "ethereum",
  "pool_id": "0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640",
  "block": "25122344",
  "tokens": [
    {"token_id": "0xa0b8...", "reserve": "61995526164300", "delta": "103817802",
     "price_usd": 1.0000, "reserve_usd": 61998696.43, "delta_usd": 103.82},
    {"token_id": "0xc02a...", "reserve": "17706554631959222896552", "delta": "-48362919581328866",
     "price_usd": 2145.78, "reserve_usd": 37994383.59, "delta_usd": -103.77}
  ],
  "total_reserve_usd": 99993080.03,
  "total_delta_usd": 0.04,
  "timestamp": 1779110450,
  "block_timestamp": 1779110447
}
```

`token_reserves` payload (JSON), flat and single-token, with `updated_at` instead of `block_timestamp`:
```json
{
  "chain": "ethereum",
  "token_id": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
  "reserve": "1602038276073898",
  "delta": "73537469563",
  "block": "25236698",
  "price_usd": 1.00009,
  "reserve_usd": 1602184632.13,
  "delta_usd": 73544.18,
  "updated_at": 1780487699,
  "timestamp": 1780487701
}
```

The legacy single `reserve_update` event no longer exists. A consumer that matched it must switch to `pool_reserves` and `token_reserves`. The legacy `t_p` event and compact `{a, c, p, t, t_p}` shape exist on the deprecated `/stream` path only. New code should not use them.

---

## Wire-format gotchas

- `block`, `previous_block`, `reserve`, `delta` arrive as **JSON strings**. They routinely exceed `Number.MAX_SAFE_INTEGER`. Parse with `BigInt` for arithmetic, or `Number()` for display-only. The USD fields are regular JSON numbers.
- Both orderings of `event:` and `data:` lines within one SSE message are valid per the spec, and the server has used both during rollout. Custom parsers should buffer one message between blank-line boundaries before dispatching, not dispatch on field order.
- `previous_block` is omitted from the payload on the first event after subscribing.

---

## Errors

| Code | Cause | Body |
|---|---|---|
| 200 | Connected, streaming | (SSE event stream) |
| 400 | Bad params, unsupported chain, asset not found, one invalid asset in a batch | `{"message": "..."}` |
| 400 | Too many entries in POST body | `{"message":"too many assets, max 25 allowed"}` for `/sse/prices`; `{"message":"too many subscriptions"}` for `/sse/reserves` |
| 429 | IP stream limit (10 concurrent) | `{"message":"ip stream limit exceeded"}` |

In-stream errors arrive as `event: error` SSE messages and terminate the stream.

---

## Best practices

- Reconnect with exponential backoff on disconnect.
- Use POST for multi-asset subscriptions (one connection instead of many), up to 25 entries.
- Parse `price` as a string for decimal precision. Don't `parseFloat` and re-serialize.
- Filter on the `event:` line. Treat unknown events as no-ops so future server-side additions don't break the handler.
- Use `BigInt` (or `int`) for raw integer fields when you need arithmetic.
- Open parallel connections if you need more than 25 subscriptions, up to the 10/IP cap.
- Validate addresses via REST `/search` before streaming.

---

## Common identifiers

| Token | Chain    | Address |
| ----- | -------- | ------- |
| WETH  | ethereum | `0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2` |
| USDC  | ethereum | `0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48` |
| USDT  | ethereum | `0xdac17f958d2ee523a2206206994597c13d831ec7` |
| WBTC  | ethereum | `0x2260fac5e5542a773aa44fbcfedf7c193bc2c599` |
| SOL   | solana   | `So11111111111111111111111111111111111111112` |
| USDC  | solana   | `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` |
| WBNB  | bsc      | `0xbb4cdb9cbd36b01bd1cbaebf2de08d9173bc095c` |

Common pools (for reserves streaming):

| Pair | Chain | DEX | Pool address |
|---|---|---|---|
| USDC/WETH (0.05%) | ethereum | Uniswap V3 | `0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640` |
| USDC/WETH (0.30%) | ethereum | Uniswap V3 | `0xe0554a476a092703abdb3ef35c80e0d76d32939f` |

Find others via REST API `/networks/{chain}/pools/search?token_address={address}`.

---

## Deprecated paths

`/stream` and `/reserves/stream` are predecessors. `/stream` still works but emits a one-shot `warning` event on connect telling clients to migrate to `/sse/prices`. `/reserves/stream` was retired and now returns 404. New code must use `/sse/prices` and `/sse/reserves`.

---

## Troubleshooting

- **This skill doesn't cover your case?** Full docs at <https://docs.dexpaprika.com/streaming/introduction>.
- **Reserves deep dive?** <https://docs.dexpaprika.com/streaming/reserves-streaming>.
- **Need REST (historical data, search, pools)?** See `https://dexpaprika.com/agents/skill.md`.
- **Still stuck?** Contact support@coinpaprika.com.
