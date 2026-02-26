use tabled::{Table, Tabled};
use tabled::settings::Style;

use crate::commands::tokens::{TokenDetail, TokenPoolItem, TokenPrice};
use crate::output::{detail_field, format_percent, format_price, format_usd, print_dexpaprika_footer, print_detail_table, truncate_address};

pub fn print_token_detail(token: &TokenDetail) {
    let mut rows: Vec<[String; 2]> = Vec::new();
    detail_field!(rows, "Name", token.name.clone().unwrap_or_else(|| "—".into()));
    detail_field!(rows, "Symbol", token.symbol.clone().unwrap_or_else(|| "—".into()));
    detail_field!(rows, "Chain", token.chain.clone().unwrap_or_else(|| "—".into()));
    detail_field!(rows, "Address", token.id.clone().unwrap_or_else(|| "—".into()));
    detail_field!(rows, "Decimals", token.decimals.map(|d| d.to_string()).unwrap_or_else(|| "—".into()));
    detail_field!(rows, "Total Supply", token.total_supply.map(|s| format!("{:.2}", s)).unwrap_or_else(|| "—".into()));

    if let Some(summary) = &token.summary {
        detail_field!(rows, "Price (USD)", summary.price_usd.map(format_price).unwrap_or_else(|| "—".into()));
        detail_field!(rows, "Liquidity (USD)", summary.liquidity_usd.map(format_usd).unwrap_or_else(|| "—".into()));

        if let Some(h24) = &summary.h24 {
            detail_field!(rows, "Volume (24h)", h24.volume_usd.map(format_usd).unwrap_or_else(|| "—".into()));
            detail_field!(rows, "Change (24h)", h24.last_price_usd_change.map(format_percent).unwrap_or_else(|| "—".into()));
            detail_field!(rows, "Buys/Sells (24h)", format!("{}/{}", h24.buys.unwrap_or(0), h24.sells.unwrap_or(0)));
        }

        if let Some(h1) = &summary.h1 {
            detail_field!(rows, "Volume (1h)", h1.volume_usd.map(format_usd).unwrap_or_else(|| "—".into()));
            detail_field!(rows, "Change (1h)", h1.last_price_usd_change.map(format_percent).unwrap_or_else(|| "—".into()));
        }
    }

    print_detail_table(rows);
    print_dexpaprika_footer();
}

#[derive(Tabled)]
struct TokenPoolRow {
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
    #[tabled(rename = "Liquidity")]
    liquidity: String,
}

pub fn print_token_pools_table(pools: &[TokenPoolItem]) {
    let rows: Vec<TokenPoolRow> = pools.iter().map(|p| {
        let pair = p.tokens.as_ref()
            .map(|ts| ts.iter().map(|t| t.symbol.clone().unwrap_or_else(|| "?".into())).collect::<Vec<_>>().join("/"))
            .unwrap_or_else(|| "—".into());
        TokenPoolRow {
            pool: p.id.as_deref().map(truncate_address).unwrap_or_else(|| "—".into()),
            dex: p.dex_name.clone().unwrap_or_else(|| "—".into()),
            pair,
            price: p.price_usd.map(format_price).unwrap_or_else(|| "—".into()),
            volume: p.volume_usd.map(format_usd).unwrap_or_else(|| "—".into()),
            liquidity: p.liquidity_usd.map(format_usd).unwrap_or_else(|| "—".into()),
        }
    }).collect();

    let table = Table::new(rows).with(Style::rounded()).to_string();
    println!("{table}");
    print_dexpaprika_footer();
}

#[derive(Tabled)]
struct PriceRow {
    #[tabled(rename = "Token")]
    token: String,
    #[tabled(rename = "Chain")]
    chain: String,
    #[tabled(rename = "Price (USD)")]
    price: String,
}

pub fn print_prices_table(prices: &[TokenPrice]) {
    let rows: Vec<PriceRow> = prices.iter().map(|p| PriceRow {
        token: p.id.as_deref().map(truncate_address).unwrap_or_else(|| "—".into()),
        chain: p.chain.clone().unwrap_or_else(|| "—".into()),
        price: p.price_usd.map(format_price).unwrap_or_else(|| "—".into()),
    }).collect();

    let table = Table::new(rows).with(Style::rounded()).to_string();
    println!("{table}");
    print_dexpaprika_footer();
}
