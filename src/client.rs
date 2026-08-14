use anyhow::{bail, Result};
use reqwest::StatusCode;

pub struct ApiClient {
    http: reqwest::Client,
    dexpaprika_base: String,
}

impl ApiClient {
    pub fn new() -> Self {
        let ua = format!(
            "dexpaprika-cli/{} ({}/{})",
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS,
            std::env::consts::ARCH,
        );
        Self {
            http: reqwest::Client::builder()
                .user_agent(&ua)
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

            // Generic deprecation hint. When any error response carries a
            // "replacement" field, the API is telling us where a removed or
            // moved endpoint now lives. Surface it so future deprecations
            // self-document without waiting on a CLI release. This keys on the
            // field being present for ANY error status, not just 410.
            if let Some(hint) = deprecation_hint(status, &body) {
                bail!("{hint}");
            }

            match status {
                StatusCode::NOT_FOUND => {
                    bail!("Not found. Check the network ID and address. API response: {body}");
                }
                s if s.is_server_error() => {
                    bail!(
                        "DexPaprika API is temporarily unavailable. Try again shortly. ({status})"
                    );
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

/// Build a caller-facing deprecation message from an error response body.
///
/// If the body parses as JSON and carries a "replacement" field, the API is
/// signalling a removed or relocated endpoint. The returned message surfaces
/// both the replacement path and the server's own "message" (when present) so
/// the caller learns exactly where to go next.
///
/// Returns `None` when the body is not JSON or has no "replacement" field, so
/// the caller falls back to its existing status-specific handling. The parse is
/// deliberately defensive: a non-JSON body or a missing field never masks the
/// original error.
fn deprecation_hint(status: StatusCode, body: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    let replacement = value.get("replacement")?.as_str()?;

    match value.get("message").and_then(|m| m.as_str()) {
        Some(message) => Some(format!(
            "This endpoint was removed. Use {replacement} instead. ({status}) API says: {message}"
        )),
        None => Some(format!(
            "This endpoint was removed. Use {replacement} instead. ({status})"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deprecation_hint_surfaces_replacement_and_message() {
        let body = r#"{"code":410,"message":"endpoint removed","replacement":"/networks/{network}/pools/search"}"#;
        let hint = deprecation_hint(StatusCode::GONE, body).expect("expected a hint");
        assert!(hint.contains("/networks/{network}/pools/search"));
        assert!(hint.contains("endpoint removed"));
        assert!(hint.contains("410"));
    }

    #[test]
    fn deprecation_hint_works_without_message_field() {
        let body = r#"{"replacement":"/search"}"#;
        let hint = deprecation_hint(StatusCode::GONE, body).expect("expected a hint");
        assert!(hint.contains("/search"));
        assert!(!hint.contains("API says"));
    }

    #[test]
    fn deprecation_hint_fires_on_any_error_status() {
        // Not just 410: any error whose body carries "replacement" self-documents.
        let body = r#"{"message":"moved","replacement":"/v2/thing"}"#;
        let hint = deprecation_hint(StatusCode::BAD_REQUEST, body).expect("expected a hint");
        assert!(hint.contains("/v2/thing"));
    }

    #[test]
    fn deprecation_hint_none_when_no_replacement() {
        let body = r#"{"code":404,"message":"not found"}"#;
        assert!(deprecation_hint(StatusCode::NOT_FOUND, body).is_none());
    }

    #[test]
    fn deprecation_hint_none_when_body_not_json() {
        assert!(deprecation_hint(StatusCode::GONE, "plain text error").is_none());
        assert!(deprecation_hint(StatusCode::GONE, "").is_none());
    }
}
