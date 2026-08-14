mod client;
mod commands;
mod output;
mod shell;

use clap::{Parser, Subcommand};
use commands::pools::PriceChangeBounds;
use output::OutputFormat;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "dexpaprika-cli",
    version,
    about = "dexpaprika-cli: DEX data from your terminal",
    long_about = "dexpaprika-cli: DEX data from your terminal\n\n\
                   Pools · Tokens · On-chain trades · 36 chains · Streaming\n\n\
                   REST API: no API key needed to start\n\
                   Streaming: metered like REST, one update = one credit\n\
                   Plans: https://dexpaprika.com/pricing\n\n\
                   Quick start:  dexpaprika-cli onboard\n\
                   API docs:     https://api.dexpaprika.com\n\
                   Docs:         https://docs.dexpaprika.com\n\
                   Enterprise:   support@coinpaprika.com"
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format: table or json
    #[arg(short, long, global = true, default_value = "table")]
    pub(crate) output: OutputFormat,

    /// JSON output without _meta wrapper (for scripts/piping)
    #[arg(long, global = true, default_value = "false")]
    pub(crate) raw: bool,
}

/// Parse a percentage bound and refuse the values f64 accepts but the API does
/// not. "nan" and "inf" parse happily into f64 and go out on the wire as NaN and
/// inf, where pools/search answers 500. The CLI turns any 5xx into "DexPaprika
/// API is temporarily unavailable", so bad input arrives at the caller dressed
/// as an outage. Checked on 2026-08-07: `price_change_percentage_24h_min=NaN`
/// and `=inf` both return 500, while `=abc` correctly returns 400.
fn finite_percent(raw: &str) -> Result<f64, String> {
    let value: f64 = raw
        .parse()
        .map_err(|_| format!("`{raw}` is not a number"))?;
    if !value.is_finite() {
        return Err(format!(
            "`{raw}` is not a finite percentage. NaN and infinity are not bounds the API accepts"
        ));
    }
    Ok(value)
}

#[derive(Subcommand)]
enum Commands {
    /// DexPaprika global stats (networks, DEXes, pools, tokens counts)
    Stats,

    /// List all supported networks/chains
    #[command(after_help = "EXAMPLES:\n  dexpaprika-cli networks")]
    Networks,

    /// List DEXes on a network
    #[command(after_help = "EXAMPLES:\n  dexpaprika-cli dexes ethereum --limit 10")]
    Dexes {
        /// Network ID (e.g., ethereum, solana, bsc)
        network: String,
        /// Maximum number of results (max 100)
        #[arg(long, default_value = "10")]
        limit: usize,
        /// Page number (1-indexed)
        #[arg(long, default_value = "1")]
        page: usize,
    },

    /// List top pools on a network
    #[command(
        after_help = "EXAMPLES:\n  dexpaprika-cli pools ethereum --limit 5\n  dexpaprika-cli pools solana --order-by volume_usd_24h --sort desc\n  dexpaprika-cli --output json pools ethereum --order-by price_change_percentage_5m --sort desc\n\nThe table shows the 24h change. Sort by the 5m, 1h or 6h window and the values\nyou sorted on are in --output json, so that example asks for JSON."
    )]
    Pools {
        /// Network ID (e.g., ethereum, solana)
        network: String,
        /// Maximum number of results (max 100)
        #[arg(long, default_value = "10")]
        limit: usize,
        /// Page number (1-indexed)
        #[arg(long, default_value = "1")]
        page: usize,
        /// Order by field: volume_usd_24h, volume_usd_7d, volume_usd_30d, liquidity_usd,
        /// txns_24h, created_at, price_usd, price_change_percentage_{24h,6h,1h,5m}
        #[arg(long, default_value = "volume_usd_24h")]
        order_by: String,
        /// Sort order
        #[arg(long, default_value = "desc")]
        sort: String,
    },

    /// Filter pools by volume, liquidity, transactions, price change, and creation date
    #[command(
        name = "pool-filter",
        after_help = "EXAMPLES:\n  dexpaprika-cli pool-filter ethereum --volume-24h-min 100000\n  dexpaprika-cli pool-filter solana --liquidity-usd-min 50000 --sort-by liquidity\n  dexpaprika-cli pool-filter ethereum --price-change-1h-min 50 --sort-by price_change_percentage_1h\n  dexpaprika-cli pool-filter ethereum --price-change-24h-max -20\n\nPRICE CHANGE BOUNDS:\n  Percentages, and negative values are the point: --price-change-24h-max -20 means\n  down 20% or more over 24h. Write -0.5 rather than -.5, which clap reads as a flag.\n  The 6h, 1h and 5m windows exist for pools only.\n  The table shows the 24h change; all four windows come back in --output json."
    )]
    PoolFilter {
        /// Network ID (e.g., ethereum, solana)
        network: String,
        /// Minimum 24h volume in USD
        #[arg(long)]
        volume_24h_min: Option<f64>,
        /// Maximum 24h volume in USD
        #[arg(long)]
        volume_24h_max: Option<f64>,
        /// Minimum 7d volume in USD
        #[arg(long)]
        volume_7d_min: Option<f64>,
        /// Maximum 7d volume in USD
        #[arg(long)]
        volume_7d_max: Option<f64>,
        /// Minimum pool liquidity in USD
        #[arg(long)]
        liquidity_usd_min: Option<f64>,
        /// Maximum pool liquidity in USD
        #[arg(long)]
        liquidity_usd_max: Option<f64>,
        /// Minimum transactions in 24h
        #[arg(long)]
        txns_24h_min: Option<u64>,
        /// Minimum 24h price change, in percent (negative allowed)
        #[arg(long, allow_negative_numbers = true, value_parser = finite_percent)]
        price_change_24h_min: Option<f64>,
        /// Maximum 24h price change, in percent (negative allowed)
        #[arg(long, allow_negative_numbers = true, value_parser = finite_percent)]
        price_change_24h_max: Option<f64>,
        /// Minimum 6h price change, in percent (negative allowed)
        #[arg(long, allow_negative_numbers = true, value_parser = finite_percent)]
        price_change_6h_min: Option<f64>,
        /// Maximum 6h price change, in percent (negative allowed)
        #[arg(long, allow_negative_numbers = true, value_parser = finite_percent)]
        price_change_6h_max: Option<f64>,
        /// Minimum 1h price change, in percent (negative allowed)
        #[arg(long, allow_negative_numbers = true, value_parser = finite_percent)]
        price_change_1h_min: Option<f64>,
        /// Maximum 1h price change, in percent (negative allowed)
        #[arg(long, allow_negative_numbers = true, value_parser = finite_percent)]
        price_change_1h_max: Option<f64>,
        /// Minimum 5m price change, in percent (negative allowed)
        #[arg(long, allow_negative_numbers = true, value_parser = finite_percent)]
        price_change_5m_min: Option<f64>,
        /// Maximum 5m price change, in percent (negative allowed)
        #[arg(long, allow_negative_numbers = true, value_parser = finite_percent)]
        price_change_5m_max: Option<f64>,
        /// Only pools created after this UNIX timestamp
        #[arg(long)]
        created_after: Option<u64>,
        /// Only pools created before this UNIX timestamp
        #[arg(long)]
        created_before: Option<u64>,
        /// Sort by field: volume_24h, volume_7d, volume_30d, liquidity, txns_24h, created_at,
        /// price_usd, price_change_percentage_24h, price_change_percentage_6h,
        /// price_change_percentage_1h, price_change_percentage_5m
        #[arg(long, default_value = "volume_24h")]
        sort_by: String,
        /// Sort direction: asc or desc
        #[arg(long, default_value = "desc")]
        sort_dir: String,
        /// Maximum number of results (max 100)
        #[arg(long, default_value = "50")]
        limit: usize,
        /// Page number (1-indexed)
        #[arg(long, default_value = "1")]
        page: usize,
    },

    /// Get detailed info about a specific pool
    #[command(
        after_help = "EXAMPLES:\n  dexpaprika-cli pool ethereum 0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640"
    )]
    Pool {
        /// Network ID
        network: String,
        /// Pool contract address
        pool_address: String,
        /// Invert the price ratio
        #[arg(long)]
        inversed: bool,
    },

    /// List pools on a specific DEX
    #[command(
        name = "dex-pools",
        after_help = "EXAMPLES:\n  dexpaprika-cli dex-pools ethereum uniswap_v3 --limit 5\n  dexpaprika-cli dex-pools ethereum curve --limit 5 --cursor eyJjaGFpbiI6...\n\nNOTE: results are cursor-paginated. Pass the next_cursor printed under the\ntable to --cursor to fetch the next page."
    )]
    DexPools {
        /// Network ID
        network: String,
        /// DEX identifier from `dexpaprika-cli dexes <network>` (e.g., uniswap_v3, curve).
        /// This is the id column, case-insensitive. A display name like "Uniswap V3"
        /// returns no pools instead of an error.
        dex: String,
        /// Maximum number of results (max 100)
        #[arg(long, default_value = "10")]
        limit: usize,
        /// Cursor for the next page, taken from the previous response
        #[arg(long)]
        cursor: Option<String>,
        /// Order by field
        #[arg(long, default_value = "volume_usd")]
        order_by: String,
        /// Sort order
        #[arg(long, default_value = "desc")]
        sort: String,
    },

    /// Get recent transactions for a pool
    #[command(
        after_help = "EXAMPLES:\n  dexpaprika-cli transactions ethereum 0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640 --limit 20\n  dexpaprika-cli transactions ethereum 0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640 --from 1712700000 --to 1712800000"
    )]
    Transactions {
        /// Network ID
        network: String,
        /// Pool contract address
        pool_address: String,
        /// Maximum number of results
        #[arg(long, default_value = "10")]
        limit: usize,
        /// Cursor for pagination
        #[arg(long)]
        cursor: Option<String>,
        /// Filter transactions starting from this UNIX timestamp (inclusive, max 7 days)
        #[arg(long)]
        from: Option<i64>,
        /// Filter transactions up to this UNIX timestamp (exclusive)
        #[arg(long)]
        to: Option<i64>,
    },

    /// Get OHLCV data for a pool
    #[command(
        name = "pool-ohlcv",
        after_help = "EXAMPLES:\n  dexpaprika-cli pool-ohlcv ethereum 0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640 --start 2025-01-01"
    )]
    PoolOhlcv {
        /// Network ID
        network: String,
        /// Pool contract address
        pool_address: String,
        /// Start date (unix timestamp, RFC3339, or yyyy-mm-dd)
        #[arg(long)]
        start: String,
        /// End date
        #[arg(long)]
        end: Option<String>,
        /// Interval (1m, 5m, 10m, 15m, 30m, 1h, 6h, 12h, 24h)
        #[arg(long, default_value = "24h")]
        interval: String,
        /// Maximum number of data points (max 366)
        #[arg(long, default_value = "50")]
        limit: usize,
        /// Invert the price ratio
        #[arg(long)]
        inversed: bool,
    },

    /// Get detailed info about a token
    #[command(
        after_help = "EXAMPLES:\n  dexpaprika-cli token ethereum 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"
    )]
    Token {
        /// Network ID
        network: String,
        /// Token contract address
        token_address: String,
    },

    /// Get pools containing a token
    #[command(
        name = "token-pools",
        after_help = "EXAMPLES:\n  dexpaprika-cli token-pools ethereum 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2 --limit 5"
    )]
    TokenPools {
        /// Network ID
        network: String,
        /// Token contract address
        token_address: String,
        /// Maximum number of results (max 100)
        #[arg(long, default_value = "10")]
        limit: usize,
        /// Page number (1-indexed)
        #[arg(long, default_value = "1")]
        page: usize,
        /// Order by field
        #[arg(long, default_value = "volume_usd")]
        order_by: String,
        /// Sort order
        #[arg(long, default_value = "desc")]
        sort: String,
    },

    /// Filter tokens on a network by volume, liquidity, FDV, txns, creation date
    #[command(
        name = "filter-tokens",
        after_help = "EXAMPLES:\n  dexpaprika-cli filter-tokens ethereum --volume-24h-min 100000\n  dexpaprika-cli filter-tokens solana --fdv-min 1000000 --sort-by liquidity_usd"
    )]
    FilterTokens {
        /// Network ID (e.g., ethereum, solana)
        network: String,
        /// Maximum number of results (max 100)
        #[arg(long, default_value = "10")]
        limit: usize,
        /// Page number (1-indexed)
        #[arg(long, default_value = "1")]
        page: usize,
        /// Sort by field (volume_24h, volume_7d, liquidity_usd, txns_24h, created_at, fdv)
        #[arg(long, default_value = "volume_24h")]
        sort_by: String,
        /// Sort direction (asc, desc)
        #[arg(long, default_value = "desc")]
        sort_dir: String,
        /// Minimum 24h volume in USD
        #[arg(long)]
        volume_24h_min: Option<f64>,
        /// Maximum 24h volume in USD
        #[arg(long)]
        volume_24h_max: Option<f64>,
        /// Minimum liquidity in USD
        #[arg(long)]
        liquidity_usd_min: Option<f64>,
        /// Minimum FDV in USD
        #[arg(long)]
        fdv_min: Option<f64>,
        /// Maximum FDV in USD
        #[arg(long)]
        fdv_max: Option<f64>,
        /// Minimum transactions in last 24h
        #[arg(long)]
        txns_24h_min: Option<u64>,
        /// Minimum 24h price change, in percent (negative allowed)
        #[arg(long, allow_negative_numbers = true, value_parser = finite_percent)]
        price_change_24h_min: Option<f64>,
        /// Maximum 24h price change, in percent (negative allowed)
        #[arg(long, allow_negative_numbers = true, value_parser = finite_percent)]
        price_change_24h_max: Option<f64>,
        /// Only tokens created after this UNIX timestamp
        #[arg(long)]
        created_after: Option<u64>,
        /// Only tokens created before this UNIX timestamp
        #[arg(long)]
        created_before: Option<u64>,
    },

    /// Get top tokens on a network ranked by volume, price, liquidity, or activity
    #[command(
        name = "top-tokens",
        after_help = "EXAMPLES:\n  dexpaprika-cli top-tokens ethereum\n  dexpaprika-cli top-tokens solana --limit 20\n  dexpaprika-cli top-tokens ethereum --order-by price_change --sort asc"
    )]
    TopTokens {
        /// Network ID (e.g., ethereum, solana, bsc)
        network: String,
        /// Maximum number of results (max 100)
        #[arg(long, default_value = "20")]
        limit: usize,
        /// Page number (1-indexed)
        #[arg(long, default_value = "1")]
        page: usize,
        /// Order by field (volume_24h, price_usd, liquidity_usd, txns, price_change)
        #[arg(long, default_value = "volume_24h")]
        order_by: String,
        /// Sort direction (asc, desc)
        #[arg(long, default_value = "desc")]
        sort: String,
    },

    /// Get batch prices for multiple tokens
    #[command(
        after_help = "EXAMPLES:\n  dexpaprika-cli prices ethereum --tokens 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2,0xdac17f958d2ee523a2206206994597c13d831ec7"
    )]
    Prices {
        /// Network ID
        network: String,
        /// Comma-separated token addresses (max 10)
        #[arg(long)]
        tokens: String,
    },

    /// Search for tokens, pools, and DEXes across all networks
    #[command(
        after_help = "EXAMPLES:\n  dexpaprika-cli search uniswap\n  dexpaprika-cli search bitcoin"
    )]
    Search {
        /// Search query
        query: String,
    },

    /// Stream real-time token prices via SSE
    #[command(after_help = "EXAMPLES:\n  \
        dexpaprika-cli stream ethereum 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2\n  \
        dexpaprika-cli stream ethereum 0xc02a... --limit 50\n  \
        dexpaprika-cli stream --tokens watchlist.json\n\n\
        WATCHLIST FORMAT (JSON array, up to 25 entries per connection):\n  \
        [{\"chain\": \"ethereum\", \"address\": \"0xc02a...\"}, {\"chain\": \"solana\", \"address\": \"JUPy...\"}]\n\n\
        JSON FIELDS:\n  \
        address         Token contract address\n  \
        chain           Network/chain ID\n  \
        price_usd       Current price in USD\n  \
        timestamp       Event timestamp (unix)\n  \
        price_timestamp Price calculation timestamp (unix)")]
    Stream {
        /// Network ID (for single-token stream)
        network: Option<String>,
        /// Token contract address (for single-token stream)
        token_address: Option<String>,
        /// Path to JSON file with token list (for multi-token stream, max 25 entries)
        #[arg(long)]
        tokens: Option<String>,
        /// Stop after N events (default: unlimited, Ctrl+C to stop)
        #[arg(long)]
        limit: Option<usize>,
    },

    /// Stream pool reserve changes via SSE, emitted when a swap moves the reserves (USD-denominated deltas)
    #[command(
        name = "stream-reserves",
        after_help = "EXAMPLES:\n  \
        dexpaprika-cli stream-reserves ethereum 0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640 --method pool_reserves\n  \
        dexpaprika-cli stream-reserves ethereum 0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48 --method token_reserves --limit 10\n  \
        dexpaprika-cli stream-reserves --subscriptions reserves.json\n\n\
        METHODS:\n  \
        pool_reserves    Subscribe to one specific pool (fires a 'pool_reserves' event with a nested tokens array when reserves change)\n  \
        token_reserves   Subscribe to one token (fires a 'token_reserves' event per pool containing it; high volume on USDC etc)\n\n\
        SUBSCRIPTIONS FILE FORMAT (JSON array, up to 25 entries per connection):\n  \
        [{\"chain\": \"ethereum\", \"address\": \"0x88e6...\", \"method\": \"pool_reserves\", \"request_id\": 1},\n   \
         {\"chain\": \"ethereum\", \"address\": \"0xa0b8...\", \"method\": \"token_reserves\"}]\n\n\
        REQUEST ID:\n  \
        --request-id (single) or per-entry \"request_id\" (multi) is an optional uint32 (0..4294967295)\n  \
        echoed back on each data event. In the file form it defaults to the array index when omitted.\n\n\
        WIRE NOTES:\n  \
        reserve/delta/block/previous_block come as JSON strings (precision-safe). Parse with BigInt if you need arithmetic on raw integers.\n  \
        USD fields (reserve_usd, delta_usd, total_delta_usd, etc.) are regular numbers."
    )]
    StreamReserves {
        /// Network ID (for single-target stream)
        network: Option<String>,
        /// Pool or token contract address (for single-target stream)
        address: Option<String>,
        /// Streaming method: pool_reserves (one pool) or token_reserves (one token across all pools)
        #[arg(long, default_value = "pool_reserves")]
        method: String,
        /// Path to JSON file with subscriptions (for multi-target stream, max 25 entries)
        #[arg(long)]
        subscriptions: Option<String>,
        /// Correlation id (uint32, 0..4294967295) echoed back on each data event (single-target stream)
        #[arg(long)]
        request_id: Option<u32>,
        /// Stop after N data events (default: unlimited, Ctrl+C to stop)
        #[arg(long)]
        limit: Option<usize>,
    },

    /// Check DexPaprika API health status
    Status,

    /// Check for CLI updates (compares with latest GitHub release)
    #[command(name = "check-update")]
    CheckUpdate,

    /// Get ready-to-paste attribution snippets for DexPaprika
    Attribution,

    /// Interactive shell mode (REPL)
    Shell,

    /// Welcome message and quick start guide
    Onboard,
}

pub(crate) fn run(
    cli: Cli,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send>> {
    Box::pin(run_inner(cli))
}

async fn run_inner(cli: Cli) -> anyhow::Result<()> {
    let client = client::ApiClient::new();
    let output = cli.output;
    let raw = cli.raw;

    match cli.command {
        Commands::Stats => commands::stats::execute(&client, output, raw).await,
        Commands::Networks => commands::networks::execute_networks(&client, output, raw).await,
        Commands::Dexes {
            network,
            limit,
            page,
        } => commands::networks::execute_dexes(&client, &network, limit, page, output, raw).await,
        Commands::Pools {
            network,
            limit,
            page,
            order_by,
            sort,
        } => {
            commands::pools::execute_pools(
                &client, &network, limit, page, &order_by, &sort, output, raw,
            )
            .await
        }
        Commands::PoolFilter {
            network,
            volume_24h_min,
            volume_24h_max,
            volume_7d_min,
            volume_7d_max,
            liquidity_usd_min,
            liquidity_usd_max,
            txns_24h_min,
            price_change_24h_min,
            price_change_24h_max,
            price_change_6h_min,
            price_change_6h_max,
            price_change_1h_min,
            price_change_1h_max,
            price_change_5m_min,
            price_change_5m_max,
            created_after,
            created_before,
            sort_by,
            sort_dir,
            limit,
            page,
        } => {
            commands::pools::execute_pool_filter(
                &client,
                &network,
                volume_24h_min,
                volume_24h_max,
                volume_7d_min,
                volume_7d_max,
                liquidity_usd_min,
                liquidity_usd_max,
                txns_24h_min,
                PriceChangeBounds {
                    price_change_24h_min,
                    price_change_24h_max,
                    price_change_6h_min,
                    price_change_6h_max,
                    price_change_1h_min,
                    price_change_1h_max,
                    price_change_5m_min,
                    price_change_5m_max,
                },
                created_after,
                created_before,
                &sort_by,
                &sort_dir,
                limit,
                page,
                output,
                raw,
            )
            .await
        }
        Commands::Pool {
            network,
            pool_address,
            inversed,
        } => {
            commands::pools::execute_pool_detail(
                &client,
                &network,
                &pool_address,
                inversed,
                output,
                raw,
            )
            .await
        }
        Commands::DexPools {
            network,
            dex,
            limit,
            cursor,
            order_by,
            sort,
        } => {
            commands::pools::execute_dex_pools(
                &client,
                &network,
                &dex,
                limit,
                cursor.as_deref(),
                &order_by,
                &sort,
                output,
                raw,
            )
            .await
        }
        Commands::Transactions {
            network,
            pool_address,
            limit,
            cursor,
            from,
            to,
        } => {
            commands::pools::execute_transactions(
                &client,
                &network,
                &pool_address,
                limit,
                cursor.as_deref(),
                from,
                to,
                output,
                raw,
            )
            .await
        }
        Commands::PoolOhlcv {
            network,
            pool_address,
            start,
            end,
            interval,
            limit,
            inversed,
        } => {
            commands::pools::execute_ohlcv(
                &client,
                &network,
                &pool_address,
                &start,
                end.as_deref(),
                &interval,
                limit,
                inversed,
                output,
                raw,
            )
            .await
        }
        Commands::Token {
            network,
            token_address,
        } => commands::tokens::execute_token(&client, &network, &token_address, output, raw).await,
        Commands::TokenPools {
            network,
            token_address,
            limit,
            page,
            order_by,
            sort,
        } => {
            commands::tokens::execute_token_pools(
                &client,
                &network,
                &token_address,
                limit,
                page,
                &order_by,
                &sort,
                output,
                raw,
            )
            .await
        }
        Commands::FilterTokens {
            network,
            limit,
            page,
            sort_by,
            sort_dir,
            volume_24h_min,
            volume_24h_max,
            liquidity_usd_min,
            fdv_min,
            fdv_max,
            txns_24h_min,
            price_change_24h_min,
            price_change_24h_max,
            created_after,
            created_before,
        } => {
            commands::tokens::execute_filter_tokens(
                &client,
                &network,
                limit,
                page,
                &sort_by,
                &sort_dir,
                volume_24h_min,
                volume_24h_max,
                liquidity_usd_min,
                fdv_min,
                fdv_max,
                txns_24h_min,
                price_change_24h_min,
                price_change_24h_max,
                created_after,
                created_before,
                output,
                raw,
            )
            .await
        }
        Commands::TopTokens {
            network,
            limit,
            page,
            order_by,
            sort,
        } => {
            commands::tokens::execute_top_tokens(
                &client, &network, limit, page, &order_by, &sort, output, raw,
            )
            .await
        }
        Commands::Prices { network, tokens } => {
            commands::tokens::execute_prices(&client, &network, &tokens, output, raw).await
        }
        Commands::Search { query } => commands::search::execute(&client, &query, output, raw).await,
        Commands::Stream {
            network,
            token_address,
            tokens,
            limit,
        } => {
            commands::stream::execute(
                &client,
                network.as_deref(),
                token_address.as_deref(),
                tokens.as_deref(),
                limit,
                output,
            )
            .await
        }
        Commands::StreamReserves {
            network,
            address,
            method,
            subscriptions,
            request_id,
            limit,
        } => {
            commands::stream_reserves::execute(
                &client,
                network.as_deref(),
                address.as_deref(),
                &method,
                subscriptions.as_deref(),
                request_id,
                limit,
                output,
            )
            .await
        }
        Commands::Status => commands::status::execute_status(&client, output, raw).await,
        Commands::CheckUpdate => commands::version::execute(output, raw).await,
        Commands::Attribution => commands::attribution::execute(output, raw),
        Commands::Shell => {
            shell::run_shell().await;
            Ok(())
        }
        Commands::Onboard => commands::onboard::execute(),
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    let output = cli.output;

    if let Err(e) = run(cli).await {
        match output {
            OutputFormat::Json => {
                println!("{}", serde_json::json!({"error": e.to_string()}));
            }
            OutputFormat::Table => {
                eprintln!("Error: {e}");
            }
        }
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dex_pools_keeps_the_positional_dex_and_takes_a_cursor() {
        let cli = Cli::try_parse_from([
            "dexpaprika-cli",
            "dex-pools",
            "ethereum",
            "uniswap_v3",
            "--limit",
            "5",
            "--cursor",
            "eyJjaGFpbiI6ImV0aGVyZXVtIn0",
        ])
        .expect("dex-pools must still take network and dex positionally");

        match cli.command {
            Commands::DexPools {
                network,
                dex,
                limit,
                cursor,
                ..
            } => {
                assert_eq!(network, "ethereum");
                assert_eq!(dex, "uniswap_v3");
                assert_eq!(limit, 5);
                assert_eq!(cursor.as_deref(), Some("eyJjaGFpbiI6ImV0aGVyZXVtIn0"));
            }
            _ => panic!("expected the dex-pools subcommand"),
        }
    }

    /// Parse a pool-filter command line and hand back its price-change bounds.
    fn parse_bounds(args: &[&str]) -> PriceChangeBounds {
        let cli = Cli::try_parse_from(args).expect("pool-filter args should parse");
        match cli.command {
            Commands::PoolFilter {
                price_change_24h_min,
                price_change_24h_max,
                price_change_6h_min,
                price_change_6h_max,
                price_change_1h_min,
                price_change_1h_max,
                price_change_5m_min,
                price_change_5m_max,
                ..
            } => PriceChangeBounds {
                price_change_24h_min,
                price_change_24h_max,
                price_change_6h_min,
                price_change_6h_max,
                price_change_1h_min,
                price_change_1h_max,
                price_change_5m_min,
                price_change_5m_max,
            },
            _ => panic!("expected the pool-filter subcommand"),
        }
    }

    #[test]
    fn dex_pools_no_longer_accepts_a_page_number() {
        // pools/search is cursor-paginated. Accepting --page and ignoring it
        // would hand back page 1 while the caller thinks they asked for page 2.
        let parsed = Cli::try_parse_from([
            "dexpaprika-cli",
            "dex-pools",
            "ethereum",
            "uniswap_v3",
            "--page",
            "2",
        ]);
        assert!(
            parsed.is_err(),
            "--page must be rejected, not silently dropped"
        );
    }

    #[test]
    fn pool_filter_accepts_negative_price_change_bounds() {
        // "down at least 20 percent" is a max of -20. Without
        // allow_negative_numbers clap reads "-20" as an unknown flag and the
        // command fails to parse at all.
        let bounds = parse_bounds(&[
            "dexpaprika-cli",
            "pool-filter",
            "ethereum",
            "--price-change-24h-max",
            "-20",
            "--price-change-5m-min",
            "-1.5",
        ]);
        assert_eq!(bounds.price_change_24h_max, Some(-20.0));
        assert_eq!(bounds.price_change_5m_min, Some(-1.5));
    }

    #[test]
    fn pool_filter_price_change_flags_land_in_their_own_field() {
        // Eight bounds of the same type sit next to each other, so give each a
        // distinct value: a transposed pair shows up here rather than as a
        // quietly wrong query.
        let bounds = parse_bounds(&[
            "dexpaprika-cli",
            "pool-filter",
            "ethereum",
            "--price-change-24h-min",
            "1",
            "--price-change-24h-max",
            "2",
            "--price-change-6h-min",
            "3",
            "--price-change-6h-max",
            "4",
            "--price-change-1h-min",
            "5",
            "--price-change-1h-max",
            "6",
            "--price-change-5m-min",
            "7",
            "--price-change-5m-max",
            "8",
        ]);
        assert_eq!(bounds.price_change_24h_min, Some(1.0));
        assert_eq!(bounds.price_change_24h_max, Some(2.0));
        assert_eq!(bounds.price_change_6h_min, Some(3.0));
        assert_eq!(bounds.price_change_6h_max, Some(4.0));
        assert_eq!(bounds.price_change_1h_min, Some(5.0));
        assert_eq!(bounds.price_change_1h_max, Some(6.0));
        assert_eq!(bounds.price_change_5m_min, Some(7.0));
        assert_eq!(bounds.price_change_5m_max, Some(8.0));
    }

    #[test]
    fn pool_filter_price_change_bounds_default_to_none() {
        let bounds = parse_bounds(&["dexpaprika-cli", "pool-filter", "ethereum"]);
        assert_eq!(bounds.price_change_24h_min, None);
        assert_eq!(bounds.price_change_5m_max, None);
    }

    #[test]
    fn filter_tokens_has_no_short_price_change_window_flags() {
        // tokens/search 400s on the 6h, 1h and 5m sort fields and silently
        // ignores their filter bounds, so the token command stays out of them.
        // The 24h pair is a different case: tokens/search does honour it, and
        // giving filter-tokens those two flags is a change for another PR, not
        // something ruled out by the API.
        for flag in [
            "--price-change-6h-min",
            "--price-change-1h-min",
            "--price-change-5m-max",
        ] {
            // Cli has no Debug impl, so unwrap_err is out; match instead.
            let err = match Cli::try_parse_from([
                "dexpaprika-cli",
                "filter-tokens",
                "ethereum",
                flag,
                "50",
            ]) {
                Ok(_) => panic!("filter-tokens should reject {flag}"),
                Err(err) => err,
            };
            // Assert why it failed, not merely that it did: any parse error at
            // all would satisfy an is_err() check, including one from renaming
            // or deleting the subcommand.
            assert_eq!(
                err.kind(),
                clap::error::ErrorKind::UnknownArgument,
                "{flag} should be rejected as an unknown argument"
            );
        }
    }

    #[test]
    fn pool_filter_rejects_nan_and_infinite_bounds() {
        // NaN and inf parse as f64 and make the API answer 500, which the client
        // reports as an outage. Refuse them at the CLI boundary instead.
        for bad in ["nan", "NaN", "inf", "-inf", "infinity"] {
            let parsed = Cli::try_parse_from([
                "dexpaprika-cli",
                "pool-filter",
                "ethereum",
                "--price-change-24h-min",
                bad,
            ]);
            assert!(parsed.is_err(), "`{bad}` should not parse as a bound");
        }
        // Control: an ordinary negative bound still gets through.
        let bounds = parse_bounds(&[
            "dexpaprika-cli",
            "pool-filter",
            "ethereum",
            "--price-change-24h-min",
            "-0.5",
        ]);
        assert_eq!(bounds.price_change_24h_min, Some(-0.5));
    }
}
