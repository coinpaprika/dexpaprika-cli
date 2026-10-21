//! The release workflow builds and publishes the binaries install.sh hands to
//! users. These checks read the workflow files as text and pin down what keeps
//! that chain honest: actions pinned to commit SHAs, least-privilege tokens, a
//! locked build and a published SHA256SUMS.

const RELEASE: &str = include_str!("../.github/workflows/release.yml");
const CI: &str = include_str!("../.github/workflows/ci.yml");

fn uses_lines(workflow: &str) -> Vec<&str> {
    workflow
        .lines()
        .filter_map(|l| {
            l.trim_start()
                .trim_start_matches("- ")
                .strip_prefix("uses: ")
        })
        .collect()
}

#[test]
fn every_action_is_pinned_to_a_full_commit_sha() {
    for (name, wf) in [("release.yml", RELEASE), ("ci.yml", CI)] {
        let uses = uses_lines(wf);
        assert!(!uses.is_empty(), "{name}: no uses: lines found");
        for u in uses {
            let reference = u.split_whitespace().next().unwrap();
            let sha = reference.rsplit_once('@').map(|(_, r)| r).unwrap_or("");
            assert!(
                sha.len() == 40 && sha.bytes().all(|b| b.is_ascii_hexdigit()),
                "{name}: {reference} is not pinned to a commit SHA"
            );
        }
    }
}

#[test]
fn release_tokens_are_least_privilege() {
    assert!(
        RELEASE.contains("\npermissions: {}\n"),
        "release.yml must grant nothing at the workflow level"
    );
    let build = RELEASE
        .split("\n  release:\n")
        .next()
        .expect("a build job before the release job");
    assert!(build.contains("permissions:\n      contents: read"));
    assert!(!build.contains("contents: write"));
    let release = RELEASE.split("\n  release:\n").nth(1).expect("release job");
    assert!(release.contains("permissions:\n      contents: write"));
}

#[test]
fn the_release_build_is_locked() {
    let build_cmds: Vec<&str> = RELEASE
        .lines()
        .filter(|l| l.contains("cargo build"))
        .collect();
    assert!(!build_cmds.is_empty());
    for cmd in build_cmds {
        assert!(cmd.contains("--locked"), "unlocked build: {cmd}");
    }
}

#[test]
fn the_release_publishes_sha256sums() {
    assert!(RELEASE.contains("sha256sum dexpaprika-cli-* > SHA256SUMS"));
    assert!(RELEASE.contains("files: artifacts/*"));
}

#[test]
fn the_tag_name_reaches_shell_steps_through_the_environment() {
    for line in RELEASE.lines() {
        let t = line.trim_start();
        if t.starts_with("tar ") || t.starts_with("7z ") || t.starts_with("cd ") {
            assert!(!t.contains("${{"), "expression in a run step: {t}");
        }
    }
}
