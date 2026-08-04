use tabled::settings::Style;
use tabled::{Table, Tabled};

use crate::commands::tokens::{TokenDetail, TokenPrice, TokenSearchItem};
use crate::output::{
    detail_field, format_percent, format_price, format_usd, print_detail_table,
    print_dexpaprika_footer, truncate_address,
};

pub fn print_token_detail(token: &TokenDetail) {
    let mut rows: Vec<[String; 2]> = Vec::new();
    detail_field!(
        rows,
        "Name",
        token.name.clone().unwrap_or_else(|| "-".into())
    );
    detail_field!(
        rows,
        "Symbol",
        token.symbol.clone().unwrap_or_else(|| "-".into())
    );
    detail_field!(
        rows,
        "Chain",
        token.chain.clone().unwrap_or_else(|| "-".into())
    );
    detail_field!(
        rows,
        "Address",
        token.id.clone().unwrap_or_else(|| "-".into())
    );
    detail_field!(
        rows,
        "Decimals",
        token
            .decimals
            .map(|d| d.to_string())
            .unwrap_or_else(|| "-".into())
    );
    detail_field!(
        rows,
        "Total Supply",
        token
            .total_supply
            .map(|s| format!("{s:.2}"))
            .unwrap_or_else(|| "-".into())
    );

    if let Some(desc) = &token.description {
        if !desc.is_empty() {
            detail_field!(rows, "Description", crate::output::truncate(desc, 80));
        }
    }
    if let Some(website) = &token.website {
        if !website.is_empty() {
            detail_field!(rows, "Website", website.clone());
        }
    }
    if let Some(telegram) = &token.telegram {
        if !telegram.is_empty() {
            detail_field!(rows, "Telegram", telegram.clone());
        }
    }
    if let Some(twitter) = &token.twitter {
        if !twitter.is_empty() {
            detail_field!(rows, "Twitter", twitter.clone());
        }
    }

    if let Some(summary) = &token.summary {
        detail_field!(
            rows,
            "Price (USD)",
            summary
                .price_usd
                .map(format_price)
                .unwrap_or_else(|| "-".into())
        );
        detail_field!(
            rows,
            "FDV",
            summary.fdv.map(format_usd).unwrap_or_else(|| "-".into())
        );
        detail_field!(
            rows,
            "Liquidity (USD)",
            summary
                .liquidity_usd
                .map(format_usd)
                .unwrap_or_else(|| "-".into())
        );
        detail_field!(
            rows,
            "Pools",
            summary
                .pools
                .map(|p| p.to_string())
                .unwrap_or_else(|| "-".into())
        );

        if let Some(h24) = &summary.h24 {
            detail_field!(
                rows,
                "Volume (24h)",
                h24.volume_usd.map(format_usd).unwrap_or_else(|| "-".into())
            );
            detail_field!(
                rows,
                "Change (24h)",
                h24.last_price_usd_change
                    .map(format_percent)
                    .unwrap_or_else(|| "-".into())
            );
            detail_field!(
                rows,
                "Buys/Sells (24h)",
                format!("{}/{}", h24.buys.unwrap_or(0), h24.sells.unwrap_or(0))
            );
            detail_field!(
                rows,
                "Txns (24h)",
                h24.txns
                    .map(|t| t.to_string())
                    .unwrap_or_else(|| "-".into())
            );
        }

        if let Some(h1) = &summary.h1 {
            detail_field!(
                rows,
                "Volume (1h)",
                h1.volume_usd.map(format_usd).unwrap_or_else(|| "-".into())
            );
            detail_field!(
                rows,
                "Change (1h)",
                h1.last_price_usd_change
                    .map(format_percent)
                    .unwrap_or_else(|| "-".into())
            );
        }

        if let Some(m5) = &summary.m5 {
            detail_field!(
                rows,
                "Change (5m)",
                m5.last_price_usd_change
                    .map(format_percent)
                    .unwrap_or_else(|| "-".into())
            );
        }
    }

    if let Some(ps) = &token.price_stats {
        detail_field!(
            rows,
            "High (24h)",
            ps.high_24h.map(format_price).unwrap_or_else(|| "-".into())
        );
        detail_field!(
            rows,
            "Low (24h)",
            ps.low_24h.map(format_price).unwrap_or_else(|| "-".into())
        );
        detail_field!(
            rows,
            "ATH",
            ps.ath.map(format_price).unwrap_or_else(|| "-".into())
        );
    }

    print_detail_table(rows);
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

// --- Unified token search table (top-tokens + token-filter) ---

#[derive(Tabled)]
struct TokenSearchRow {
    #[tabled(rename = "Address")]
    address: String,
    #[tabled(rename = "Chain")]
    chain: String,
    #[tabled(rename = "Price")]
    price: String,
    #[tabled(rename = "Volume (24h)")]
    volume_24h: String,
    #[tabled(rename = "24h Change")]
    change: String,
    #[tabled(rename = "Liquidity")]
    liquidity: String,
    #[tabled(rename = "FDV")]
    fdv: String,
    #[tabled(rename = "Txns (24h)")]
    txns: String,
}

/// Render token rows from the unified search endpoint. The flat search payload
/// carries no name/symbol/buys/sells/pools, so only the available fields are
/// shown. Used by both the top-tokens and token-filter commands.
pub fn print_token_search_table(tokens: &[TokenSearchItem]) {
    let rows: Vec<TokenSearchRow> = tokens
        .iter()
        .map(|t| TokenSearchRow {
            address: t
                .address
                .as_deref()
                .map(truncate_address)
                .unwrap_or_else(|| "-".into()),
            chain: t.chain.clone().unwrap_or_else(|| "-".into()),
            price: t.price_usd.map(format_price).unwrap_or_else(|| "-".into()),
            volume_24h: t
                .volume_usd_24h
                .map(format_usd)
                .unwrap_or_else(|| "-".into()),
            change: t
                .price_change_percentage_24h
                .map(format_percent)
                .unwrap_or_else(|| "-".into()),
            liquidity: t
                .liquidity_usd
                .map(format_usd)
                .unwrap_or_else(|| "-".into()),
            fdv: t.fdv_usd.map(format_usd).unwrap_or_else(|| "-".into()),
            txns: t
                .txns_24h
                .map(|n| n.to_string())
                .unwrap_or_else(|| "-".into()),
        })
        .collect();

    let table = Table::new(rows).with(Style::rounded()).to_string();
    println!("{table}");
    print_dexpaprika_footer();
}

pub fn print_prices_table(prices: &[TokenPrice]) {
    let rows: Vec<PriceRow> = prices
        .iter()
        .map(|p| PriceRow {
            token: p
                .id
                .as_deref()
                .map(truncate_address)
                .unwrap_or_else(|| "-".into()),
            chain: p.chain.clone().unwrap_or_else(|| "-".into()),
            price: p.price_usd.map(format_price).unwrap_or_else(|| "-".into()),
        })
        .collect();

    let table = Table::new(rows).with(Style::rounded()).to_string();
    println!("{table}");
    print_dexpaprika_footer();
}
