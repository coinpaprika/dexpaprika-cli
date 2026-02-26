use anyhow::{bail, Result};
use futures::StreamExt;
use reqwest_eventsource::{Event, EventSource};
use serde::{Deserialize, Serialize};

use crate::client::ApiClient;
use crate::output::OutputFormat;

#[derive(Debug, Deserialize, Serialize)]
pub struct StreamEvent {
    #[serde(rename = "a")]
    pub address: String,
    #[serde(rename = "c")]
    pub chain: String,
    #[serde(rename = "p")]
    pub price_usd: String,
    #[serde(rename = "t")]
    pub timestamp: i64,
    #[serde(rename = "t_p")]
    pub price_timestamp: i64,
}

#[derive(Debug, Deserialize, Serialize)]
struct StreamToken {
    chain: String,
    address: String,
    method: String,
}

pub async fn execute(
    client: &ApiClient,
    network: Option<&str>,
    token_address: Option<&str>,
    tokens_file: Option<&str>,
    limit: Option<usize>,
    output: OutputFormat,
) -> Result<()> {
    if let Some(file) = tokens_file {
        stream_multi(client, file, limit, output).await
    } else {
        match (network, token_address) {
            (Some(net), Some(addr)) => stream_single(net, addr, limit, output).await,
            _ => bail!("Provide either <network> <token_address> or --tokens <file.json>"),
        }
    }
}

async fn stream_single(network: &str, address: &str, limit: Option<usize>, output: OutputFormat) -> Result<()> {
    let url = format!(
        "https://streaming.dexpaprika.com/stream?method=t_p&chain={}&address={}",
        network, address
    );

    let mut es = EventSource::get(&url);
    let mut count = 0usize;

    loop {
        tokio::select! {
            event = es.next() => {
                match event {
                    Some(Ok(Event::Message(msg))) => {
                        match serde_json::from_str::<StreamEvent>(&msg.data) {
                            Ok(data) => {
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
                        eprintln!("Stream error: {e}");
                        break;
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

async fn stream_multi(client: &ApiClient, file_path: &str, limit: Option<usize>, output: OutputFormat) -> Result<()> {
    let content = std::fs::read_to_string(file_path)?;
    let user_tokens: Vec<serde_json::Value> = serde_json::from_str(&content)?;

    let tokens: Vec<StreamToken> = user_tokens.iter().map(|t| StreamToken {
        chain: t.get("chain").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        address: t.get("address").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        method: "t_p".to_string(),
    }).collect();

    if tokens.len() > 2000 {
        bail!("Maximum 2,000 tokens per stream connection. You specified {}.", tokens.len());
    }

    let body = serde_json::to_string(&tokens)?;

    let resp = client.http_client()
        .post("https://streaming.dexpaprika.com/stream")
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
                        while let Some(pos) = buffer.find('\n') {
                            let line = buffer[..pos].trim().to_string();
                            buffer = buffer[pos + 1..].to_string();
                            if let Some(data_str) = line.strip_prefix("data: ") {
                                if let Ok(event) = serde_json::from_str::<StreamEvent>(data_str) {
                                    crate::output::stream::print_stream_event(&event, output);
                                    count += 1;
                                    if let Some(lim) = limit {
                                        if count >= lim { return Ok(()); }
                                    }
                                }
                            }
                        }
                    }
                    Some(Err(e)) => {
                        eprintln!("Stream error: {e}");
                        break;
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
