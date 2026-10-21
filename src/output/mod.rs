use anyhow::Result;
use chrono::Utc;
use serde::Serialize;
use tabled::settings::object::{Columns, Segment};
use tabled::settings::{Format, Modify, Style, Width};
use tabled::Table;

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
}

// --- Attribution / _meta wrapper ---

#[derive(Serialize)]
pub struct WrappedResponse<'a, T: Serialize> {
    pub data: &'a T,
    pub _meta: ResponseMeta,
}

#[derive(Serialize)]
pub struct ResponseMeta {
    pub source: String,
    pub url: String,
    pub api_docs: String,
    pub attribution: String,
    pub timestamp: String,
}

impl ResponseMeta {
    pub fn dexpaprika(entity_path: &str) -> Self {
        Self {
            source: "DexPaprika".into(),
            url: format!("https://dexpaprika.com{entity_path}"),
            api_docs: "https://api.dexpaprika.com".into(),
            attribution: "Powered by DexPaprika · Free DEX & DeFi data".into(),
            timestamp: Utc::now().to_rfc3339(),
        }
    }
}

// --- Terminal safety ---

/// Whether a terminal would act on this character instead of showing it: C0
/// controls (ESC among them), DEL, C1 controls, and the bidi embedding and
/// isolate controls that can make one string display as another.
fn is_terminal_control(c: char) -> bool {
    c.is_control() || matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
}

/// Make API, on-chain or otherwise untrusted text safe to print in table mode.
///
/// Token names, symbols, links, DEX names and error bodies come from whoever
/// deployed the contract or answered the request, and printed raw an ESC
/// sequence in one can clear the screen, retitle the window or, on terminals
/// that honour OSC 52, write the clipboard. Every control character is
/// dropped except newline and tab, which stay so multi-line text keeps its
/// shape. JSON output does not go through here: serde_json escapes it.
pub fn sanitize_for_terminal(s: &str) -> String {
    s.chars()
        .filter(|&c| c == '\n' || c == '\t' || !is_terminal_control(c))
        .collect()
}

/// Same as [`sanitize_for_terminal`] for text that must stay on one line, such
/// as a field in a streamed event: newline, carriage return and tab become a
/// space, so a value cannot start a fake line of output.
pub fn sanitize_line(s: &str) -> String {
    s.chars()
        .filter_map(|c| match c {
            '\n' | '\r' | '\t' => Some(' '),
            c if is_terminal_control(c) => None,
            c => Some(c),
        })
        .collect()
}

/// Sanitize every cell of a table before it is laid out, so the widths are
/// measured on what will actually be shown.
pub fn safe_table(mut table: Table) -> Table {
    table.modify(Segment::all(), Format::content(sanitize_for_terminal));
    table
}

// --- Shared output helpers ---

pub fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut truncated: String = s.chars().take(max.saturating_sub(1)).collect();
    truncated.push('\u{2026}');
    truncated
}

/// Shorten an id to its first six and last four characters.
///
/// Counted in characters, not bytes: ids come from the API and the stream, and
/// slicing bytes would abort the process on a multi-byte character that
/// straddles the cut.
pub fn truncate_address(addr: &str) -> String {
    let len = addr.chars().count();
    if len <= 13 {
        return addr.to_string();
    }
    let head: String = addr.chars().take(6).collect();
    let tail: String = addr.chars().skip(len - 4).collect();
    format!("{head}...{tail}")
}

pub fn format_usd(n: f64) -> String {
    if n.abs() >= 1_000_000_000_000.0 {
        format!("${:.1}T", n / 1_000_000_000_000.0)
    } else if n.abs() >= 1_000_000_000.0 {
        format!("${:.1}B", n / 1_000_000_000.0)
    } else if n.abs() >= 1_000_000.0 {
        format!("${:.1}M", n / 1_000_000.0)
    } else if n.abs() >= 1_000.0 {
        format!("${:.1}K", n / 1_000.0)
    } else {
        format!("${n:.2}")
    }
}

pub fn format_price(n: f64) -> String {
    if n >= 1.0 {
        format!("${n:.2}")
    } else if n >= 0.01 {
        format!("${n:.4}")
    } else {
        format!("${n:.8}")
    }
}

pub fn format_percent(n: f64) -> String {
    if n >= 0.0 {
        format!("+{n:.2}%")
    } else {
        format!("{n:.2}%")
    }
}

#[allow(dead_code)]
pub fn print_json<T: Serialize>(data: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(data)?);
    Ok(())
}

pub fn print_json_wrapped<T: Serialize>(data: &T, meta: ResponseMeta, raw: bool) -> Result<()> {
    if raw {
        println!("{}", serde_json::to_string_pretty(data)?);
    } else {
        let wrapped = WrappedResponse { data, _meta: meta };
        println!("{}", serde_json::to_string_pretty(&wrapped)?);
    }
    Ok(())
}

pub fn print_detail_table(rows: Vec<[String; 2]>) {
    println!("{}", render_detail_table(rows));
}

fn render_detail_table(rows: Vec<[String; 2]>) -> String {
    safe_table(Table::from_iter(rows))
        .with(Style::rounded())
        .with(Modify::new(Columns::first()).with(Width::wrap(20)))
        .with(Modify::new(Columns::last()).with(Width::wrap(80)))
        .to_string()
}

pub fn print_dexpaprika_footer() {
    println!("\n Data: DexPaprika (dexpaprika.com) \u{00b7} Free API: api.dexpaprika.com");
}

/// Print a cursor-pagination hint for the unified search endpoints. The search
/// API is cursor-based (no page numbers), so when more results exist we surface
/// the opaque cursor the caller can pass to fetch the next page.
pub fn print_more_results_hint(has_next_page: Option<bool>, next_cursor: Option<&str>) {
    if has_next_page == Some(true) {
        match next_cursor {
            Some(c) if !c.is_empty() => {
                println!(
                    "  More results available. next_cursor: {}",
                    sanitize_line(c)
                );
            }
            _ => println!("  More results available."),
        }
    }
}

/// Same as `print_more_results_hint`, for the four search endpoints that now
/// gate paging on a key (`/networks/{network}/pools/search`, `/pools/search`,
/// `/networks/{network}/tokens/search`, `/tokens/search`): a keyless caller
/// would otherwise copy `next_cursor` into another run and only find out about
/// the gate from the API's own refusal. `has_key` is whether this run had one
/// configured at all (`ApiClient::has_api_key`); when it does, the hint stays
/// exactly what `print_more_results_hint` already prints.
pub fn print_search_more_results_hint(
    has_next_page: Option<bool>,
    next_cursor: Option<&str>,
    has_key: bool,
) {
    print_more_results_hint(has_next_page, next_cursor);
    if has_next_page == Some(true) && !has_key {
        println!(
            "  Paging needs an API key (a free key pages without limit). Get one at \
             https://console.dexpaprika.com, then run `dexpaprika-cli config set-key`."
        );
    }
}

macro_rules! detail_field {
    ($rows:expr, $label:expr, $val:expr) => {
        $rows.push([$label.into(), $val]);
    };
}

// No `pub(crate) use` re-export: macro_rules! is textually scoped, so every
// module declared below this point already sees detail_field!. The re-export
// and the matching `use crate::output::{detail_field}` lines in four submodules
// were redundant, and clippy on a current toolchain fails the build over them.
pub mod indicators;
pub mod networks;
pub mod pools;
pub mod search;
pub mod stats;
pub mod status;
pub mod stream;
pub mod stream_reserves;
pub mod tokens;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_short_string_unchanged() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn truncate_long_string_appends_ellipsis() {
        let result = truncate("hello world", 5);
        assert_eq!(result.chars().count(), 5);
        assert!(result.ends_with('\u{2026}'));
    }

    #[test]
    fn format_usd_billions() {
        assert_eq!(format_usd(1_500_000_000.0), "$1.5B");
    }

    #[test]
    fn format_usd_small() {
        assert_eq!(format_usd(42.5), "$42.50");
    }

    const HOSTILE: &str = "\u{1b}[2J\u{1b}[HEvil\u{1b}]0;FAKE TITLE\u{7}\u{1b}]52;c;ZXZpbA==\u{7}\u{7f}\u{9b}31m\u{202e}x";

    #[test]
    fn sanitize_for_terminal_drops_every_control_but_newline_and_tab() {
        let clean = sanitize_for_terminal(&format!("{HOSTILE}\nline two\tcol\r"));
        assert!(
            !clean
                .chars()
                .any(|c| c != '\n' && c != '\t' && is_terminal_control(c)),
            "{clean:?}"
        );
        assert!(clean.contains("Evil"));
        assert!(clean.contains("\nline two\tcol"));
    }

    #[test]
    fn sanitize_line_keeps_text_on_one_line() {
        let clean = sanitize_line("a\nb\rc\td\u{1b}[1m");
        assert_eq!(clean, "a b c d[1m");
    }

    #[test]
    fn a_detail_table_never_carries_escape_bytes() {
        let table = render_detail_table(vec![
            ["Name".into(), HOSTILE.into()],
            ["Website".into(), "\u{1b}]8;;https://evil.test\u{7}x".into()],
        ]);
        assert!(!table.contains('\u{1b}'), "{table:?}");
        assert!(!table.contains('\u{7}'), "{table:?}");
        assert!(!table.contains('\u{9b}'), "{table:?}");
        assert!(table.contains("Evil"));
    }

    #[test]
    fn truncate_address_does_not_split_a_multi_byte_character() {
        // Byte 6 and the start of the last four bytes both fall inside a
        // multi-byte character here; byte slicing panicked on this.
        let id = "0xa\u{e9}\u{e9}abcdefgh\u{e9}a\u{e9}";
        assert_eq!(truncate_address(id), "0xa\u{e9}\u{e9}a...h\u{e9}a\u{e9}");
        // A head that is all multi-byte characters: six characters, not six bytes.
        let wide = "\u{1f600}\u{1f600}\u{1f600}\u{1f600}\u{1f600}\u{1f600}\u{1f600}0123456789";
        assert_eq!(
            truncate_address(wide),
            "\u{1f600}\u{1f600}\u{1f600}\u{1f600}\u{1f600}\u{1f600}...6789"
        );
        assert_eq!(truncate_address("\u{e9}\u{e9}"), "\u{e9}\u{e9}");
    }

    #[test]
    fn truncate_address_short() {
        assert_eq!(truncate_address("0x1234567890abcdef"), "0x1234...cdef");
    }
}
