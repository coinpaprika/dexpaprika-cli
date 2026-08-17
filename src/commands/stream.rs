use anyhow::{bail, Context, Result};
use futures::StreamExt;
use reqwest_eventsource::{Event, EventSource};
use serde::{Deserialize, Serialize};

use crate::client::ApiClient;
use crate::output::OutputFormat;

/// Raw `token_price` SSE payload. Wire field names are mapped to readable
/// CLI struct names via serde renames.
#[derive(Debug, Deserialize)]
pub struct RawStreamEvent {
    pub address: String,
    pub chain: String,
    #[serde(rename = "price")]
    pub price_usd: String,
    pub timestamp: i64,
    #[serde(rename = "timestamp_price")]
    pub price_timestamp: i64,
}

/// CLI-facing event with readable field names.
#[derive(Debug, Serialize)]
pub struct StreamEvent {
    pub address: String,
    pub chain: String,
    pub price_usd: String,
    pub timestamp: i64,
    pub price_timestamp: i64,
}

impl From<RawStreamEvent> for StreamEvent {
    fn from(raw: RawStreamEvent) -> Self {
        Self {
            address: raw.address,
            chain: raw.chain,
            price_usd: raw.price_usd,
            timestamp: raw.timestamp,
            price_timestamp: raw.price_timestamp,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct StreamToken {
    chain: String,
    address: String,
    method: String,
}

/// Hard cap on subscriptions in a POST body. The server enforces the same
/// value and rejects oversized batches with HTTP 400.
const MAX_SUBSCRIPTIONS_PER_POST: usize = 25;

pub async fn execute(
    client: &ApiClient,
    network: Option<&str>,
    token_address: Option<&str>,
    tokens_file: Option<&str>,
    limit: Option<usize>,
    output: OutputFormat,
) -> Result<()> {
    // --limit 0 means emit nothing
    if limit == Some(0) {
        return Ok(());
    }

    if tokens_file.is_some() && (network.is_some() || token_address.is_some()) {
        bail!("Cannot use both <network> <token_address> and --tokens <file>. Pick one.");
    }

    if let Some(file) = tokens_file {
        stream_multi(client, file, limit, output).await
    } else {
        match (network, token_address) {
            (Some(net), Some(addr)) => stream_single(client, net, addr, limit, output).await,
            _ => bail!("Provide either <network> <token_address> or --tokens <file.json>"),
        }
    }
}

async fn stream_single(
    client: &ApiClient,
    network: &str,
    address: &str,
    limit: Option<usize>,
    output: OutputFormat,
) -> Result<()> {
    let url = format!(
        "https://streaming.dexpaprika.com/sse/prices?method=token_price&chain={network}&address={address}"
    );

    // EventSource::get cannot carry headers, so build the request first and let
    // the client attach the key when one is configured.
    let mut es = EventSource::new(client.authorize(client.http_client().get(&url)))?;
    let mut count = 0usize;

    loop {
        tokio::select! {
            event = es.next() => {
                match event {
                    Some(Ok(Event::Message(msg))) => {
                        // Skip ping/warning/error events; only handle token_price.
                        // EventSource has already parsed the event type for us.
                        if msg.event != "token_price" {
                            continue;
                        }
                        match serde_json::from_str::<RawStreamEvent>(&msg.data) {
                            Ok(raw) => {
                                let data = StreamEvent::from(raw);
                                crate::output::stream::print_stream_event(&data, output);
                                count += 1;
                                if let Some(lim) = limit {
                                    if count >= lim { break; }
                                }
                            }
                            Err(e) => {
                                eprintln!("Parse error: {e}");
                            }
                        }
                    }
                    Some(Ok(Event::Open)) => {}
                    Some(Err(e)) => {
                        bail!("Stream error: {e}");
                    }
                    None => break,
                }
            }
            _ = tokio::signal::ctrl_c() => {
                break;
            }
        }
    }

    Ok(())
}

async fn stream_multi(
    client: &ApiClient,
    file_path: &str,
    limit: Option<usize>,
    output: OutputFormat,
) -> Result<()> {
    let content = std::fs::read_to_string(file_path)
        .with_context(|| format!("failed to read {file_path}"))?;
    let user_tokens: Vec<serde_json::Value> = serde_json::from_str(&content).map_err(|e| {
        anyhow::anyhow!(
            "Invalid JSON in {file_path}: {e}\n\n\
             Expected format: [{{\n  \
               \"chain\": \"ethereum\",\n  \
               \"address\": \"0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2\"\n\
             }}]"
        )
    })?;

    if user_tokens.is_empty() {
        bail!("Token list in {file_path} is empty. Add at least one token.");
    }

    let tokens: Vec<StreamToken> = user_tokens
        .iter()
        .map(|t| StreamToken {
            chain: t
                .get("chain")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            address: t
                .get("address")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            method: "token_price".to_string(),
        })
        .collect();

    for (i, tok) in tokens.iter().enumerate() {
        if tok.chain.is_empty() || tok.address.is_empty() {
            bail!(
                "Token at index {i} is missing \"chain\" or \"address\".\n\n\
                 Expected format: {{\"chain\": \"ethereum\", \"address\": \"0x...\"}}"
            );
        }
    }

    if tokens.len() > MAX_SUBSCRIPTIONS_PER_POST {
        bail!(
            "Maximum {MAX_SUBSCRIPTIONS_PER_POST} tokens per POST stream. \
             You specified {}. Open multiple parallel streams if you need more \
             (up to 10 concurrent SSE streams per IP).",
            tokens.len()
        );
    }

    let body = serde_json::to_string(&tokens)?;

    let resp = client
        .authorize(
            client
                .http_client()
                .post("https://streaming.dexpaprika.com/sse/prices")
                .header("Accept", "text/event-stream")
                .header("Content-Type", "application/json"),
        )
        .body(body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        bail!("Stream POST error {status}: {body}");
    }

    let mut stream = resp.bytes_stream();
    let mut count = 0usize;
    let mut buffer = String::new();

    loop {
        tokio::select! {
            chunk = stream.next() => {
                match chunk {
                    Some(Ok(bytes)) => {
                        buffer.push_str(&String::from_utf8_lossy(&bytes));
                        // SSE messages are separated by a blank line. Buffer one
                        // message at a time, then dispatch on the parsed event
                        // type. Both `event:`/`data:` line orderings are valid
                        // and the server has emitted either during rollout, so
                        // a naive line-by-line parser would silently mis-dispatch.
                        while let Some(boundary) = buffer.find("\n\n") {
                            let message = buffer[..boundary].to_string();
                            buffer.drain(..boundary + 2);
                            if let Some(event) = parse_token_price_message(&message) {
                                crate::output::stream::print_stream_event(&event, output);
                                count += 1;
                                if let Some(lim) = limit {
                                    if count >= lim { return Ok(()); }
                                }
                            }
                        }
                    }
                    Some(Err(e)) => {
                        bail!("Stream error: {e}");
                    }
                    None => break,
                }
            }
            _ = tokio::signal::ctrl_c() => {
                break;
            }
        }
    }

    Ok(())
}

/// Parse one buffered SSE message into a `StreamEvent`. Returns `None` for
/// non-`token_price` events (ping, warning, error) or malformed payloads.
fn parse_token_price_message(message: &str) -> Option<StreamEvent> {
    let mut event_name: Option<&str> = None;
    let mut data: Option<&str> = None;

    for line in message.lines() {
        if let Some(rest) = line.strip_prefix("event:") {
            event_name = Some(rest.trim());
        } else if let Some(rest) = line.strip_prefix("data:") {
            data = Some(rest.trim_start());
        }
    }

    if event_name != Some("token_price") {
        return None;
    }
    let data = data?;
    serde_json::from_str::<RawStreamEvent>(data)
        .ok()
        .map(StreamEvent::from)
}
