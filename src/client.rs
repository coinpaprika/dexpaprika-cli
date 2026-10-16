use anyhow::{bail, Result};
use reqwest::StatusCode;

pub struct ApiClient {
    http: reqwest::Client,
    dexpaprika_base: String,
    /// Optional API key. `None` means keyless, which is the default and works.
    api_key: Option<String>,
}

impl ApiClient {
    /// Client that sends an API key when one is configured.
    ///
    /// The key is the ENTIRE `Authorization` value, with nothing in front of it.
    ///
    /// The host is never inferred from the key. Keyless callers and free keys
    /// use api.dexpaprika.com; Dev, Pro and Enterprise keys use
    /// api-pro.dexpaprika.com, selected with `--base-url` or
    /// DEXPAPRIKA_API_BASE_URL (see `config::resolve_base_url`).
    pub fn with_api_key(api_key: Option<String>, base_url: String) -> Self {
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
            dexpaprika_base: base_url,
            api_key,
        }
    }

    /// Attach the API key to a request builder, if one is configured.
    ///
    /// Used by the streaming commands, which build their own requests rather
    /// than going through `dexpaprika_get`. Streaming authenticates the same way
    /// as REST: a bad key returns 401, and no key is served keyless, which covers
    /// price streams on the showcase tokens only (see `stream_refusal`).
    pub fn authorize(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match &self.api_key {
            Some(key) => req.header("Authorization", key),
            None => req,
        }
    }

    /// The REST host this client calls.
    pub fn base_url(&self) -> &str {
        &self.dexpaprika_base
    }

    /// Whether a key is configured, from any source (flag, environment, or the
    /// stored config). Paging past the first page of the four keyless-gated
    /// search endpoints needs one; everything else works the same with or
    /// without it.
    pub fn has_api_key(&self) -> bool {
        self.api_key.is_some()
    }

    pub async fn dexpaprika_get<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, &str)],
    ) -> Result<T> {
        let url = format!("{}{}", self.dexpaprika_base, path);
        let mut req = self.http.get(&url);

        if let Some(key) = &self.api_key {
            // The key alone is the entire Authorization value, nothing in front of it.
            req = req.header("Authorization", key);
        }

        if !params.is_empty() {
            req = req.query(params);
        }

        let resp = req.send().await?;
        let status = resp.status();

        // `x-api-warning` is the API telling us something about this specific
        // response without failing it (today: a keyless request that sent
        // `cursor` on one of the four search endpoints still gets its page, plus
        // a warning that paging keyless will soon stop working). To stderr only,
        // so JSON on stdout stays clean for scripts piping it.
        if let Some(warning) = resp
            .headers()
            .get("x-api-warning")
            .and_then(|v| v.to_str().ok())
        {
            eprintln!("Warning: {warning}");
        }

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

            // The technical-indicator endpoints refuse a plan gap with
            // `"error":"plan_required"` plus a `required_tier`. Checked live on
            // 2026-10-13: token OHLCV's 403 now carries that same envelope too
            // (`{"error":"plan_required","message":"this endpoint requires a Dev
            // or Pro plan","required_tier":"dev", ...}`), so this check runs
            // ahead of `plan_gate_hint` and is what actually fires for both
            // endpoints today. Keyed on the `error` field, the same pattern
            // `stream_refusal` uses for "preview_only", so an unrelated 403
            // falls through instead of picking up a hint that does not apply.
            if let Some(hint) = plan_required_hint(status, &body) {
                bail!("{hint}");
            }

            // Same idea, for the older shape of the same token-OHLCV gate: a
            // bare `{"message":"this endpoint requires a Dev or Pro plan"}` with
            // no `error` field, which `plan_required_hint` above does not match.
            // Kept as a fallback in case that shape reappears; live traffic hits
            // the envelope check above instead. Keyed on the message text rather
            // than the request path, the same way deprecation_hint keys on
            // "replacement".
            if let Some(hint) = plan_gate_hint(status, &body) {
                bail!("{hint}");
            }

            // api-pro.dexpaprika.com sits behind a Cloudflare edge that blocks
            // the request itself when the Authorization header does not carry
            // the expected key prefix, with Cloudflare's own 403 page (an HTML
            // body) rather than any DexPaprika JSON envelope. Every hint above
            // already returned None for a body that does not parse as JSON, so
            // reaching here with a 403 that still is not JSON means Cloudflare
            // refused the request before the API saw it. A 403 WITH a JSON
            // body (plan_required and friends) was handled above and never
            // reaches this check.
            if let Some(hint) = cloudflare_block_hint(status, &body) {
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

/// Turn a plan-gated REST refusal into a message naming the plan and where to
/// get one.
///
/// Token OHLCV is Dev, Pro and Enterprise only. A keyless or free-key caller
/// gets 403 with `{"message":"this endpoint requires a Dev or Pro plan"}` and,
/// without this, saw only the raw body. Matched on the exact message so an
/// unrelated 403 (a bad key, a different gate) falls through to the generic
/// error instead of picking up a hint that does not apply to it.
fn plan_gate_hint(status: StatusCode, body: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    let message = value.get("message")?.as_str()?;
    if message != "this endpoint requires a Dev or Pro plan" {
        return None;
    }
    Some(format!(
        "{message} ({status}). Token OHLCV needs a Dev or Pro plan; Dev covers the last 30 \
         days of history. Get one at https://dexpaprika.com/api/pricing, then run \
         `dexpaprika-cli config set-key <key>` and set \
         DEXPAPRIKA_API_BASE_URL=https://api-pro.dexpaprika.com. Manage keys at \
         https://console.dexpaprika.com."
    ))
}

/// Turn a `plan_required` refusal into a message naming the plan it needs.
///
/// The pool and token technical-indicator endpoints answer a plan gap with
/// `403` and an envelope carrying `"error":"plan_required"` and a
/// `required_tier`, rather than the fixed message `plan_gate_hint` matches on.
/// Token indicators are the gated case today (keyless and free keys see this;
/// pool indicators are open on every plan), but this is written generically so
/// any endpoint answering with the same envelope is covered without a
/// per-endpoint switch.
///
/// The same envelope also carries an optional `param`, naming the query
/// parameter that triggered the refusal. When `required_tier` is "free" and
/// `param` is "cursor", the refusal is the four search endpoints
/// (`/networks/{network}/pools/search`, `/pools/search`,
/// `/networks/{network}/tokens/search`, `/tokens/search`) telling a keyless
/// caller that paging past the first page needs a key. That case gets its own
/// wording naming paging specifically, since "needs at least the free plan" on
/// its own reads oddly for something any free key already covers in full; every
/// other combination falls through to the generic hint below.
fn plan_required_hint(status: StatusCode, body: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    if value.get("error")?.as_str()? != "plan_required" {
        return None;
    }
    let message = value
        .get("message")
        .and_then(|m| m.as_str())
        .unwrap_or("this endpoint needs a higher plan");
    let required_tier = value.get("required_tier").and_then(|t| t.as_str());
    let param = value.get("param").and_then(|p| p.as_str());

    if required_tier == Some("free") && param == Some("cursor") {
        return Some(format!(
            "{message} ({status}). Paging past the first page needs an API key; a free key \
             pages without limit. Get one at https://console.dexpaprika.com, then run \
             `dexpaprika-cli config set-key <key>`, set DEXPAPRIKA_API_KEY, or pass --api-key."
        ));
    }

    let mut hint = match required_tier {
        Some(tier) => format!("{message} ({status}). Needs at least the {tier} plan."),
        None => format!("{message} ({status})."),
    };
    hint.push_str(
        " See current plans at https://dexpaprika.com/api/pricing. Get or manage a key at \
         https://console.dexpaprika.com, then run `dexpaprika-cli config set-key <key>` and, \
         for Dev, Pro or Enterprise, set DEXPAPRIKA_API_BASE_URL=https://api-pro.dexpaprika.com.",
    );
    Some(hint)
}

/// Turn a non-JSON 403 body into a clear message about the key being refused
/// before the request reached the API.
///
/// Returns `None` for anything other than a 403, and `None` when the body
/// parses as JSON, so a `plan_required` (or any other) JSON 403 body keeps
/// going through its own handling untouched; this only fires on the case none
/// of the JSON-keyed hints above could ever match.
fn cloudflare_block_hint(status: StatusCode, body: &str) -> Option<String> {
    if status != StatusCode::FORBIDDEN {
        return None;
    }
    if serde_json::from_str::<serde_json::Value>(body).is_ok() {
        return None;
    }
    Some(format!(
        "The API key was refused before it reached the API ({status}). Check that the whole \
         key was pasted into the Authorization header with nothing in front of it. Keys are at \
         https://console.dexpaprika.com."
    ))
}

/// Turn a refused stream connection into a message that says what to do next.
///
/// Keyless streaming covers price streams on the showcase tokens only. Any other
/// token, and the reserves feed, is refused at connect time with 403 and
/// `"error":"preview_only"`; without this the caller saw a bare status code.
/// Keys on the `error` field, never on the message text.
///
/// streaming-pro.dexpaprika.com sits behind the same Cloudflare edge as
/// api-pro and blocks a connect attempt the same way, with the same HTML
/// block page, when the Authorization header does not carry the expected key
/// prefix. `cloudflare_block_hint` already returns `None` for any body that
/// parses as JSON, so `preview_only` and any other JSON-bodied refusal above
/// keeps going through its own handling untouched; this only replaces the
/// final fallback that used to pass the raw HTML straight through.
pub fn stream_refusal(status: StatusCode, body: &str) -> String {
    if let Some(hint) = deprecation_hint(status, body) {
        return hint;
    }
    let preview_only = serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| v.get("error")?.as_str().map(|e| e == "preview_only"))
        .unwrap_or(false);
    if preview_only {
        return format!(
            "This stream needs an API key ({status}, preview_only). Keyless streaming covers \
             price streams on the showcase tokens only; a free key opens every token and the \
             reserves feed. Get one at https://console.dexpaprika.com, then run \
             `dexpaprika-cli config set-key <key>` or pass --api-key."
        );
    }
    if let Some(hint) = cloudflare_block_hint(status, body) {
        return hint;
    }
    format!("Stream refused ({status}): {body}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_refusal_explains_preview_only() {
        // Body as served by streaming.dexpaprika.com/sse/reserves, keyless, 2026-09-28.
        let body = r#"{"error":"preview_only","tier":"keyless","message":"this stream requires an API key","links":{"register":"https://console.dexpaprika.com"}}"#;
        let msg = stream_refusal(StatusCode::FORBIDDEN, body);
        assert!(msg.contains("console.dexpaprika.com"));
        assert!(msg.contains("config set-key"));
        assert!(msg.contains("403"));
    }

    #[test]
    fn stream_refusal_keys_on_error_field_not_status() {
        let body = r#"{"error":"something_else","message":"nope"}"#;
        let msg = stream_refusal(StatusCode::FORBIDDEN, body);
        assert!(!msg.contains("config set-key"));
        assert!(msg.contains("something_else"));
    }

    #[test]
    fn stream_refusal_passes_through_non_json() {
        let msg = stream_refusal(StatusCode::UNAUTHORIZED, "bad key");
        assert!(msg.contains("401"));
        assert!(msg.contains("bad key"));
    }

    #[test]
    fn stream_refusal_explains_a_403_html_body() {
        // Cloudflare's own block page in front of streaming-pro.dexpaprika.com,
        // same page api-pro returns, for a key sent without the expected prefix.
        let body = "<html>Sorry, you have been blocked</html>";
        let msg = stream_refusal(StatusCode::FORBIDDEN, body);
        assert!(msg.contains("The API key was refused before it reached the API"));
        assert!(msg.contains("https://console.dexpaprika.com"));
        assert!(msg.contains("403"));
        assert!(!msg.contains("Bearer"));
        assert!(!msg.contains("Sorry, you have been blocked"));
    }

    #[test]
    fn stream_refusal_json_403_keeps_its_own_handling_not_the_cloudflare_hint() {
        // A JSON-bodied 403 that is neither preview_only nor the Cloudflare
        // page must keep falling through to the existing generic fallback,
        // not pick up the HTML-only hint.
        let body = r#"{"error":"something_else","message":"nope"}"#;
        let msg = stream_refusal(StatusCode::FORBIDDEN, body);
        assert!(!msg.contains("The API key was refused before it reached the API"));
        assert!(msg.contains("something_else"));
    }

    #[test]
    fn stream_refusal_non_403_status_unaffected_by_the_cloudflare_hint() {
        let msg = stream_refusal(StatusCode::UNAUTHORIZED, "<html>nope</html>");
        assert!(!msg.contains("The API key was refused before it reached the API"));
        assert!(msg.contains("401"));
    }

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

    #[test]
    fn plan_gate_hint_names_the_plan_and_links_pricing_and_console() {
        // The bare, pre-envelope shape this function was written for. Live
        // traffic on GET /networks/{network}/tokens/{address}/ohlcv has since
        // moved to the plan_required envelope (see the plan_required_hint
        // tests below), so this function is a fallback kept for that older
        // shape rather than what fires on the wire today.
        let body = r#"{"message":"this endpoint requires a Dev or Pro plan"}"#;
        let hint = plan_gate_hint(StatusCode::FORBIDDEN, body).expect("expected a hint");
        assert!(hint.contains("this endpoint requires a Dev or Pro plan"));
        assert!(hint.contains("Dev covers the last 30 days"));
        assert!(hint.contains("https://dexpaprika.com/api/pricing"));
        assert!(hint.contains("https://console.dexpaprika.com"));
        assert!(hint.contains("403"));
        assert!(!hint.contains("Bearer"));
    }

    #[test]
    fn plan_gate_hint_none_for_an_unrelated_403() {
        // A bad key, or any other 403, must not pick up the plan hint.
        let body = r#"{"message":"invalid API key"}"#;
        assert!(plan_gate_hint(StatusCode::FORBIDDEN, body).is_none());
    }

    #[test]
    fn plan_gate_hint_none_when_body_not_json() {
        assert!(plan_gate_hint(StatusCode::FORBIDDEN, "plain text error").is_none());
        assert!(plan_gate_hint(StatusCode::FORBIDDEN, "").is_none());
    }

    #[test]
    fn plan_required_hint_names_the_tier_and_links_pricing_and_console() {
        // Envelope captured live from GET
        // /networks/ethereum/tokens/{address}/indicators on 2026-10-13, keyless.
        let body = r#"{"error":"plan_required","tier":"keyless","message":"this endpoint requires a Dev or Pro plan","required_tier":"dev","links":{"coupon":"https://dexpaprika.com/api/pricing?coupon=START10","docs":"https://docs.dexpaprika.com/","pricing":"https://dexpaprika.com/api/pricing","register":"https://console.dexpaprika.com"}}"#;
        let hint = plan_required_hint(StatusCode::FORBIDDEN, body).expect("expected a hint");
        assert!(hint.contains("this endpoint requires a Dev or Pro plan"));
        assert!(hint.contains("dev plan"));
        assert!(hint.contains("https://dexpaprika.com/api/pricing"));
        assert!(hint.contains("https://console.dexpaprika.com"));
        assert!(hint.contains("403"));
        assert!(!hint.contains("Bearer"));
    }

    #[test]
    fn plan_required_hint_also_covers_the_token_ohlcv_gate_now() {
        // Checked live on 2026-10-13: GET
        // /networks/ethereum/tokens/{address}/ohlcv now answers the same
        // plan_required envelope, not the bare message plan_gate_hint was
        // written for. This is the body dexpaprika_get sees for that endpoint
        // today, and plan_required_hint must match it.
        let body = r#"{"error":"plan_required","tier":"keyless","message":"this endpoint requires a Dev or Pro plan","required_tier":"dev","links":{"coupon":"https://dexpaprika.com/api/pricing?coupon=START10","docs":"https://docs.dexpaprika.com/","pricing":"https://dexpaprika.com/api/pricing","register":"https://console.dexpaprika.com"}}"#;
        let hint = plan_required_hint(StatusCode::FORBIDDEN, body).expect("expected a hint");
        assert!(hint.contains("dev plan"));
    }

    #[test]
    fn plan_required_hint_works_without_required_tier() {
        let body = r#"{"error":"plan_required","message":"needs a higher plan"}"#;
        let hint = plan_required_hint(StatusCode::FORBIDDEN, body).expect("expected a hint");
        assert!(hint.contains("needs a higher plan"));
        assert!(hint.contains("https://dexpaprika.com/api/pricing"));
    }

    #[test]
    fn plan_required_hint_none_for_an_unrelated_403() {
        // A bad key, or the token-ohlcv plan_gate_hint case, must not pick this up.
        let body = r#"{"message":"this endpoint requires a Dev or Pro plan"}"#;
        assert!(plan_required_hint(StatusCode::FORBIDDEN, body).is_none());
        let body = r#"{"error":"invalid_key","message":"invalid API key"}"#;
        assert!(plan_required_hint(StatusCode::FORBIDDEN, body).is_none());
    }

    #[test]
    fn plan_required_hint_none_when_body_not_json() {
        assert!(plan_required_hint(StatusCode::FORBIDDEN, "plain text error").is_none());
        assert!(plan_required_hint(StatusCode::FORBIDDEN, "").is_none());
    }

    #[test]
    fn plan_required_hint_names_the_key_mechanisms_for_keyless_cursor_paging() {
        // The 403 body the four search endpoints will answer once blocking starts
        // (pools/search, tokens/search, with and without the {network} prefix)
        // for a keyless request that sent `cursor`, per the launch spec.
        let body = r#"{"error":"plan_required","tier":"keyless","required_tier":"free","param":"cursor","message":"Search results beyond the first page require an API key (free key: all pages). Create one in under a minute at https://console.dexpaprika.com","links":{"register":"https://console.dexpaprika.com","pricing":"https://dexpaprika.com/api/pricing","docs":"https://docs.dexpaprika.com/knowledge-base/rate-limits#pagination-rules","coupon":"https://dexpaprika.com/api/pricing?coupon=START10"}}"#;
        let hint = plan_required_hint(StatusCode::FORBIDDEN, body).expect("expected a hint");
        assert!(hint.contains("Paging past the first page needs an API key"));
        assert!(hint.contains("https://console.dexpaprika.com"));
        assert!(hint.contains("config set-key"));
        assert!(hint.contains("DEXPAPRIKA_API_KEY"));
        assert!(hint.contains("--api-key"));
        assert!(!hint.contains("Bearer"));
        assert!(!hint.contains('\u{2014}'));
        assert!(!hint.contains('\u{2013}'));
    }

    #[test]
    fn plan_required_hint_cursor_param_without_free_tier_falls_through_to_generic() {
        // A different endpoint could in principle answer plan_required with
        // param "cursor" while gating on a higher tier; that is not the paging
        // case this CLI knows about, so it must not pick up the paging wording.
        let body = r#"{"error":"plan_required","required_tier":"dev","param":"cursor","message":"needs a higher plan"}"#;
        let hint = plan_required_hint(StatusCode::FORBIDDEN, body).expect("expected a hint");
        assert!(!hint.contains("Paging past the first page"));
        assert!(hint.contains("dev plan"));
    }

    #[test]
    fn plan_required_hint_free_tier_without_cursor_param_falls_through_to_generic() {
        let body = r#"{"error":"plan_required","required_tier":"free","param":"interval","message":"needs a higher plan"}"#;
        let hint = plan_required_hint(StatusCode::FORBIDDEN, body).expect("expected a hint");
        assert!(!hint.contains("Paging past the first page"));
        assert!(hint.contains("free plan"));
    }

    #[test]
    fn plan_required_hint_without_a_param_field_still_parses() {
        // The existing envelope shape (token OHLCV, token indicators) carries no
        // `param` at all; adding that parse must not break it.
        let body = r#"{"error":"plan_required","tier":"keyless","message":"this endpoint requires a Dev or Pro plan","required_tier":"dev"}"#;
        let hint = plan_required_hint(StatusCode::FORBIDDEN, body).expect("expected a hint");
        assert!(hint.contains("dev plan"));
        assert!(!hint.contains("Paging past the first page"));
    }

    #[test]
    fn cloudflare_block_hint_explains_a_403_html_body() {
        // Cloudflare's own block page in front of api-pro.dexpaprika.com, seen
        // live on a key sent without the expected prefix.
        let body = "<html>Sorry, you have been blocked</html>";
        let hint = cloudflare_block_hint(StatusCode::FORBIDDEN, body).expect("expected a hint");
        assert!(hint.contains("The API key was refused before it reached the API"));
        assert!(hint.contains("https://console.dexpaprika.com"));
        assert!(hint.contains("403"));
        assert!(!hint.contains("Bearer"));
        assert!(!hint.contains("Sorry, you have been blocked"));
    }

    #[test]
    fn cloudflare_block_hint_none_for_a_json_403_body() {
        // plan_required and any other JSON-bodied 403 must keep going through
        // its own handling; this hint only fires on a body that is not JSON.
        let body =
            r#"{"error":"plan_required","required_tier":"dev","message":"needs a higher plan"}"#;
        assert!(cloudflare_block_hint(StatusCode::FORBIDDEN, body).is_none());
    }

    #[test]
    fn cloudflare_block_hint_none_for_a_non_403_status() {
        assert!(cloudflare_block_hint(StatusCode::UNAUTHORIZED, "<html>nope</html>").is_none());
        assert!(cloudflare_block_hint(StatusCode::NOT_FOUND, "<html>nope</html>").is_none());
    }

    #[test]
    fn has_api_key_reflects_what_the_client_was_built_with() {
        let keyless = ApiClient::with_api_key(None, "https://api.dexpaprika.com".to_string());
        assert!(!keyless.has_api_key());

        let keyed = ApiClient::with_api_key(
            Some("api_abc123".to_string()),
            "https://api.dexpaprika.com".to_string(),
        );
        assert!(keyed.has_api_key());
    }
}
