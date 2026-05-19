//! `stream-reserves` subcommand: subscribe to block-level pool reserve
//! updates over Server-Sent Events.
//!
//! Two methods:
//! - `pool_reserves`: subscribe to one specific pool. Events fire when that
//!   pool's reserves change.
//! - `token_reserves`: subscribe to one token. Events fire for every pool
//!   containing that token (high volume on major assets).
//!
//! The reserves feed uses precision-safe JSON string encoding for the raw
//! integer fields (`block`, `previous_block`, `reserve`, `delta`) since
//! those routinely exceed `Number.MAX_SAFE_INTEGER` (53 bits).

use anyhow::{bail, Context, Result};
use futures::StreamExt;
use reqwest_eventsource::{Event, EventSource};
use serde::{Deserialize, Serialize};

use crate::client::ApiClient;
use crate::output::OutputFormat;

/// Per-token entry inside a `reserve_update` event.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TokenReserveStat {
    pub token_id: String,
    /// Raw on-chain reserve amount, encoded as a JSON string for precision.
    pub reserve: String,
    /// Change in reserve since `previous_block`, encoded as a JSON string.
    pub delta: String,
    pub price_usd: f64,
    pub reserve_usd: f64,
    pub delta_usd: f64,
}

/// Raw `reserve_update` SSE payload.
#[derive(Debug, Deserialize)]
pub struct RawReserveEvent {
    pub chain: String,
    pub pool_id: String,
    /// Block number, encoded as a JSON string for precision.
    pub block: String,
    /// Previous observed block, encoded as a JSON string. Omitted on the
    /// first event after subscribing.
    #[serde(default)]
    pub previous_block: Option<String>,
    pub tokens: Vec<TokenReserveStat>,
    pub total_reserve_usd: f64,
    pub total_delta_usd: f64,
}

/// CLI-facing event.
#[derive(Debug, Serialize)]
pub struct ReserveEvent {
    pub chain: String,
    pub pool_id: String,
    pub block: String,
    pub previous_block: Option<String>,
    pub tokens: Vec<TokenReserveStat>,
    pub total_reserve_usd: f64,
    pub total_delta_usd: f64,
}

impl From<RawReserveEvent> for ReserveEvent {
    fn from(raw: RawReserveEvent) -> Self {
        Self {
            chain: raw.chain,
            pool_id: raw.pool_id,
            block: raw.block,
            previous_block: raw.previous_block,
            tokens: raw.tokens,
            total_reserve_usd: raw.total_reserve_usd,
            total_delta_usd: raw.total_delta_usd,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct ReserveSubscription {
    chain: String,
    address: String,
    method: String,
}

const MAX_SUBSCRIPTIONS_PER_POST: usize = 25;

pub async fn execute(
    client: &ApiClient,
    network: Option<&str>,
    address: Option<&str>,
    method: &str,
    subscriptions_file: Option<&str>,
    limit: Option<usize>,
    output: OutputFormat,
) -> Result<()> {
    if limit == Some(0) {
        return Ok(());
    }

    if !matches!(method, "pool_reserves" | "token_reserves") {
        bail!(
            "Invalid --method '{method}'. Use 'pool_reserves' (one pool) \
             or 'token_reserves' (one token across all its pools)."
        );
    }

    if subscriptions_file.is_some() && (network.is_some() || address.is_some()) {
        bail!("Cannot use both <network> <address> and --subscriptions <file>. Pick one.");
    }

    if let Some(file) = subscriptions_file {
        stream_multi(client, file, limit, output).await
    } else {
        match (network, address) {
            (Some(net), Some(addr)) => stream_single(net, addr, method, limit, output).await,
            _ => bail!(
                "Provide either <network> <address> --method <pool_reserves|token_reserves> \
                 or --subscriptions <file.json>"
            ),
        }
    }
}

async fn stream_single(
    network: &str,
    address: &str,
    method: &str,
    limit: Option<usize>,
    output: OutputFormat,
) -> Result<()> {
    let url = format!(
        "https://streaming.dexpaprika.com/sse/reserves?method={method}&chain={network}&address={address}"
    );

    let mut es = EventSource::get(&url);
    let mut count = 0usize;

    loop {
        tokio::select! {
            event = es.next() => {
                match event {
                    Some(Ok(Event::Message(msg))) => {
                        if msg.event != "reserve_update" {
                            continue;
                        }
                        match serde_json::from_str::<RawReserveEvent>(&msg.data) {
                            Ok(raw) => {
                                let data = ReserveEvent::from(raw);
                                crate::output::stream_reserves::print_reserve_event(&data, output);
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
    let user_subs: Vec<serde_json::Value> = serde_json::from_str(&content).map_err(|e| {
        anyhow::anyhow!(
            "Invalid JSON in {file_path}: {e}\n\n\
             Expected format: [{{\n  \
               \"chain\": \"ethereum\",\n  \
               \"address\": \"0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640\",\n  \
               \"method\": \"pool_reserves\"\n\
             }}]"
        )
    })?;

    if user_subs.is_empty() {
        bail!("Subscription list in {file_path} is empty. Add at least one entry.");
    }

    let subs: Vec<ReserveSubscription> = user_subs
        .iter()
        .map(|t| ReserveSubscription {
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
            method: t
                .get("method")
                .and_then(|v| v.as_str())
                .unwrap_or("pool_reserves")
                .to_string(),
        })
        .collect();

    for (i, sub) in subs.iter().enumerate() {
        if sub.chain.is_empty() || sub.address.is_empty() {
            bail!(
                "Subscription at index {i} is missing \"chain\" or \"address\".\n\n\
                 Expected format: {{\"chain\": \"ethereum\", \"address\": \"0x...\", \"method\": \"pool_reserves\"}}"
            );
        }
        if !matches!(sub.method.as_str(), "pool_reserves" | "token_reserves") {
            bail!(
                "Subscription at index {i} has invalid method '{}'. \
                 Use 'pool_reserves' or 'token_reserves'.",
                sub.method
            );
        }
    }

    if subs.len() > MAX_SUBSCRIPTIONS_PER_POST {
        bail!(
            "Maximum {MAX_SUBSCRIPTIONS_PER_POST} subscriptions per POST stream. \
             You specified {}. Open multiple parallel streams if you need more \
             (up to 10 concurrent SSE streams per IP).",
            subs.len()
        );
    }

    let body = serde_json::to_string(&subs)?;

    let resp = client
        .http_client()
        .post("https://streaming.dexpaprika.com/sse/reserves")
        .header("Accept", "text/event-stream")
        .header("Content-Type", "application/json")
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
                        while let Some(boundary) = buffer.find("\n\n") {
                            let message = buffer[..boundary].to_string();
                            buffer.drain(..boundary + 2);
                            if let Some(event) = parse_reserve_message(&message) {
                                crate::output::stream_reserves::print_reserve_event(&event, output);
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

/// Parse one buffered SSE message into a `ReserveEvent`. Returns `None` for
/// non-`reserve_update` events (ping, warning, error) or malformed payloads.
fn parse_reserve_message(message: &str) -> Option<ReserveEvent> {
    let mut event_name: Option<&str> = None;
    let mut data: Option<&str> = None;

    for line in message.lines() {
        if let Some(rest) = line.strip_prefix("event:") {
            event_name = Some(rest.trim());
        } else if let Some(rest) = line.strip_prefix("data:") {
            data = Some(rest.trim_start());
        }
    }

    if event_name != Some("reserve_update") {
        return None;
    }
    let data = data?;
    serde_json::from_str::<RawReserveEvent>(data)
        .ok()
        .map(ReserveEvent::from)
}
