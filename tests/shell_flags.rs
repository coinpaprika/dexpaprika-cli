//! A shell line is split into words and parsed like a command line, so a line
//! assembled from untrusted text could carry `--base-url` or `--api-key` and
//! send the configured key somewhere else. The shell refuses both per line.

mod common;

use common::{cli, scratch_home, Reply, Stub};

/// A second local host the environment points at, so a line that drops or
/// ignores the session's base URL still never leaves the machine.
fn decoy() -> Stub {
    Stub::start(|_| Reply::json(404, r#"{"message":"decoy"}"#))
}

#[test]
fn a_shell_line_cannot_redirect_the_key_with_base_url() {
    let home = scratch_home("shell-base-url");
    let stub = Stub::start(|_| Reply::json(404, r#"{"message":"not found"}"#));
    let decoy = decoy();
    for line in [
        format!("pool ethereum 0xabc --base-url={}\n", stub.base),
        format!("pool ethereum 0xabc --base-url {}\n", stub.base),
        format!("--base-url {} pool ethereum 0xabc\n", stub.base),
    ] {
        cli(&home)
            .env("DEXPAPRIKA_API_KEY", "api_shell_secret_0000")
            .env("DEXPAPRIKA_API_BASE_URL", &decoy.base)
            .arg("shell")
            .write_stdin(line.clone())
            .assert()
            .success();
        let leaked: Vec<_> = stub
            .seen()
            .into_iter()
            .filter(|s| s.header("authorization").is_some())
            .map(|s| s.target)
            .collect();
        assert!(
            leaked.is_empty(),
            "line {line:?} sent the key to the stub: {leaked:?}"
        );
    }
}

#[test]
fn a_shell_line_cannot_swap_the_key() {
    let home = scratch_home("shell-api-key");
    let stub = Stub::start(|_| Reply::json(404, r#"{"message":"not found"}"#));
    let decoy = decoy();
    cli(&home)
        .env("DEXPAPRIKA_API_BASE_URL", &decoy.base)
        .args(["--base-url", &stub.base, "shell"])
        .write_stdin("pool ethereum 0xabc --api-key api_injected_0000\n")
        .assert()
        .success();
    let sent: Vec<_> = stub
        .seen()
        .into_iter()
        .chain(decoy.seen())
        .filter_map(|s| s.header("authorization").map(str::to_string))
        .collect();
    assert!(sent.is_empty(), "an injected key was sent: {sent:?}");
}

#[test]
fn the_shell_uses_the_host_and_key_it_was_started_with() {
    let home = scratch_home("shell-session");
    let stub = Stub::start(|_| Reply::json(404, r#"{"message":"not found"}"#));
    let decoy = decoy();
    cli(&home)
        .env("DEXPAPRIKA_API_KEY", "api_session_key_0000")
        .env("DEXPAPRIKA_API_BASE_URL", &decoy.base)
        .args(["--base-url", &stub.base, "shell"])
        .write_stdin("pool ethereum 0xabc\n")
        .assert()
        .success();
    assert!(decoy.seen().is_empty(), "the line ignored the session host");
    let seen = stub.seen();
    assert_eq!(seen.len(), 1, "{seen:?}");
    assert_eq!(seen[0].target, "/networks/ethereum/pools/0xabc");
    assert_eq!(
        seen[0].header("authorization"),
        Some("api_session_key_0000")
    );
}

#[test]
fn a_double_dash_keeps_a_split_value_from_becoming_a_flag() {
    // For harnesses that build a command string: with `--` after the command
    // name, a value that word-splits into `--base-url=...` is an extra
    // positional and the command is refused instead of obeyed. Documented in
    // the README; this pins the parser behaviour the advice relies on.
    let home = scratch_home("double-dash");
    let stub = Stub::start(|_| Reply::json(404, r#"{"message":"not found"}"#));
    let smuggled = format!("--base-url={}", stub.base);
    let decoy = decoy();
    cli(&home)
        .env("DEXPAPRIKA_API_KEY", "api_dash_key_0000")
        .env("DEXPAPRIKA_API_BASE_URL", &decoy.base)
        .args(["pool", "--", "ethereum", "0xabc", &smuggled])
        .assert()
        .failure();
    assert!(stub.seen().is_empty());
    assert!(decoy.seen().is_empty());
}
