//! Positional ids must stay inside their own path segment. Before the fix an
//! address such as `../../usage` sent the request to a different path on the
//! same host, with the caller's key attached.

mod common;

use common::{cli, scratch_home, Reply, Stub};

const HOSTILE: &[&str] = &[
    "../../usage",
    "../key/info",
    "..%2F..%2Fusage",
    "a/b",
    "x?y=1",
    "x#y",
    "%2e%2e",
    "..\\x",
    "\u{2024}\u{2024}",
    "\u{ff0e}\u{ff0e}",
    "a\u{2215}b",
    "a\u{ff0f}b",
];

fn stub() -> Stub {
    Stub::start(|_| Reply::json(200, "{}"))
}

#[test]
fn a_hostile_pool_address_never_reaches_the_host() {
    let home = scratch_home("pool-address");
    for payload in HOSTILE {
        let stub = stub();
        let run = cli(&home)
            .args(["--base-url", &stub.base, "pool", "ethereum", payload])
            .assert();
        let seen = stub.seen();
        assert!(
            seen.is_empty(),
            "pool address {payload:?} produced requests {:?}",
            seen.iter().map(|s| &s.target).collect::<Vec<_>>()
        );
        run.failure();
    }
}

#[test]
fn a_hostile_token_address_never_reaches_the_host() {
    let home = scratch_home("token-address");
    for payload in HOSTILE {
        let stub = stub();
        let run = cli(&home)
            .args(["--base-url", &stub.base, "token", "ethereum", payload])
            .assert();
        let seen: Vec<_> = stub.seen().into_iter().map(|s| s.target).collect();
        assert!(seen.is_empty(), "token address {payload:?} sent {seen:?}");
        run.failure();
    }
}

#[test]
fn a_hostile_network_never_reaches_the_host() {
    let home = scratch_home("network");
    for payload in HOSTILE.iter().chain(&["eth:1", "eth main"]) {
        let stub = stub();
        let run = cli(&home)
            .args(["--base-url", &stub.base, "dexes", payload])
            .assert();
        let seen: Vec<_> = stub.seen().into_iter().map(|s| s.target).collect();
        assert!(seen.is_empty(), "network {payload:?} sent {seen:?}");
        run.failure();
    }
}

#[test]
fn control_a_real_pool_address_still_goes_out_as_typed() {
    let home = scratch_home("pool-ok");
    let stub = Stub::start(|_| Reply::json(404, r#"{"message":"not found"}"#));
    cli(&home)
        .args([
            "--base-url",
            &stub.base,
            "pool",
            "ethereum",
            "0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640",
        ])
        .assert()
        .failure();
    let seen = stub.seen();
    assert_eq!(seen.len(), 1);
    assert_eq!(
        seen[0].target,
        "/networks/ethereum/pools/0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640"
    );
}
