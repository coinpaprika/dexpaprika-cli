# DexPaprika Streaming — Real-Time Token Prices

Stream live crypto prices via SSE. ~1 second updates, 1–2,000 tokens per connection. No API key, no rate limits.

---

## Option A: CLI (Recommended)

Install (if you haven't already):

```bash
curl -sSL https://raw.githubusercontent.com/coinpaprika/dexpaprika-cli/main/install.sh | sh
```

Then stream:

### Single token

```bash
dexpaprika-cli stream ethereum 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2
```

### Multiple tokens

```bash
dexpaprika-cli stream ethereum --tokens 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2,0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48
```

### Limit events

```bash
dexpaprika-cli stream ethereum 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2 --limit 10
```

---

## Option B: Direct SSE API

Can't install binaries? Connect to the SSE endpoint directly.

**Base URL:** `https://streaming.dexpaprika.com`

### Single token (GET)

`GET /stream?method=t_p&chain={network}&address={token_address}`

```python
import requests, json

r = requests.get("https://streaming.dexpaprika.com/stream",
    params={"method": "t_p", "chain": "ethereum", "address": "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"},
    stream=True)

for line in r.iter_lines():
    if line and line.startswith(b'data:'):
        data = json.loads(line[5:])
        print(f"{data['c']} {data['a']}: ${data['p']}")
```

### Multiple tokens (POST)

`POST /stream` with `Accept: text/event-stream` and `Content-Type: application/json`.

Body: array of `{"chain", "address", "method": "t_p"}` objects (max 2,000).

**One invalid asset cancels the entire stream.** Validate with REST API `/search` first.

```python
import requests, json

assets = [
    {"chain": "ethereum", "address": "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2", "method": "t_p"},
    {"chain": "solana", "address": "So11111111111111111111111111111111111111112", "method": "t_p"}
]

r = requests.post("https://streaming.dexpaprika.com/stream",
    headers={"Accept": "text/event-stream", "Content-Type": "application/json"},
    json=assets, stream=True)

for line in r.iter_lines():
    if line and line.startswith(b'data:'):
        data = json.loads(line[5:])
        print(f"{data['c']} {data['a']}: ${data['p']}")
```

### Response format

Each SSE event:

```json
{"a": "0xc02a...", "c": "ethereum", "p": "2739.79", "t": 1769778188, "t_p": 1769778187}
```

| Field | Meaning                          |
| ----- | -------------------------------- |
| `a`   | Token address                    |
| `c`   | Chain ID                         |
| `p`   | Price USD (string for precision) |
| `t`   | Server timestamp (Unix)          |
| `t_p` | Price timestamp (Unix)           |

### Errors

`200` connected | `400` bad params / unsupported chain / asset not found | `429` capacity exceeded (retry with backoff)

---

## Best Practices

- Reconnect with exponential backoff on disconnect
- Use POST for 10+ assets (one connection vs many)
- Parse `p` as string/decimal to preserve precision
- Validate assets via REST `/search` before streaming

## Common Tokens

| Token | Chain    | Address                                      |
| ----- | -------- | -------------------------------------------- |
| WETH  | ethereum | `0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2` |
| USDC  | ethereum | `0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48` |
| SOL   | solana   | `So11111111111111111111111111111111111111112`  |

## Troubleshooting

- **This skill doesn't cover your use case?** Fetch the full docs at <https://docs.dexpaprika.com/streaming>
- **Need REST API (historical data, search, pools)?** See `https://dexpaprika.com/agents/skill.md`
- **Still stuck?** Contact support@coinpaprika.com
