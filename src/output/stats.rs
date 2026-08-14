use crate::commands::stats::DexStats;
use crate::output::{print_detail_table, print_dexpaprika_footer};

pub fn print_stats(stats: &DexStats) {
    let mut rows: Vec<[String; 2]> = Vec::new();
    detail_field!(
        rows,
        "Chains/Networks",
        stats
            .chains
            .map(|v| v.to_string())
            .unwrap_or_else(|| "-".into())
    );
    detail_field!(
        rows,
        "DEXes/Factories",
        stats
            .factories
            .map(|v| v.to_string())
            .unwrap_or_else(|| "-".into())
    );
    detail_field!(
        rows,
        "Pools",
        stats
            .pools
            .map(|v| v.to_string())
            .unwrap_or_else(|| "-".into())
    );
    detail_field!(
        rows,
        "Tokens",
        stats
            .tokens
            .map(|v| v.to_string())
            .unwrap_or_else(|| "-".into())
    );
    print_detail_table(rows);
    print_dexpaprika_footer();
}
