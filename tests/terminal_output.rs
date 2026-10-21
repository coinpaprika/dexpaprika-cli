//! Strings from the API reach the terminal in table mode. Token metadata is
//! chosen by whoever deployed the contract, so an ESC sequence in a name,
//! symbol or link must never be printed as-is. JSON mode is unchanged.

mod common;

use common::{cli, has_terminal_control, scratch_home, Reply, Stub};

/// Clear screen, retitle the window, write the clipboard (OSC 52), C1 CSI and a
/// bidi override, all JSON-escaped so the stub can serve them.
const HOSTILE: &str =
    r#"\u001b[2J\u001b[HEvil\u001b]0;FAKE TITLE\u0007\u001b]52;c;ZXZpbA==\u0007\u009b31m\u202ex"#;

fn assert_clean(what: &str, bytes: &[u8]) {
    assert!(
        !has_terminal_control(bytes),
        "{what} reached the terminal with control bytes: {:?}",
        String::from_utf8_lossy(bytes)
    );
}

#[test]
fn token_detail_table_strips_escape_sequences() {
    let home = scratch_home("token-detail");
    let body = format!(
        r#"{{"id":"0xabc","name":"{HOSTILE}","symbol":"{HOSTILE}","chain":"ethereum","website":"{HOSTILE}","telegram":"{HOSTILE}","twitter":"{HOSTILE}","description":"{HOSTILE}"}}"#
    );
    let stub = Stub::start(move |_| Reply::json(200, &body));
    let out = cli(&home)
        .args(["--base-url", &stub.base, "token", "ethereum", "0xabc"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_clean("token detail", &out);
    assert!(String::from_utf8_lossy(&out).contains("Evil"));
}

#[test]
fn search_lines_strip_escape_sequences() {
    let home = scratch_home("search");
    let body = format!(
        r#"{{"tokens":[{{"id":"0xabc","name":"{HOSTILE}","symbol":"{HOSTILE}","chain":"{HOSTILE}"}}],"pools":[{{"id":"0xdef","dex_name":"{HOSTILE}","chain":"{HOSTILE}"}}],"dexes":[{{"id":"{HOSTILE}","name":"{HOSTILE}","chain":"{HOSTILE}"}}]}}"#
    );
    let stub = Stub::start(move |_| Reply::json(200, &body));
    let out = cli(&home)
        .args(["--base-url", &stub.base, "search", "evil"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_clean("search output", &out);
}

#[test]
fn dexes_table_strips_escape_sequences() {
    let home = scratch_home("dexes");
    let body = format!(
        r#"{{"dexes":[{{"dex_id":"{HOSTILE}","dex_name":"{HOSTILE}","protocol":"{HOSTILE}"}}]}}"#
    );
    let stub = Stub::start(move |_| Reply::json(200, &body));
    let out = cli(&home)
        .args(["--base-url", &stub.base, "dexes", "ethereum"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_clean("dexes table", &out);
}

#[test]
fn an_error_body_is_stripped_before_it_reaches_stderr() {
    let home = scratch_home("error-body");
    let stub = Stub::start(|_| Reply {
        status: 400,
        body: b"\x1b]52;c;ZXZpbA==\x07\x1b[2Jbad request".to_vec(),
        headers: Vec::new(),
    });
    let err = cli(&home)
        .args(["--base-url", &stub.base, "pool", "ethereum", "0xabc"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    assert_clean("error body", &err);
    assert!(String::from_utf8_lossy(&err).contains("bad request"));
}

#[test]
fn the_usage_plan_is_stripped_in_config_show() {
    let home = scratch_home("usage-plan");
    let body = format!(r#"{{"plan":"{HOSTILE}"}}"#);
    let stub = Stub::start(move |_| Reply::json(200, &body));
    let out = cli(&home)
        .env("DEXPAPRIKA_API_KEY", "api_test_key_0000")
        .args(["--base-url", &stub.base, "config", "show"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_clean("/usage plan", &out);
    assert!(String::from_utf8_lossy(&out).contains("Evil"));
}

#[test]
fn json_mode_still_carries_the_value_escaped() {
    let home = scratch_home("json-mode");
    let body = format!(r#"{{"id":"0xabc","name":"{HOSTILE}"}}"#);
    let stub = Stub::start(move |_| Reply::json(200, &body));
    let out = cli(&home)
        .args([
            "--base-url",
            &stub.base,
            "--output",
            "json",
            "token",
            "ethereum",
            "0xabc",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8_lossy(&out);
    assert!(!text.contains('\u{1b}'), "{text:?}");
    assert!(text.contains(r"\u001b"), "{text:?}");
}
