use tabled::settings::Style;
use tabled::{Table, Tabled};

use crate::commands::search_pools::{PoolRow, SearchPoolToken};
use crate::output::{
    format_percent, format_price, format_usd, print_dexpaprika_footer, truncate_address,
};

/// Build a "USDC/WETH" pair label from the token legs. Prefers symbols (only
/// present with --detailed); falls back to truncated addresses otherwise.
fn pool_pair(tokens: &Option<Vec<SearchPoolToken>>) -> String {
    tokens
        .as_ref()
        .map(|ts| {
            ts.iter()
                .map(|t| {
                    t.symbol
                        .clone()
                        .or_else(|| t.id.as_deref().map(truncate_address))
                        .unwrap_or_else(|| "?".into())
                })
                .collect::<Vec<_>>()
                .join("/")
        })
        .unwrap_or_else(|| "—".into())
}

/// Compact view: the common case, one line per pool with the headline metrics.
#[derive(Tabled)]
struct PoolSearchRow {
    #[tabled(rename = "Pool")]
    pool: String,
    #[tabled(rename = "Chain")]
    chain: String,
    #[tabled(rename = "DEX")]
    dex: String,
    #[tabled(rename = "Pair")]
    pair: String,
    #[tabled(rename = "Price")]
    price: String,
    #[tabled(rename = "Vol (24h)")]
    volume_24h: String,
    #[tabled(rename = "Liquidity")]
    liquidity: String,
    #[tabled(rename = "24h %")]
    change_24h: String,
}

/// Wide view (--detailed): adds the 7d/30d volume columns, txn count, and FDV
/// so the extra payload the flag pulls down is actually surfaced.
#[derive(Tabled)]
struct PoolSearchRowDetailed {
    #[tabled(rename = "Pool")]
    pool: String,
    #[tabled(rename = "Chain")]
    chain: String,
    #[tabled(rename = "DEX")]
    dex: String,
    #[tabled(rename = "Pair")]
    pair: String,
    #[tabled(rename = "Price")]
    price: String,
    #[tabled(rename = "Vol (24h)")]
    volume_24h: String,
    #[tabled(rename = "Vol (7d)")]
    volume_7d: String,
    #[tabled(rename = "Vol (30d)")]
    volume_30d: String,
    #[tabled(rename = "Liquidity")]
    liquidity: String,
    #[tabled(rename = "Txns (24h)")]
    txns_24h: String,
    #[tabled(rename = "24h %")]
    change_24h: String,
    #[tabled(rename = "FDV")]
    fdv: String,
}

/// Sum the FDV across the pool's token legs. Only populated under --detailed;
/// shown as a dash when nothing reports an FDV.
fn pool_fdv(tokens: &Option<Vec<SearchPoolToken>>) -> String {
    let total: f64 = tokens
        .as_ref()
        .map(|ts| ts.iter().filter_map(|t| t.fdv).sum())
        .unwrap_or(0.0);
    if total > 0.0 {
        format_usd(total)
    } else {
        "—".into()
    }
}

pub fn print_pool_rows(rows: &[PoolRow], detailed: bool) {
    if rows.is_empty() {
        println!("No pools matched those filters.");
        print_dexpaprika_footer();
        return;
    }

    if detailed {
        let table_rows: Vec<PoolSearchRowDetailed> = rows
            .iter()
            .map(|p| PoolSearchRowDetailed {
                pool: p
                    .id
                    .as_deref()
                    .map(truncate_address)
                    .unwrap_or_else(|| "—".into()),
                chain: p.chain.clone().unwrap_or_else(|| "—".into()),
                dex: p.dex_name.clone().unwrap_or_else(|| "—".into()),
                pair: pool_pair(&p.tokens),
                price: p.price_usd.map(format_price).unwrap_or_else(|| "—".into()),
                volume_24h: p
                    .volume_usd_24h
                    .map(format_usd)
                    .unwrap_or_else(|| "—".into()),
                volume_7d: p
                    .volume_usd_7d
                    .map(format_usd)
                    .unwrap_or_else(|| "—".into()),
                volume_30d: p
                    .volume_usd_30d
                    .map(format_usd)
                    .unwrap_or_else(|| "—".into()),
                liquidity: p
                    .liquidity_usd
                    .map(format_usd)
                    .unwrap_or_else(|| "—".into()),
                txns_24h: p
                    .transactions_24h
                    .map(|t| t.to_string())
                    .unwrap_or_else(|| "—".into()),
                change_24h: p
                    .price_change_percentage_24h
                    .map(format_percent)
                    .unwrap_or_else(|| "—".into()),
                fdv: pool_fdv(&p.tokens),
            })
            .collect();
        let table = Table::new(table_rows).with(Style::rounded()).to_string();
        println!("{table}");
    } else {
        let table_rows: Vec<PoolSearchRow> = rows
            .iter()
            .map(|p| PoolSearchRow {
                pool: p
                    .id
                    .as_deref()
                    .map(truncate_address)
                    .unwrap_or_else(|| "—".into()),
                chain: p.chain.clone().unwrap_or_else(|| "—".into()),
                dex: p.dex_name.clone().unwrap_or_else(|| "—".into()),
                pair: pool_pair(&p.tokens),
                price: p.price_usd.map(format_price).unwrap_or_else(|| "—".into()),
                volume_24h: p
                    .volume_usd_24h
                    .map(format_usd)
                    .unwrap_or_else(|| "—".into()),
                liquidity: p
                    .liquidity_usd
                    .map(format_usd)
                    .unwrap_or_else(|| "—".into()),
                change_24h: p
                    .price_change_percentage_24h
                    .map(format_percent)
                    .unwrap_or_else(|| "—".into()),
            })
            .collect();
        let table = Table::new(table_rows).with(Style::rounded()).to_string();
        println!("{table}");
    }

    print_dexpaprika_footer();
}
