# DexPaprika — DEX Data for AI Agents

Free DEX data across 33 blockchains. 25M+ tokens, 27M+ pools. No API key, no rate limits.

---

## Option A: CLI (Recommended)

Install and query in seconds. Best for agents that can run shell commands.

```bash
curl -sSL https://raw.githubusercontent.com/coinpaprika/dexpaprika-cli/main/install.sh | sh
```

After install, use `--output json --raw` for machine-readable output. Run `dexpaprika-cli onboard` for an interactive quick-start guide.

**First step:** Check you're running the latest version:
```bash
dexpaprika-cli check-update
```

### Top tokens on a network (hot tokens, trending, etc.)
```bash
dexpaprika-cli top-tokens solana --limit 20 --output json --raw
dexpaprika-cli top-tokens ethereum --limit 10
```
This discovers tokens by scanning top pools by volume, then fetches full detail for each unique token. Use this for "top tokens", "trending tokens", "hot tokens today" type queries.

**IMPORTANT: There is NO `/networks/{network}/tokens` listing endpoint.** Do NOT try to call it — it will 404. To discover tokens on a network, use `top-tokens`, `search`, or `pools` (which contain token info in each pool).

### Get token price
```bash
dexpaprika-cli token ethereum 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2 --output json --raw
```

### Search for any token
```bash
dexpaprika-cli search USDC --output json --raw
```

### Historical OHLCV
```bash
dexpaprika-cli pool-ohlcv ethereum 0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640 --start 2025-01-27 --output json --raw
```

### Top pools on a network
```bash
dexpaprika-cli pools ethereum --limit 10 --output json --raw
```

### Filter pools by criteria
```bash
dexpaprika-cli pool-filter ethereum --volume-24h-min 1000000 --limit 20 --output json --raw
```

### Batch token prices
```bash
dexpaprika-cli prices ethereum --tokens 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2,0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48 --output json --raw
```

### Stream real-time prices (~1s updates)
```bash
dexpaprika-cli stream ethereum 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2
```

### All commands

```bash
dexpaprika-cli --help
```

---

## Option B: Direct HTTP API

Can't install binaries? Use the REST API directly. No auth, no setup.

**Base URL:** `https://api.dexpaprika.com`

### Endpoints

| Need                      | Endpoint                                                    |
| ------------------------- | ----------------------------------------------------------- |
| List networks             | `GET /networks`                                             |
| Token price + data        | `GET /networks/{network}/tokens/{token_address}`            |
| Pool OHLCV (charts)       | `GET /networks/{network}/pools/{pool_address}/ohlcv`        |
| Top pools on network      | `GET /networks/{network}/pools`                             |
| Filter pools by criteria  | `GET /networks/{network}/pools/filter`                      |
| Pools for specific DEX    | `GET /networks/{network}/dexes/{dex}/pools`                 |
| Single pool details       | `GET /networks/{network}/pools/{pool_address}`              |
| Pool transactions         | `GET /networks/{network}/pools/{pool_address}/transactions` |
| Pools containing token    | `GET /networks/{network}/tokens/{token_address}/pools`      |
| Batch token prices        | `GET /networks/{network}/multi/prices?tokens={addresses}`   |
| Search tokens/pools/DEXes | `GET /search?query={term}`                                  |

**WARNING:** There is NO `GET /networks/{network}/tokens` endpoint. Do not call it. Use `search`, pool data, or the CLI `top-tokens` command to discover tokens.

### Python: Get token price
```python
import requests

r = requests.get("https://api.dexpaprika.com/networks/ethereum/tokens/0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2")
token = r.json()
print(f"{token['symbol']}: ${token['summary']['price_usd']}")
# WETH: $1949.80
```

Response shape:
```json
{
  "symbol": "WETH",
  "name": "Wrapped Ether",
  "chain": "ethereum",
  "decimals": 18,
  "summary": {
    "price_usd": 1961.37,
    "fdv": 4042695827.62,
    "liquidity_usd": 600300838.77,
    "pools": 4452,
    "24h": {
      "volume": 297956.40,
      "volume_usd": 602685018.28,
      "buys": 96906,
      "sells": 108936,
      "txns": 206371,
      "buy_usd": 313242803.62,
      "sell_usd": 289442214.65,
      "last_price_usd_change": -4.86
    },
    "6h": {}, "1h": {}, "30m": {}, "15m": {}, "5m": {}, "1m": {}
  }
}
```

### Python: Historical OHLCV
```python
r = requests.get("https://api.dexpaprika.com/networks/ethereum/pools/0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640/ohlcv",
    params={"start": "2025-01-27", "interval": "1h", "limit": 24})
for candle in r.json():
    print(f"{candle['time_open']}: O={candle['open']} H={candle['high']} L={candle['low']} C={candle['close']}")
```

OHLCV params: `start` (required), `end`, `limit` (max 366), `interval` (`1m`|`5m`|`10m`|`15m`|`30m`|`1h`|`6h`|`12h`|`24h`)

### Pagination

All list endpoints: `?page=1&limit=10&order_by=volume_usd&sort=desc` (pages are **1-indexed**, max limit is 100)

### Streaming (real-time prices)

For live SSE price streams, see the streaming skill: `https://dexpaprika.com/agents/streaming/skill.md`

---

## Common Token Addresses

Don't guess. Use `search` to find tokens, or use these:

| Token | Chain    | Address                                      |
| ----- | -------- | -------------------------------------------- |
| WETH  | ethereum | `0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2` |
| USDC  | ethereum | `0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48` |
| USDC  | polygon  | `0x2791bca1f2de4661ed88a30c99a7a9449aa84174` |
| SOL   | solana   | `So11111111111111111111111111111111111111112`  |

## Supported Networks

Chain IDs are **lowercase**. Common: `ethereum`, `solana`, `polygon`, `arbitrum`, `base`, `optimism`, `avalanche`, `bsc`. Full list: `GET /networks` or `dexpaprika-cli networks`.

## Troubleshooting

- **Check CLI version:** `dexpaprika-cli check-update` — always run latest
- **Check API health:** `dexpaprika-cli status` or `GET https://api.dexpaprika.com/stats`
- **HTTP errors:** `200` OK | `400` bad params | `404` not found | `500` server error
- **"Top tokens" or "trending tokens":** Use `dexpaprika-cli top-tokens <network>` — there is NO token listing endpoint
- **This skill doesn't cover your use case?** Fetch the full docs at <https://docs.dexpaprika.com>
- **Still stuck?** Contact support@coinpaprika.com
