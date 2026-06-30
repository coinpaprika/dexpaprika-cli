//! Pure helpers that normalize DexPaprika sort-field values for the unified
//! search endpoints.
//!
//! The `/networks/{network}/pools/search` and `/networks/{network}/tokens/search`
//! endpoints return HTTP 400 for the legacy sort-field names that the old
//! list/filter endpoints accepted. Every incoming sort field must therefore be
//! mapped to its canonical value before it is sent upstream. Unknown values fall
//! back to the default, "volume_usd_24h".

/// Default sort field shared by both pools and tokens search.
const DEFAULT_SORT_FIELD: &str = "volume_usd_24h";

/// Map a pool sort field (legacy or canonical) to the canonical value accepted
/// by `/networks/{network}/pools/search`.
pub fn map_pool_sort_field(field: &str) -> &'static str {
    match field {
        // Canonical values pass through unchanged.
        "volume_usd_24h" => "volume_usd_24h",
        "volume_usd_7d" => "volume_usd_7d",
        "volume_usd_30d" => "volume_usd_30d",
        "liquidity_usd" => "liquidity_usd",
        "txns_24h" => "txns_24h",
        "created_at" => "created_at",
        "price_usd" => "price_usd",
        "price_change_percentage_24h" => "price_change_percentage_24h",
        // Legacy aliases.
        "volume_usd" => "volume_usd_24h",
        "volume_24h" => "volume_usd_24h",
        "volume_7d" => "volume_usd_7d",
        "volume_30d" => "volume_usd_30d",
        "transactions" => "txns_24h",
        "last_price_change_usd_24h" => "price_change_percentage_24h",
        "liquidity" => "liquidity_usd",
        _ => DEFAULT_SORT_FIELD,
    }
}

/// Map a token sort field (legacy or canonical) to the canonical value accepted
/// by `/networks/{network}/tokens/search`. Note that tokens/search rejects price
/// ordering, so "price_usd" falls back to the volume default.
pub fn map_token_sort_field(field: &str) -> &'static str {
    match field {
        // Canonical values pass through unchanged.
        "volume_usd_24h" => "volume_usd_24h",
        "volume_usd_7d" => "volume_usd_7d",
        "volume_usd_30d" => "volume_usd_30d",
        "liquidity_usd" => "liquidity_usd",
        "txns_24h" => "txns_24h",
        "fdv_usd" => "fdv_usd",
        "created_at" => "created_at",
        "price_change_percentage_24h" => "price_change_percentage_24h",
        // Legacy aliases.
        "volume_24h" => "volume_usd_24h",
        "volume_7d" => "volume_usd_7d",
        "volume_30d" => "volume_usd_30d",
        "txns" => "txns_24h",
        "price_change" => "price_change_percentage_24h",
        "fdv" => "fdv_usd",
        // tokens/search 400s on price ordering: fall back to volume.
        "price_usd" => "volume_usd_24h",
        _ => DEFAULT_SORT_FIELD,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_legacy_values_map_to_canonical() {
        assert_eq!(map_pool_sort_field("volume_usd"), "volume_usd_24h");
        assert_eq!(map_pool_sort_field("volume_24h"), "volume_usd_24h");
        assert_eq!(map_pool_sort_field("volume_7d"), "volume_usd_7d");
        assert_eq!(map_pool_sort_field("volume_30d"), "volume_usd_30d");
        assert_eq!(map_pool_sort_field("transactions"), "txns_24h");
        assert_eq!(
            map_pool_sort_field("last_price_change_usd_24h"),
            "price_change_percentage_24h"
        );
        assert_eq!(map_pool_sort_field("liquidity"), "liquidity_usd");
    }

    #[test]
    fn pool_canonical_values_pass_through() {
        for v in [
            "volume_usd_24h",
            "volume_usd_7d",
            "volume_usd_30d",
            "liquidity_usd",
            "txns_24h",
            "created_at",
            "price_usd",
            "price_change_percentage_24h",
        ] {
            assert_eq!(map_pool_sort_field(v), v);
        }
    }

    #[test]
    fn pool_unknown_value_falls_back_to_default() {
        assert_eq!(map_pool_sort_field("nonsense"), "volume_usd_24h");
        assert_eq!(map_pool_sort_field(""), "volume_usd_24h");
    }

    #[test]
    fn token_legacy_values_map_to_canonical() {
        assert_eq!(map_token_sort_field("volume_24h"), "volume_usd_24h");
        assert_eq!(map_token_sort_field("volume_7d"), "volume_usd_7d");
        assert_eq!(map_token_sort_field("volume_30d"), "volume_usd_30d");
        assert_eq!(map_token_sort_field("txns"), "txns_24h");
        assert_eq!(
            map_token_sort_field("price_change"),
            "price_change_percentage_24h"
        );
        assert_eq!(map_token_sort_field("fdv"), "fdv_usd");
    }

    #[test]
    fn token_price_ordering_falls_back_to_volume() {
        assert_eq!(map_token_sort_field("price_usd"), "volume_usd_24h");
    }

    #[test]
    fn token_canonical_values_pass_through() {
        for v in [
            "volume_usd_24h",
            "volume_usd_7d",
            "volume_usd_30d",
            "liquidity_usd",
            "txns_24h",
            "fdv_usd",
            "created_at",
            "price_change_percentage_24h",
        ] {
            assert_eq!(map_token_sort_field(v), v);
        }
    }

    #[test]
    fn token_unknown_value_falls_back_to_default() {
        assert_eq!(map_token_sort_field("nonsense"), "volume_usd_24h");
        assert_eq!(map_token_sort_field(""), "volume_usd_24h");
    }
}
