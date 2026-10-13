use tabled::settings::Style;
use tabled::{Table, Tabled};

use crate::commands::indicators::{IndicatorPoint, IndicatorResponse};
use crate::output::{print_dexpaprika_footer, print_more_results_hint};

/// Four decimal places: plenty for rsi (0-100), and enough to tell adjacent
/// macd/histogram candles apart without the noise of a raw f64 Display.
fn format_indicator_value(n: f64) -> String {
    format!("{n:.4}")
}

fn ts(point: &IndicatorPoint) -> String {
    point
        .timestamp
        .as_deref()
        .unwrap_or("-")
        .chars()
        .take(19)
        .collect()
}

fn opt(v: Option<f64>) -> String {
    v.map(format_indicator_value).unwrap_or_else(|| "-".into())
}

#[derive(Tabled)]
struct ValueRow {
    #[tabled(rename = "Timestamp")]
    timestamp: String,
    #[tabled(rename = "Value")]
    value: String,
}

#[derive(Tabled)]
struct MacdRow {
    #[tabled(rename = "Timestamp")]
    timestamp: String,
    #[tabled(rename = "MACD")]
    macd: String,
    #[tabled(rename = "Signal")]
    signal: String,
    #[tabled(rename = "Histogram")]
    histogram: String,
}

#[derive(Tabled)]
struct BbandsRow {
    #[tabled(rename = "Timestamp")]
    timestamp: String,
    #[tabled(rename = "Upper")]
    upper: String,
    #[tabled(rename = "Middle")]
    middle: String,
    #[tabled(rename = "Lower")]
    lower: String,
}

/// Render a pool or token indicator response. The column set follows the
/// `indicator` field the API returned rather than the `--indicator` flag the
/// caller typed, so a response is always rendered as what it actually is.
pub fn print_indicator_table(resp: &IndicatorResponse) {
    match resp.indicator.as_deref() {
        Some("macd") => print_macd_table(&resp.data),
        Some("bbands") => print_bbands_table(&resp.data),
        _ => print_value_table(&resp.data),
    }
    print_more_results_hint(resp.has_next_page, resp.next_cursor.as_deref());
    print_dexpaprika_footer();
}

fn print_value_table(data: &[IndicatorPoint]) {
    let rows: Vec<ValueRow> = data
        .iter()
        .map(|d| ValueRow {
            timestamp: ts(d),
            value: opt(d.value),
        })
        .collect();
    let table = Table::new(rows).with(Style::rounded()).to_string();
    println!("{table}");
}

fn print_macd_table(data: &[IndicatorPoint]) {
    let rows: Vec<MacdRow> = data
        .iter()
        .map(|d| MacdRow {
            timestamp: ts(d),
            macd: opt(d.macd),
            signal: opt(d.signal),
            histogram: opt(d.histogram),
        })
        .collect();
    let table = Table::new(rows).with(Style::rounded()).to_string();
    println!("{table}");
}

fn print_bbands_table(data: &[IndicatorPoint]) {
    let rows: Vec<BbandsRow> = data
        .iter()
        .map(|d| BbandsRow {
            timestamp: ts(d),
            upper: opt(d.upper),
            middle: opt(d.middle),
            lower: opt(d.lower),
        })
        .collect();
    let table = Table::new(rows).with(Style::rounded()).to_string();
    println!("{table}");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(ts: &str) -> IndicatorPoint {
        IndicatorPoint {
            timestamp: Some(ts.to_string()),
            value: None,
            macd: None,
            signal: None,
            histogram: None,
            upper: None,
            middle: None,
            lower: None,
        }
    }

    #[test]
    fn missing_values_render_as_a_dash() {
        assert_eq!(opt(None), "-");
    }

    #[test]
    fn present_values_round_to_four_places() {
        assert_eq!(opt(Some(54.38666301724629)), "54.3867");
    }

    #[test]
    fn timestamp_is_truncated_to_nineteen_characters() {
        let mut p = point("2026-10-13T06:00:00Z");
        p.timestamp = Some("2026-10-13T06:00:00.123456Z".to_string());
        assert_eq!(ts(&p), "2026-10-13T06:00:00");
    }

    #[test]
    fn missing_timestamp_renders_as_a_dash() {
        let mut p = point("x");
        p.timestamp = None;
        assert_eq!(ts(&p), "-");
    }
}
