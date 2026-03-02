use crate::commands::search::DexSearchResult;
use crate::output::{format_price, format_usd, format_percent, print_dexpaprika_footer, truncate_address};

pub fn print_dex_search(result: &DexSearchResult) {
    if let Some(tokens) = &result.tokens {
        if !tokens.is_empty() {
            println!("Tokens:");
            for t in tokens {
                let price_str = t.price_usd.map(format_price).unwrap_or_default();
                let vol_str = t.volume_usd.map(|v| format!("vol {}", format_usd(v))).unwrap_or_default();
                let change_str = t.price_usd_change.map(|c| format_percent(c)).unwrap_or_default();
                let extra = [price_str, vol_str, change_str]
                    .iter()
                    .filter(|s| !s.is_empty())
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" | ");
                let suffix = if extra.is_empty() { String::new() } else { format!("  {extra}") };
                println!("  {} ({}) — {} [{}]{suffix}",
                    t.name.as_deref().unwrap_or("—"),
                    t.symbol.as_deref().unwrap_or("—"),
                    t.id.as_deref().map(truncate_address).unwrap_or_else(|| "—".into()),
                    t.chain.as_deref().unwrap_or("—"),
                );
            }
            println!();
        }
    }

    if let Some(pools) = &result.pools {
        if !pools.is_empty() {
            println!("Pools:");
            for p in pools {
                let vol_str = p.volume_usd.map(|v| format!("vol {}", format_usd(v))).unwrap_or_default();
                let price_str = p.price_usd.map(format_price).unwrap_or_default();
                let extra = [price_str, vol_str]
                    .iter()
                    .filter(|s| !s.is_empty())
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" | ");
                let suffix = if extra.is_empty() { String::new() } else { format!("  {extra}") };
                println!("  {} — {} [{}]{suffix}",
                    p.id.as_deref().map(truncate_address).unwrap_or_else(|| "—".into()),
                    p.dex_name.as_deref().unwrap_or("—"),
                    p.chain.as_deref().unwrap_or("—"),
                );
            }
            println!();
        }
    }

    if let Some(dexes) = &result.dexes {
        if !dexes.is_empty() {
            println!("DEXes:");
            for d in dexes {
                println!("  {} — {} [{}]",
                    d.id.as_deref().unwrap_or("—"),
                    d.name.as_deref().unwrap_or("—"),
                    d.chain.as_deref().unwrap_or("—"),
                );
            }
            println!();
        }
    }

    print_dexpaprika_footer();
}
