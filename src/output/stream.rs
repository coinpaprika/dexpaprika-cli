use crate::commands::stream::StreamEvent;
use crate::output::OutputFormat;

pub fn print_stream_event(event: &StreamEvent, output: OutputFormat) {
    match output {
        OutputFormat::Table => {
            let time = chrono::DateTime::from_timestamp(event.timestamp, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| event.timestamp.to_string());
            let addr = crate::output::truncate_address(&event.address);
            println!("{}  {}  {}  ${}", time, event.chain, addr, event.price_usd);
        }
        OutputFormat::Json => {
            if let Ok(json) = serde_json::to_string(event) {
                println!("{json}");
            }
        }
    }
}
