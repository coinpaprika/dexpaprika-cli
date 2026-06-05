//! `stream-reserves` subcommand: subscribe to block-level reserve updates over
//! Server-Sent Events.
//!
//! Two methods, each with its own event shape:
//! - `pool_reserves`: subscribe to one specific pool. The server emits a
//!   method-named `pool_reserves` event carrying a nested `tokens` array (one
//!   entry per leg) plus `timestamp` and `block_timestamp`.
//! - `token_reserves`: subscribe to one token. Events fire for every pool
//!   containing that token (high volume on major assets). The server emits a
//!   method-named `token_reserves` event with a single flat token payload plus
//!   `updated_at` and `timestamp`.
//!
//! The legacy single `reserve_update` event no longer exists; this command
//! matches the two method-named events instead.
//!
//! Optional `request_id` correlation: pass `--request-id <0..4294967295>` and
//! the server echoes it back on a `request_id:` SSE line attached to each data
//! event (never on ping/warning/error). For multi-target streams the per-asset
//! body field defaults to the array index when omitted.
//!
//! The reserves feed uses precision-safe JSON string encoding for the raw
//! integer fields (`block`, `previous_block`, `reserve`, `delta`) since those
//! routinely exceed `Number.MAX_SAFE_INTEGER` (53 bits).

use anyhow::{bail, Context, Result};
use futures::StreamExt;
use reqwest_eventsource::{Event, EventSource};
use serde::{Deserialize, Serialize};

use crate::client::ApiClient;
use crate::output::OutputFormat;

/// Per-token reserve stat. Shared by both event shapes: `pool_reserves` nests a
/// `Vec` of these, while `token_reserves` carries the token fields flat at the
/// top level (and we lift them into a single-element vec on the CLI side).
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

/// Raw `pool_reserves` SSE payload: one event covers a whole pool, with a
/// nested per-token array and the new timestamp fields.
#[derive(Debug, Deserialize)]
pub struct RawPoolReserveEvent {
    pub chain: String,
    pub pool_id: String,
    /// Block number, encoded as a JSON string for precision.
    pub block: String,
    /// Previous observed block. Omitted on the first event after subscribing.
    #[serde(default)]
    pub previous_block: Option<String>,
    pub tokens: Vec<TokenReserveStat>,
    pub total_reserve_usd: f64,
    pub total_delta_usd: f64,
    /// Event emission time (unix seconds).
    #[serde(default)]
    pub timestamp: Option<i64>,
    /// On-chain block time (unix seconds).
    #[serde(default)]
    pub block_timestamp: Option<i64>,
}

/// Raw `token_reserves` SSE payload: a single token's reserve, flat (no nested
/// `tokens` array). Unlike `pool_reserves` it carries no `pool_id` or
/// `previous_block`, and adds `updated_at` (last reserve change for the token)
/// alongside `timestamp`. Verified against the live feed.
#[derive(Debug, Deserialize)]
pub struct RawTokenReserveEvent {
    pub chain: String,
    pub block: String,
    pub token_id: String,
    pub reserve: String,
    pub delta: String,
    pub price_usd: f64,
    pub reserve_usd: f64,
    pub delta_usd: f64,
    /// Event emission time (unix seconds).
    #[serde(default)]
    pub timestamp: Option<i64>,
    /// Last reserve change time for this token (unix seconds).
    #[serde(default)]
    pub updated_at: Option<i64>,
}

/// CLI-facing reserve event, normalized across both wire shapes. The single
/// token from a `token_reserves` event lands as a one-element `tokens` vec so
/// the output layer has a uniform shape to render.
#[derive(Debug, Serialize)]
pub struct ReserveEvent {
    /// Which method produced this event: "pool_reserves" or "token_reserves".
    pub method: &'static str,
    pub chain: String,
    /// Set for pool_reserves events. token_reserves carries no pool id (the
    /// token spans many pools), so it stays `None` there.
    pub pool_id: Option<String>,
    pub block: String,
    pub previous_block: Option<String>,
    pub tokens: Vec<TokenReserveStat>,
    pub total_reserve_usd: f64,
    pub total_delta_usd: f64,
    pub timestamp: Option<i64>,
    /// Set for pool_reserves events (on-chain block time).
    pub block_timestamp: Option<i64>,
    /// Set for token_reserves events (last reserve change for the token).
    pub updated_at: Option<i64>,
    /// Echoed correlation id, present only when the request carried one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<u32>,
}

impl From<RawPoolReserveEvent> for ReserveEvent {
    fn from(raw: RawPoolReserveEvent) -> Self {
        Self {
            method: "pool_reserves",
            chain: raw.chain,
            pool_id: Some(raw.pool_id),
            block: raw.block,
            previous_block: raw.previous_block,
            tokens: raw.tokens,
            total_reserve_usd: raw.total_reserve_usd,
            total_delta_usd: raw.total_delta_usd,
            timestamp: raw.timestamp,
            block_timestamp: raw.block_timestamp,
            updated_at: None,
            request_id: None,
        }
    }
}

impl From<RawTokenReserveEvent> for ReserveEvent {
    fn from(raw: RawTokenReserveEvent) -> Self {
        let token = TokenReserveStat {
            token_id: raw.token_id,
            reserve: raw.reserve,
            delta: raw.delta,
            price_usd: raw.price_usd,
            reserve_usd: raw.reserve_usd,
            delta_usd: raw.delta_usd,
        };
        // token_reserves carries no pool-level totals, so derive them from the
        // single token for a consistent output shape.
        let total_reserve_usd = token.reserve_usd;
        let total_delta_usd = token.delta_usd;
        Self {
            method: "token_reserves",
            chain: raw.chain,
            pool_id: None,
            block: raw.block,
            // token_reserves does not carry previous_block on the wire.
            previous_block: None,
            tokens: vec![token],
            total_reserve_usd,
            total_delta_usd,
            timestamp: raw.timestamp,
            block_timestamp: None,
            updated_at: raw.updated_at,
            request_id: None,
        }
    }
}

/// Decode one data payload into a `ReserveEvent`, dispatching on the SSE event
/// name. The event name is the method, so we deserialize the matching shape.
fn decode_reserve_payload(event_name: &str, data: &str) -> Option<ReserveEvent> {
    match event_name {
        "pool_reserves" => serde_json::from_str::<RawPoolReserveEvent>(data)
            .ok()
            .map(ReserveEvent::from),
        "token_reserves" => serde_json::from_str::<RawTokenReserveEvent>(data)
            .ok()
            .map(ReserveEvent::from),
        _ => None,
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct ReserveSubscription {
    chain: String,
    address: String,
    method: String,
    /// Per-asset correlation id echoed back on data events. Defaults to the
    /// array index server-side when omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    request_id: Option<u32>,
}

const MAX_SUBSCRIPTIONS_PER_POST: usize = 25;

#[allow(clippy::too_many_arguments)]
pub async fn execute(
    client: &ApiClient,
    network: Option<&str>,
    address: Option<&str>,
    method: &str,
    subscriptions_file: Option<&str>,
    request_id: Option<u32>,
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
            (Some(net), Some(addr)) => {
                stream_single(net, addr, method, request_id, limit, output).await
            }
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
    request_id: Option<u32>,
    limit: Option<usize>,
    output: OutputFormat,
) -> Result<()> {
    let mut url = format!(
        "https://streaming.dexpaprika.com/sse/reserves?method={method}&chain={network}&address={address}"
    );
    if let Some(rid) = request_id {
        url.push_str(&format!("&request_id={rid}"));
    }

    let mut es = EventSource::get(&url);
    let mut count = 0usize;

    loop {
        tokio::select! {
            event = es.next() => {
                match event {
                    Some(Ok(Event::Message(msg))) => {
                        // Only the two method-named events carry reserve data.
                        // Ping/warning/error are skipped. The reqwest-eventsource
                        // parser does not expose extra SSE lines like request_id,
                        // so on the single-target stream we already know the id
                        // and attach the one we sent.
                        if !matches!(msg.event.as_str(), "pool_reserves" | "token_reserves") {
                            continue;
                        }
                        match decode_reserve_payload(&msg.event, &msg.data) {
                            Some(mut data) => {
                                data.request_id = request_id;
                                crate::output::stream_reserves::print_reserve_event(&data, output);
                                count += 1;
                                if let Some(lim) = limit {
                                    if count >= lim { break; }
                                }
                            }
                            None => {
                                eprintln!("Parse error on {} event", msg.event);
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
               \"method\": \"pool_reserves\",\n  \
               \"request_id\": 1\n\
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
            // Optional per-asset correlation id. When omitted the server
            // defaults it to the array index.
            request_id: t
                .get("request_id")
                .and_then(|v| v.as_u64())
                .and_then(|n| u32::try_from(n).ok()),
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

/// Parse one buffered SSE message into a `ReserveEvent`. Dispatches on the
/// method-named event (`pool_reserves` / `token_reserves`) and lifts the
/// optional `request_id:` line (present only on data events) onto the result.
/// Returns `None` for ping/warning/error events or malformed payloads.
fn parse_reserve_message(message: &str) -> Option<ReserveEvent> {
    let mut event_name: Option<&str> = None;
    let mut data: Option<&str> = None;
    let mut request_id: Option<u32> = None;

    for line in message.lines() {
        if let Some(rest) = line.strip_prefix("event:") {
            event_name = Some(rest.trim());
        } else if let Some(rest) = line.strip_prefix("data:") {
            data = Some(rest.trim_start());
        } else if let Some(rest) = line.strip_prefix("request_id:") {
            request_id = rest.trim().parse::<u32>().ok();
        }
    }

    let event_name = event_name?;
    if !matches!(event_name, "pool_reserves" | "token_reserves") {
        return None;
    }
    let data = data?;
    let mut event = decode_reserve_payload(event_name, data)?;
    event.request_id = request_id;
    Some(event)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Real `pool_reserves` data payload captured from the live feed.
    const POOL_RESERVES_DATA: &str = r#"{"chain":"ethereum","pool_id":"0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640","block":"25236702","tokens":[{"token_id":"0xa0b8","reserve":"26891112547557","delta":"13999711886","price_usd":1.0,"reserve_usd":26892684.6,"delta_usd":14000.5},{"token_id":"0xc02a","reserve":"34775607852594224217028","delta":"-7459083661935418790","price_usd":1876.4,"reserve_usd":65253466.5,"delta_usd":-13996.3}],"total_reserve_usd":92146151.27,"total_delta_usd":4.19,"timestamp":1780487749,"block_timestamp":1780487747}"#;

    // Real `token_reserves` data payload captured from the live feed: flat, no
    // pool_id, no previous_block, carries updated_at.
    const TOKEN_RESERVES_DATA: &str = r#"{"chain":"ethereum","token_id":"0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48","reserve":"1602038276073898","delta":"73537469563","block":"25236698","price_usd":1.00009,"reserve_usd":1602184632.13,"delta_usd":73544.18,"updated_at":1780487699,"timestamp":1780487701}"#;

    #[test]
    fn decodes_pool_reserves_with_nested_tokens() {
        let event = decode_reserve_payload("pool_reserves", POOL_RESERVES_DATA).unwrap();
        assert_eq!(event.method, "pool_reserves");
        assert_eq!(
            event.pool_id.as_deref(),
            Some("0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640")
        );
        assert_eq!(event.tokens.len(), 2);
        assert_eq!(event.block_timestamp, Some(1780487747));
        assert!(event.updated_at.is_none());
    }

    #[test]
    fn decodes_token_reserves_flat_into_single_token() {
        let event = decode_reserve_payload("token_reserves", TOKEN_RESERVES_DATA).unwrap();
        assert_eq!(event.method, "token_reserves");
        // token_reserves carries no pool id; the single token lifts into a
        // one-element vec and drives the derived totals.
        assert!(event.pool_id.is_none());
        assert!(event.previous_block.is_none());
        assert_eq!(event.tokens.len(), 1);
        assert_eq!(event.updated_at, Some(1780487699));
        assert!(event.block_timestamp.is_none());
        assert_eq!(event.total_reserve_usd, 1602184632.13);
    }

    #[test]
    fn legacy_reserve_update_event_is_no_longer_decoded() {
        // The old single event name must not match anything now.
        assert!(decode_reserve_payload("reserve_update", POOL_RESERVES_DATA).is_none());
    }

    #[test]
    fn parses_message_and_lifts_request_id_line() {
        let message = format!("event: token_reserves\nrequest_id: 7\ndata: {TOKEN_RESERVES_DATA}");
        let event = parse_reserve_message(&message).unwrap();
        assert_eq!(event.method, "token_reserves");
        assert_eq!(event.request_id, Some(7));
    }

    #[test]
    fn ping_message_is_skipped() {
        let message = "event: ping\ndata: {\"time\":1780487708}";
        assert!(parse_reserve_message(message).is_none());
    }

    #[test]
    fn message_without_request_id_leaves_it_none() {
        let message = format!("event: pool_reserves\ndata: {POOL_RESERVES_DATA}");
        let event = parse_reserve_message(&message).unwrap();
        assert!(event.request_id.is_none());
    }
}
