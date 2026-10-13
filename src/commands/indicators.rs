use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::client::ApiClient;
use crate::output::OutputFormat;

/// One data point of a technical indicator. The fields present depend on the
/// indicator: rsi, ema and vwap carry `value`; macd carries `macd`, `signal`
/// and `histogram`; bbands carries `upper`, `middle` and `lower`. `timestamp`
/// is always present and is the candle's open time.
#[derive(Debug, Deserialize, Serialize)]
pub struct IndicatorPoint {
    pub timestamp: Option<String>,
    pub value: Option<f64>,
    pub macd: Option<f64>,
    pub signal: Option<f64>,
    pub histogram: Option<f64>,
    pub upper: Option<f64>,
    pub middle: Option<f64>,
    pub lower: Option<f64>,
}

/// Response shape shared by `GET /networks/{network}/pools/{pool_address}/indicators`
/// and `GET /networks/{network}/tokens/{token_address}/indicators`. Only one of
/// `pool_address`/`token_address` is ever present, never both.
#[derive(Debug, Deserialize, Serialize)]
pub struct IndicatorResponse {
    pub network: Option<String>,
    #[serde(default)]
    pub pool_address: Option<String>,
    #[serde(default)]
    pub token_address: Option<String>,
    pub indicator: Option<String>,
    pub interval: Option<String>,
    #[serde(default)]
    pub parameters: Option<serde_json::Value>,
    #[serde(default)]
    pub data: Vec<IndicatorPoint>,
    pub has_next_page: Option<bool>,
    pub next_cursor: Option<String>,
}

/// Every flag `pool-indicator` and `token-indicator` accept, carried together
/// so the two commands build the same query string the same way. Every field
/// but `indicator` is optional, and only the ones the caller actually set go on
/// the wire: the API refuses a parameter that belongs to a different indicator
/// (`indicator=rsi&fast_period=3` is a 400), and `interval` has a server-side
/// default that differs by plan, so hard-coding one here would be guessing at
/// a number this CLI does not own.
#[derive(Debug, Default)]
pub struct IndicatorArgs {
    pub indicator: String,
    pub interval: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub limit: Option<usize>,
    pub cursor: Option<String>,
    pub period: Option<u32>,
    pub fast_period: Option<u32>,
    pub slow_period: Option<u32>,
    pub signal_period: Option<u32>,
    pub std_dev: Option<f64>,
}

/// Turn the set flags into query parameters, in the order the API docs list
/// them. Validates `--start`/`--end` the same way `pool-ohlcv` does, and
/// refuses `--cursor` together with `--start`: a cursor already encodes where
/// to resume, and the API does not expect both at once.
pub fn indicator_params(args: &IndicatorArgs) -> Result<Vec<(&'static str, String)>> {
    super::pools::check_time("--start", args.start.as_deref())?;
    super::pools::check_time("--end", args.end.as_deref())?;
    if args.cursor.is_some() && args.start.is_some() {
        anyhow::bail!(
            "--cursor and --start cannot be combined. A cursor already encodes where to \
             resume; pass --cursor on its own to fetch the next page."
        );
    }

    let mut params: Vec<(&'static str, String)> = vec![("indicator", args.indicator.clone())];
    if let Some(v) = &args.interval {
        params.push(("interval", v.clone()));
    }
    if let Some(v) = &args.start {
        params.push(("start", v.clone()));
    }
    if let Some(v) = &args.end {
        params.push(("end", v.clone()));
    }
    if let Some(v) = args.limit {
        params.push(("limit", v.to_string()));
    }
    if let Some(v) = &args.cursor {
        params.push(("cursor", v.clone()));
    }
    if let Some(v) = args.period {
        params.push(("period", v.to_string()));
    }
    if let Some(v) = args.fast_period {
        params.push(("fast_period", v.to_string()));
    }
    if let Some(v) = args.slow_period {
        params.push(("slow_period", v.to_string()));
    }
    if let Some(v) = args.signal_period {
        params.push(("signal_period", v.to_string()));
    }
    if let Some(v) = args.std_dev {
        params.push(("std_dev", v.to_string()));
    }
    Ok(params)
}

/// A technical indicator (rsi, ema, macd, bbands or vwap) for a pool. Available
/// on every plan, with the same interval and history limits as `pool-ohlcv`:
/// https://docs.dexpaprika.com/knowledge-base/rate-limits
pub async fn execute_pool_indicator(
    client: &ApiClient,
    network: &str,
    pool_address: &str,
    args: IndicatorArgs,
    output: OutputFormat,
    raw: bool,
) -> Result<()> {
    let params = indicator_params(&args)?;
    let param_refs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();

    let resp: IndicatorResponse = client
        .dexpaprika_get(
            &format!("/networks/{network}/pools/{pool_address}/indicators"),
            &param_refs,
        )
        .await?;

    match output {
        OutputFormat::Table => crate::output::indicators::print_indicator_table(&resp),
        OutputFormat::Json => {
            crate::output::print_json_wrapped(
                &resp,
                crate::output::ResponseMeta::dexpaprika(&format!(
                    "/pool/{network}/{pool_address}/indicators"
                )),
                raw,
            )?;
        }
    }
    Ok(())
}

/// A technical indicator (rsi, ema, macd, bbands or vwap) for a token. Needs a
/// Dev, Pro or Enterprise plan; keyless and a free key get 403 `plan_required`,
/// which `dexpaprika_get` turns into a message naming the plan (see
/// `plan_required_hint` in client.rs).
pub async fn execute_token_indicator(
    client: &ApiClient,
    network: &str,
    token_address: &str,
    args: IndicatorArgs,
    output: OutputFormat,
    raw: bool,
) -> Result<()> {
    let params = indicator_params(&args)?;
    let param_refs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();

    let resp: IndicatorResponse = client
        .dexpaprika_get(
            &format!("/networks/{network}/tokens/{token_address}/indicators"),
            &param_refs,
        )
        .await?;

    match output {
        OutputFormat::Table => crate::output::indicators::print_indicator_table(&resp),
        OutputFormat::Json => {
            crate::output::print_json_wrapped(
                &resp,
                crate::output::ResponseMeta::dexpaprika(&format!(
                    "/token/{network}/{token_address}/indicators"
                )),
                raw,
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_args(indicator: &str) -> IndicatorArgs {
        IndicatorArgs {
            indicator: indicator.to_string(),
            ..IndicatorArgs::default()
        }
    }

    #[test]
    fn only_indicator_is_sent_when_nothing_else_is_set() {
        let params = indicator_params(&base_args("rsi")).expect("should build");
        assert_eq!(params, vec![("indicator", "rsi".to_string())]);
    }

    #[test]
    fn rsi_sends_only_the_flags_the_caller_set() {
        let args = IndicatorArgs {
            period: Some(21),
            interval: Some("1h".to_string()),
            ..base_args("rsi")
        };
        let params = indicator_params(&args).expect("should build");
        assert_eq!(
            params,
            vec![
                ("indicator", "rsi".to_string()),
                ("interval", "1h".to_string()),
                ("period", "21".to_string()),
            ]
        );
    }

    #[test]
    fn macd_carries_its_three_periods_and_no_bare_period() {
        let args = IndicatorArgs {
            fast_period: Some(12),
            slow_period: Some(26),
            signal_period: Some(9),
            ..base_args("macd")
        };
        let params = indicator_params(&args).expect("should build");
        assert_eq!(
            params,
            vec![
                ("indicator", "macd".to_string()),
                ("fast_period", "12".to_string()),
                ("slow_period", "26".to_string()),
                ("signal_period", "9".to_string()),
            ]
        );
    }

    #[test]
    fn bbands_carries_period_and_std_dev() {
        let args = IndicatorArgs {
            period: Some(20),
            std_dev: Some(2.5),
            ..base_args("bbands")
        };
        let params = indicator_params(&args).expect("should build");
        assert_eq!(
            params,
            vec![
                ("indicator", "bbands".to_string()),
                ("period", "20".to_string()),
                ("std_dev", "2.5".to_string()),
            ]
        );
    }

    #[test]
    fn start_and_end_are_validated_like_ohlcv() {
        let args = IndicatorArgs {
            start: Some("yesterday".to_string()),
            ..base_args("rsi")
        };
        let err = indicator_params(&args).expect_err("bad --start should be rejected");
        assert!(err.to_string().contains("--start"));
    }

    #[test]
    fn relative_start_and_end_pass_through() {
        let args = IndicatorArgs {
            start: Some("-6h".to_string()),
            end: Some("-1h".to_string()),
            ..base_args("rsi")
        };
        let params = indicator_params(&args).expect("should build");
        assert_eq!(
            params,
            vec![
                ("indicator", "rsi".to_string()),
                ("start", "-6h".to_string()),
                ("end", "-1h".to_string()),
            ]
        );
    }

    #[test]
    fn cursor_with_start_is_rejected() {
        let args = IndicatorArgs {
            start: Some("-6h".to_string()),
            cursor: Some("1791882000".to_string()),
            ..base_args("rsi")
        };
        let err = indicator_params(&args).expect_err("combining cursor and start should fail");
        assert!(err.to_string().contains("--cursor"));
        assert!(err.to_string().contains("--start"));
    }

    #[test]
    fn cursor_alone_is_fine() {
        let args = IndicatorArgs {
            cursor: Some("1791882000".to_string()),
            ..base_args("rsi")
        };
        let params = indicator_params(&args).expect("should build");
        assert_eq!(
            params,
            vec![
                ("indicator", "rsi".to_string()),
                ("cursor", "1791882000".to_string()),
            ]
        );
    }

    /// One response object, copied from the live `/indicators` payload for
    /// `indicator=macd` captured on 2026-10-13.
    const LIVE_MACD_SAMPLE: &str = r#"{
      "network": "ethereum",
      "pool_address": "0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640",
      "indicator": "macd",
      "interval": "1h",
      "parameters": {"fast_period": 12, "signal_period": 9, "slow_period": 26},
      "data": [
        {"timestamp": "2026-10-13T06:00:00Z", "macd": -11.534946379135818, "signal": -22.75562121248413, "histogram": 11.220674833348312}
      ],
      "has_next_page": false,
      "next_cursor": ""
    }"#;

    #[test]
    fn macd_response_decodes_from_the_live_shape() {
        let resp: IndicatorResponse =
            serde_json::from_str(LIVE_MACD_SAMPLE).expect("live sample must decode");
        assert_eq!(
            resp.pool_address.as_deref(),
            Some("0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640")
        );
        assert_eq!(resp.token_address, None);
        assert_eq!(resp.indicator.as_deref(), Some("macd"));
        assert_eq!(resp.data.len(), 1);
        assert_eq!(resp.data[0].macd, Some(-11.534946379135818));
        assert_eq!(resp.data[0].signal, Some(-22.75562121248413));
        assert_eq!(resp.data[0].histogram, Some(11.220674833348312));
        assert_eq!(resp.data[0].value, None);
    }
}
