use crate::commands::search::DexSearchResult;
use crate::output::{print_dexpaprika_footer, truncate_address};

pub fn print_dex_search(result: &DexSearchResult) {
    if let Some(tokens) = &result.tokens {
        if !tokens.is_empty() {
            println!("Tokens:");
            for t in tokens {
                println!("  {} ({}) — {} [{}]",
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
                println!("  {} — {} [{}]",
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
