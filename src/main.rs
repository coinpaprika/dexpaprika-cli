mod client;
mod commands;
mod output;
mod shell;

use clap::{Parser, Subcommand};
use output::OutputFormat;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "dexpaprika-cli",
    version,
    about = "dexpaprika-cli — Free DEX data from your terminal",
    long_about = "dexpaprika-cli — Free DEX data from your terminal\n\n\
                   Pools · Tokens · On-chain trades · 26+ chains · Real-time streaming\n\n\
                   No API key. No rate limits. No credit card. Forever free.\n\n\
                   Quick start:  dexpaprika-cli onboard\n\
                   API docs:     https://api.dexpaprika.com\n\
                   Docs:         https://docs.dexpaprika.com"
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
        /// Maximum number of results
        #[arg(long, default_value = "10")]
        limit: usize,
        /// Page number (0-indexed)
        #[arg(long, default_value = "0")]
        page: usize,
    },

    /// List top pools on a network
    #[command(after_help = "EXAMPLES:\n  dexpaprika-cli pools ethereum --limit 5\n  dexpaprika-cli pools solana --order-by volume_usd --sort desc")]
    Pools {
        /// Network ID (e.g., ethereum, solana)
        network: String,
        /// Maximum number of results
        #[arg(long, default_value = "10")]
        limit: usize,
        /// Page number (0-indexed)
        #[arg(long, default_value = "0")]
        page: usize,
        /// Order by field
        #[arg(long, default_value = "volume_usd")]
        order_by: String,
        /// Sort order
        #[arg(long, default_value = "desc")]
        sort: String,
    },

    /// Get detailed info about a specific pool
    #[command(after_help = "EXAMPLES:\n  dexpaprika-cli pool ethereum 0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640")]
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
    #[command(name = "dex-pools", after_help = "EXAMPLES:\n  dexpaprika-cli dex-pools ethereum uniswap_v3 --limit 5")]
    DexPools {
        /// Network ID
        network: String,
        /// DEX identifier (e.g., uniswap_v3, sushiswap)
        dex: String,
        /// Maximum number of results
        #[arg(long, default_value = "10")]
        limit: usize,
        /// Page number (0-indexed)
        #[arg(long, default_value = "0")]
        page: usize,
    },

    /// Get recent transactions for a pool
    #[command(after_help = "EXAMPLES:\n  dexpaprika-cli transactions ethereum 0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640 --limit 20")]
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
    },

    /// Get OHLCV data for a pool
    #[command(name = "pool-ohlcv", after_help = "EXAMPLES:\n  dexpaprika-cli pool-ohlcv ethereum 0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640 --start 2025-01-01")]
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
    #[command(after_help = "EXAMPLES:\n  dexpaprika-cli token ethereum 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2")]
    Token {
        /// Network ID
        network: String,
        /// Token contract address
        token_address: String,
    },

    /// Get pools containing a token
    #[command(name = "token-pools", after_help = "EXAMPLES:\n  dexpaprika-cli token-pools ethereum 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2 --limit 5")]
    TokenPools {
        /// Network ID
        network: String,
        /// Token contract address
        token_address: String,
        /// Maximum number of results
        #[arg(long, default_value = "10")]
        limit: usize,
        /// Page number (0-indexed)
        #[arg(long, default_value = "0")]
        page: usize,
        /// Order by field
        #[arg(long, default_value = "volume_usd")]
        order_by: String,
        /// Sort order
        #[arg(long, default_value = "desc")]
        sort: String,
    },

    /// Get batch prices for multiple tokens
    #[command(after_help = "EXAMPLES:\n  dexpaprika-cli prices ethereum --tokens 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2,0xdac17f958d2ee523a2206206994597c13d831ec7")]
    Prices {
        /// Network ID
        network: String,
        /// Comma-separated token addresses (max 10)
        #[arg(long)]
        tokens: String,
    },

    /// Search for tokens, pools, and DEXes across all networks
    #[command(after_help = "EXAMPLES:\n  dexpaprika-cli search uniswap\n  dexpaprika-cli search bitcoin")]
    Search {
        /// Search query
        query: String,
    },

    /// Stream real-time token prices via SSE
    #[command(after_help = "EXAMPLES:\n  dexpaprika-cli stream ethereum 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2\n  dexpaprika-cli stream --tokens watchlist.json --limit 100")]
    Stream {
        /// Network ID (for single-token stream)
        network: Option<String>,
        /// Token contract address (for single-token stream)
        token_address: Option<String>,
        /// Path to JSON file with token list (for multi-token stream)
        #[arg(long)]
        tokens: Option<String>,
        /// Stop after N events (default: unlimited, Ctrl+C to stop)
        #[arg(long)]
        limit: Option<usize>,
    },

    /// Check DexPaprika API health status
    Status,

    /// Get ready-to-paste attribution snippets for DexPaprika
    Attribution,

    /// Interactive shell mode (REPL)
    Shell,

    /// Welcome message and quick start guide
    Onboard,
}

pub(crate) fn run(cli: Cli) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send>> {
    Box::pin(run_inner(cli))
}

async fn run_inner(cli: Cli) -> anyhow::Result<()> {
    let client = client::ApiClient::new();
    let output = cli.output;
    let raw = cli.raw;

    match cli.command {
        Commands::Stats => commands::stats::execute(&client, output, raw).await,
        Commands::Networks => commands::networks::execute_networks(&client, output, raw).await,
        Commands::Dexes { network, limit, page } => commands::networks::execute_dexes(&client, &network, limit, page, output, raw).await,
        Commands::Pools { network, limit, page, order_by, sort } => {
            commands::pools::execute_pools(&client, &network, limit, page, &order_by, &sort, output, raw).await
        }
        Commands::Pool { network, pool_address, inversed } => {
            commands::pools::execute_pool_detail(&client, &network, &pool_address, inversed, output, raw).await
        }
        Commands::DexPools { network, dex, limit, page } => {
            commands::pools::execute_dex_pools(&client, &network, &dex, limit, page, output, raw).await
        }
        Commands::Transactions { network, pool_address, limit, cursor } => {
            commands::pools::execute_transactions(&client, &network, &pool_address, limit, cursor.as_deref(), output, raw).await
        }
        Commands::PoolOhlcv { network, pool_address, start, end, interval, limit, inversed } => {
            commands::pools::execute_ohlcv(&client, &network, &pool_address, &start, end.as_deref(), &interval, limit, inversed, output, raw).await
        }
        Commands::Token { network, token_address } => {
            commands::tokens::execute_token(&client, &network, &token_address, output, raw).await
        }
        Commands::TokenPools { network, token_address, limit, page, order_by, sort } => {
            commands::tokens::execute_token_pools(&client, &network, &token_address, limit, page, &order_by, &sort, output, raw).await
        }
        Commands::Prices { network, tokens } => {
            commands::tokens::execute_prices(&client, &network, &tokens, output, raw).await
        }
        Commands::Search { query } => commands::search::execute(&client, &query, output, raw).await,
        Commands::Stream { network, token_address, tokens, limit } => {
            commands::stream::execute(&client, network.as_deref(), token_address.as_deref(), tokens.as_deref(), limit, output).await
        }
        Commands::Status => commands::status::execute_status(&client, output, raw).await,
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
                println!(
                    "{}",
                    serde_json::json!({"error": e.to_string()})
                );
            }
            OutputFormat::Table => {
                eprintln!("Error: {e}");
            }
        }
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
