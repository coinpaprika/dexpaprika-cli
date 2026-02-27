use tabled::{Table, Tabled};
use tabled::settings::Style;

use crate::commands::pools::{PoolListItem, PoolDetail, PoolTransaction, PoolOhlcv};
use crate::output::{detail_field, format_percent, format_price, format_usd, print_dexpaprika_footer, print_detail_table, truncate_address};

fn pool_pair(tokens: &Option<Vec<crate::commands::pools::PoolToken>>) -> String {
    tokens.as_ref()
        .map(|ts| {
            ts.iter()
                .map(|t| t.symbol.clone().unwrap_or_else(|| "?".into()))
                .collect::<Vec<_>>()
                .join("/")
        })
        .unwrap_or_else(|| "—".into())
}

#[derive(Tabled)]
struct PoolRow {
    #[tabled(rename = "Pool")]
    pool: String,
    #[tabled(rename = "DEX")]
    dex: String,
    #[tabled(rename = "Pair")]
    pair: String,
    #[tabled(rename = "Price")]
    price: String,
    #[tabled(rename = "Volume (24h)")]
    volume: String,
    #[tabled(rename = "24h Change")]
    change: String,
}

pub fn print_pools_table(pools: &[PoolListItem]) {
    let rows: Vec<PoolRow> = pools.iter().map(|p| PoolRow {
        pool: p.id.as_deref().map(truncate_address).unwrap_or_else(|| "—".into()),
        dex: p.dex_name.clone().unwrap_or_else(|| "—".into()),
        pair: pool_pair(&p.tokens),
        price: p.price_usd.map(format_price).unwrap_or_else(|| "—".into()),
        volume: p.volume_usd.map(format_usd).unwrap_or_else(|| "—".into()),
        change: p.last_price_change_usd_24h.map(format_percent).unwrap_or_else(|| "—".into()),
    }).collect();

    let table = Table::new(rows).with(Style::rounded()).to_string();
    println!("{table}");
    print_dexpaprika_footer();
}

pub fn print_pool_detail(pool: &PoolDetail) {
    let mut rows: Vec<[String; 2]> = Vec::new();
    detail_field!(rows, "Pool ID", pool.id.clone().unwrap_or_else(|| "—".into()));
    detail_field!(rows, "DEX", format!("{} ({})", pool.dex_name.as_deref().unwrap_or("—"), pool.dex_id.as_deref().unwrap_or("—")));
    detail_field!(rows, "Pair", pool_pair(&pool.tokens));
    detail_field!(rows, "Price (USD)", pool.last_price_usd.map(format_price).unwrap_or_else(|| "—".into()));
    detail_field!(rows, "Created At", pool.created_at.clone().unwrap_or_else(|| "—".into()));

    if let Some(h24) = &pool.h24 {
        detail_field!(rows, "Volume (24h)", h24.volume_usd.map(format_usd).unwrap_or_else(|| "—".into()));
        detail_field!(rows, "24h Change", h24.last_price_usd_change.map(format_percent).unwrap_or_else(|| "—".into()));
        detail_field!(rows, "Txns (24h)", h24.txns.map(|t| t.to_string()).unwrap_or_else(|| "—".into()));
    }

    if let Some(tokens) = &pool.tokens {
        for (i, t) in tokens.iter().enumerate() {
            detail_field!(rows, &format!("Token {}", i), format!("{} ({}) — {}",
                t.name.as_deref().unwrap_or("—"),
                t.symbol.as_deref().unwrap_or("—"),
                t.id.as_deref().map(truncate_address).unwrap_or_else(|| "—".into())
            ));
        }
    }

    print_detail_table(rows);
    print_dexpaprika_footer();
}

#[derive(Tabled)]
struct TxRow {
    #[tabled(rename = "Time")]
    time: String,
    #[tabled(rename = "Type")]
    tx_type: String,
    #[tabled(rename = "Amount (USD)")]
    amount: String,
    #[tabled(rename = "Token 0")]
    token_0: String,
    #[tabled(rename = "Token 1")]
    token_1: String,
}

pub fn print_transactions_table(txs: &[PoolTransaction]) {
    let rows: Vec<TxRow> = txs.iter().map(|tx| {
        let t0 = format!("{:.4} {}",
            tx.volume_0.unwrap_or(0.0),
            tx.token_0_symbol.as_deref().unwrap_or("?")
        );
        let t1 = format!("{:.4} {}",
            tx.volume_1.unwrap_or(0.0),
            tx.token_1_symbol.as_deref().unwrap_or("?")
        );
        let total_usd = tx.price_0_usd.unwrap_or(0.0) * tx.volume_0.unwrap_or(0.0)
            + tx.price_1_usd.unwrap_or(0.0) * tx.volume_1.unwrap_or(0.0);

        TxRow {
            time: tx.created_at.clone().unwrap_or_else(|| "—".into()),
            tx_type: "swap".into(),
            amount: format_usd(total_usd.abs()),
            token_0: crate::output::truncate(&t0, 25),
            token_1: crate::output::truncate(&t1, 25),
        }
    }).collect();

    let table = Table::new(rows).with(Style::rounded()).to_string();
    println!("{table}");
    print_dexpaprika_footer();
}

#[derive(Tabled)]
struct OhlcvRow {
    #[tabled(rename = "Date")]
    date: String,
    #[tabled(rename = "Open")]
    open: String,
    #[tabled(rename = "High")]
    high: String,
    #[tabled(rename = "Low")]
    low: String,
    #[tabled(rename = "Close")]
    close: String,
    #[tabled(rename = "Volume")]
    volume: String,
}

pub fn print_pool_ohlcv_table(data: &[PoolOhlcv]) {
    let rows: Vec<OhlcvRow> = data.iter().map(|d| OhlcvRow {
        date: d.time_open.as_deref().unwrap_or("—").chars().take(19).collect(),
        open: d.open.map(format_price).unwrap_or_else(|| "—".into()),
        high: d.high.map(format_price).unwrap_or_else(|| "—".into()),
        low: d.low.map(format_price).unwrap_or_else(|| "—".into()),
        close: d.close.map(format_price).unwrap_or_else(|| "—".into()),
        volume: d.volume.map(format_usd).unwrap_or_else(|| "—".into()),
    }).collect();

    let table = Table::new(rows).with(Style::rounded()).to_string();
    println!("{table}");
    print_dexpaprika_footer();
}
