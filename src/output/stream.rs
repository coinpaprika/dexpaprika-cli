use crate::commands::stream::StreamEvent;
use crate::output::OutputFormat;

pub fn print_stream_event(event: &StreamEvent, output: OutputFormat) {
    match output {
        OutputFormat::Table => println!("{}", stream_line(event)),
        OutputFormat::Json => {
            if let Ok(json) = serde_json::to_string(event) {
                println!("{json}");
            }
        }
    }
}

/// One table-mode line for a price event. Chain, address and price arrive from
/// the stream, so the whole line is cleaned before it reaches the terminal.
fn stream_line(event: &StreamEvent) -> String {
    let time = chrono::DateTime::from_timestamp(event.timestamp, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| event.timestamp.to_string());
    let addr = crate::output::truncate_address(&event.address);
    crate::output::sanitize_line(&format!(
        "{}  {}  {}  ${}",
        time, event.chain, addr, event.price_usd
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hostile_chain_or_price_cannot_reach_the_terminal() {
        let event = StreamEvent {
            address: "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2".into(),
            chain: "eth\u{1b}[2J\u{1b}]0;FAKE\u{7}".into(),
            price_usd: "1\n2026-01-01 00:00:00  ethereum  0xfake  $0.01".into(),
            timestamp: 1_700_000_000,
            price_timestamp: 1_700_000_000,
        };
        let line = stream_line(&event);
        assert!(!line.contains('\u{1b}'), "{line:?}");
        assert!(!line.contains('\u{7}'), "{line:?}");
        assert!(!line.contains('\n'), "{line:?}");
    }
}
