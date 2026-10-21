//! A key on the command line is kept in shell history and is visible to other
//! local users in the process list. The CLI keeps accepting it, warns when it
//! does, and offers routes that keep the key out of argv: a prompt or stdin for
//! `config set-key`, DEXPAPRIKA_API_KEY, and a key file.

mod common;

use common::{cli, scratch_home, Reply, Stub};

const ARGV_WARNING: &str = "command-line argument";

fn usage_stub() -> Stub {
    Stub::start(|_| Reply::json(200, r#"{"plan":"free"}"#))
}

fn sent_keys(stub: &Stub) -> Vec<String> {
    stub.seen()
        .into_iter()
        .filter_map(|s| s.header("authorization").map(str::to_string))
        .collect()
}

#[test]
fn set_key_reads_the_key_from_stdin_without_an_argument() {
    let home = scratch_home("set-key-stdin");
    let stub = usage_stub();
    let out = cli(&home)
        .args(["--base-url", &stub.base, "config", "set-key"])
        .write_stdin("api_from_stdin_0000\n")
        .assert()
        .success()
        .get_output()
        .clone();
    assert_eq!(sent_keys(&stub), vec!["api_from_stdin_0000".to_string()]);
    let stored = std::fs::read_to_string(home.join(".dexpaprika/config.json")).unwrap();
    assert!(stored.contains("api_from_stdin_0000"));
    assert!(!String::from_utf8_lossy(&out.stderr).contains(ARGV_WARNING));
}

#[test]
fn a_key_file_is_used_and_nothing_warns() {
    let home = scratch_home("key-file");
    let path = home.join("dexpaprika.key");
    std::fs::write(&path, "api_from_file_0000\n").unwrap();
    for by_env in [false, true] {
        let stub = Stub::start(|_| Reply::json(404, r#"{"message":"not found"}"#));
        let mut cmd = cli(&home);
        cmd.args(["--base-url", &stub.base]);
        if by_env {
            cmd.env("DEXPAPRIKA_API_KEY_FILE", &path);
        } else {
            cmd.arg("--api-key-file").arg(&path);
        }
        let out = cmd
            .args(["pool", "ethereum", "0xabc"])
            .assert()
            .failure()
            .get_output()
            .clone();
        assert_eq!(
            sent_keys(&stub),
            vec!["api_from_file_0000".to_string()],
            "by_env={by_env}"
        );
        assert!(!String::from_utf8_lossy(&out.stderr).contains(ARGV_WARNING));
    }
}

#[test]
fn a_key_in_argv_still_works_but_warns() {
    let home = scratch_home("argv-key");
    let stub = Stub::start(|_| Reply::json(404, r#"{"message":"not found"}"#));
    let out = cli(&home)
        .args([
            "--base-url",
            &stub.base,
            "--api-key",
            "api_from_argv_0000",
            "pool",
            "ethereum",
            "0xabc",
        ])
        .assert()
        .failure()
        .get_output()
        .clone();
    assert_eq!(sent_keys(&stub), vec!["api_from_argv_0000".to_string()]);
    assert!(
        String::from_utf8_lossy(&out.stderr).contains(ARGV_WARNING),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let stub = usage_stub();
    let out = cli(&home)
        .args([
            "--base-url",
            &stub.base,
            "config",
            "set-key",
            "api_set_argv_0000",
        ])
        .assert()
        .success()
        .get_output()
        .clone();
    assert_eq!(sent_keys(&stub), vec!["api_set_argv_0000".to_string()]);
    assert!(String::from_utf8_lossy(&out.stderr).contains(ARGV_WARNING));
}

#[test]
fn an_unreadable_key_file_is_an_error_not_keyless() {
    let home = scratch_home("missing-key-file");
    let stub = Stub::start(|_| Reply::json(404, r#"{"message":"not found"}"#));
    cli(&home)
        .args(["--base-url", &stub.base, "--api-key-file"])
        .arg(home.join("nope.key"))
        .args(["pool", "ethereum", "0xabc"])
        .assert()
        .failure();
    assert!(stub.seen().is_empty());
}
