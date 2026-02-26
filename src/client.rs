use anyhow::{bail, Result};
use reqwest::StatusCode;

pub struct ApiClient {
    http: reqwest::Client,
    dexpaprika_base: String,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::builder()
                .user_agent("dexpaprika-cli/0.1.0")
                .build()
                .expect("failed to build HTTP client"),
            dexpaprika_base: "https://api.dexpaprika.com".to_string(),
        }
    }

    pub async fn dexpaprika_get<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, &str)],
    ) -> Result<T> {
        let url = format!("{}{}", self.dexpaprika_base, path);
        let mut req = self.http.get(&url);

        if !params.is_empty() {
            req = req.query(params);
        }

        let resp = req.send().await?;
        let status = resp.status();

        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            match status {
                StatusCode::NOT_FOUND => {
                    bail!("Not found. Check the network ID and address. API response: {body}");
                }
                s if s.is_server_error() => {
                    bail!("DexPaprika API is temporarily unavailable. Try again shortly. ({status})");
                }
                _ => {
                    bail!("DexPaprika API error {status}: {body}");
                }
            }
        }

        Ok(resp.json().await?)
    }

    pub fn http_client(&self) -> &reqwest::Client {
        &self.http
    }
}
