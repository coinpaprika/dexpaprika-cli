//! Network ids and on-chain addresses on their way into a REST path.
//!
//! Positional arguments used to be pasted into the path as typed, so an address
//! like `../../usage` or `x?y=1` turned the request into a different one on the
//! same host, under the caller's key. Every id now goes through here first:
//! validate, then encode as exactly one path segment. `client::request_url`
//! then checks that the URL parser kept the path we built.

use anyhow::{bail, Result};

/// A network id as the API spells it: lowercase letters, digits, `_` and `-`.
///
/// Input is lowercased first, so `Ethereum` still means `ethereum`. Anything
/// else is refused before a request is built. The returned value needs no
/// further encoding: every allowed character is safe in a path segment.
pub fn network(raw: &str) -> Result<String> {
    let id = raw.to_ascii_lowercase();
    let valid = !id.is_empty()
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-');
    if !valid {
        bail!(
            "Invalid network {raw:?}. A network id is lowercase letters, digits, '_' and '-', \
             for example ethereum, solana or bsc. Run `dexpaprika-cli networks` for the list."
        );
    }
    Ok(id)
}

/// Refuse an address that could change the shape of a URL.
///
/// Rejected: empty, `.` and `..`, anything outside printable ASCII (which also
/// covers whitespace, control characters and look-alikes such as U+2024 or
/// U+FF0E), and the characters `/`, `\`, `?`, `#` and `%`. Refusing `%` outright
/// means a percent-encoded `/`, `.` or `%` can never reach the server, so there
/// is nothing left to find after one decode. No address format on any network
/// the API covers uses these characters.
pub fn check_address(raw: &str, what: &str) -> Result<()> {
    if raw.is_empty() || raw == "." || raw == ".." {
        bail!("Invalid {what} {raw:?}: expected an on-chain address.");
    }
    if let Some(bad) = raw
        .chars()
        .find(|c| !c.is_ascii_graphic() || matches!(c, '/' | '\\' | '?' | '#' | '%'))
    {
        bail!(
            "Invalid {what} {raw:?}: {bad:?} cannot appear in an address. Pass the address \
             exactly as it appears on chain."
        );
    }
    Ok(())
}

/// Validate an address, then encode it as one path segment.
pub fn address(raw: &str, what: &str) -> Result<String> {
    check_address(raw, what)?;
    Ok(encode_segment(raw))
}

/// Percent-encode everything that is not a path-segment character (RFC 3986
/// `pchar`). Unreserved characters, sub-delims, `:` and `@` stay as they are, so
/// every real address goes out on the wire exactly as it did before; `/` is
/// always encoded, so the value can never become two segments.
fn encode_segment(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for b in raw.bytes() {
        let keep = b.is_ascii_alphanumeric()
            || matches!(
                b,
                b'-' | b'.'
                    | b'_'
                    | b'~'
                    | b'!'
                    | b'$'
                    | b'&'
                    | b'\''
                    | b'('
                    | b')'
                    | b'*'
                    | b'+'
                    | b','
                    | b';'
                    | b'='
                    | b':'
                    | b'@'
            );
        if keep {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Payloads that try to leave the segment they were given.
    const HOSTILE: &[&str] = &[
        "",
        ".",
        "..",
        "../key/info",
        "../../usage",
        "..%2F..%2Fusage",
        "a/b",
        "x?y=1",
        "x#y",
        "%2e%2e",
        "..\\x",
        "a b",
        "a\tb",
        "a\nb",
        "a\u{1b}[2Jb",
        "a\u{7f}b",
        "a\u{9b}b",
        "\u{2024}\u{2024}",
        "\u{ff0e}\u{ff0e}",
        "a\u{2215}b",
        "a\u{ff0f}b",
    ];

    #[test]
    fn hostile_addresses_are_refused() {
        for bad in HOSTILE {
            assert!(
                address(bad, "pool address").is_err(),
                "expected {bad:?} to be refused"
            );
        }
    }

    #[test]
    fn hostile_networks_are_refused() {
        for bad in HOSTILE
            .iter()
            .chain(&["eth:1", "eth.main", "eth+", "ethereum "])
        {
            assert!(network(bad).is_err(), "expected {bad:?} to be refused");
        }
    }

    #[test]
    fn real_addresses_go_out_unchanged() {
        for good in [
            "0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640",
            "So11111111111111111111111111111111111111112",
            "0x2::sui::SUI",
            "EQBynBO23ywHy_CgarY9NK9FTz0yDsG82PtcbSTQgGoXwiuA",
            "0.0.456858",
        ] {
            assert_eq!(address(good, "address").unwrap(), good);
        }
    }

    #[test]
    fn non_pchar_ascii_is_encoded_so_the_value_stays_one_segment() {
        assert_eq!(address("a<b>", "address").unwrap(), "a%3Cb%3E");
        assert_eq!(address("a|b", "address").unwrap(), "a%7Cb");
    }

    #[test]
    fn networks_are_lowercased_and_kept() {
        assert_eq!(network("ethereum").unwrap(), "ethereum");
        assert_eq!(network("Ethereum").unwrap(), "ethereum");
        assert_eq!(network("polygon_pos").unwrap(), "polygon_pos");
        assert_eq!(network("arbitrum-one").unwrap(), "arbitrum-one");
    }

    #[test]
    fn the_error_does_not_echo_control_bytes() {
        let err = address("a\u{1b}]0;x\u{7}", "pool address")
            .unwrap_err()
            .to_string();
        assert!(!err.contains('\u{1b}'));
        assert!(!err.contains('\u{7}'));
    }
}
