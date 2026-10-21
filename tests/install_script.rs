//! Runs tests/install/run.sh, which drives install.sh against local fixtures
//! through a fake `curl`: a genuine release installs; a tampered archive, a
//! missing SHA256SUMS or entry, a `..` or absolute member, a symlink and a
//! non-version tag are all refused with nothing installed.

#[cfg(unix)]
#[test]
fn install_sh_verifies_the_checksum_and_contains_extraction() {
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/install/run.sh");
    let out = std::process::Command::new("sh")
        .arg(script)
        .output()
        .expect("run tests/install/run.sh");
    let report = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "install.sh cases failed:\n{report}");
}
