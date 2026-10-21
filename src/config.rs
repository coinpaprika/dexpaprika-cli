//! Optional API key: where it comes from, and where it is stored.
//!
//! Keyless is and stays the default. Everything here is inert until somebody
//! sets a key, and a missing config file is a normal state rather than an error.
//!
//! Shape ported from `coinpaprika-cli` so the two behave identically, with one
//! addition: keys are sanitised, because a key carrying a newline would corrupt
//! the outbound header rather than fail cleanly.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Environment variable consulted when no `--api-key` is passed.
pub const API_KEY_ENV_VAR: &str = "DEXPAPRIKA_API_KEY";

/// Environment variable naming a file that holds the key, for setups that keep
/// secrets in files (a secrets mount, a 0600 file) rather than in variables.
pub const API_KEY_FILE_ENV_VAR: &str = "DEXPAPRIKA_API_KEY_FILE";

/// Printed whenever the key arrives as a command-line argument.
pub const ARGV_KEY_WARNING: &str =
    "Warning: a key passed as a command-line argument is kept in your \
     shell history and is visible to other users on this machine in the process list. Run \
     `dexpaprika-cli config set-key` with no argument to be prompted for it, or use \
     DEXPAPRIKA_API_KEY or --api-key-file instead.";

/// Environment variable consulted when no `--base-url` is passed.
pub const BASE_URL_ENV_VAR: &str = "DEXPAPRIKA_API_BASE_URL";

/// The REST host for keyless callers and free keys.
pub const DEFAULT_BASE_URL: &str = "https://api.dexpaprika.com";

/// The REST host for Dev, Pro and Enterprise keys.
pub const PAID_BASE_URL: &str = "https://api-pro.dexpaprika.com";

/// The REST host: `--base-url`, then `DEXPAPRIKA_API_BASE_URL`, then the free
/// host. Never inferred from the key: a paid key is pointed at
/// [`PAID_BASE_URL`] by whoever configures it.
pub fn resolve_base_url(cli_base: Option<&str>) -> String {
    let from_env = std::env::var(BASE_URL_ENV_VAR).ok();
    pick_base_url(cli_base, from_env.as_deref())
}

fn pick_base_url(cli_base: Option<&str>, env_base: Option<&str>) -> String {
    [cli_base, env_base]
        .into_iter()
        .flatten()
        .map(|b| b.trim().trim_end_matches('/'))
        .find(|b| !b.is_empty())
        .unwrap_or(DEFAULT_BASE_URL)
        .to_string()
}

/// The host a key would be sent to, when that host is not DexPaprika's.
///
/// `--base-url` and DEXPAPRIKA_API_BASE_URL exist for the paid host, and the
/// key goes wherever they point. When that is somewhere else (a word-split
/// `--base-url=...` smuggled into a command string, a stale variable), the
/// caller should hear about it before the key leaves.
pub fn foreign_key_host(base_url: &str) -> Option<String> {
    let host = reqwest::Url::parse(base_url)
        .ok()
        .and_then(|u| u.host_str().map(str::to_ascii_lowercase))
        .unwrap_or_else(|| base_url.to_string());
    let ours = host == "dexpaprika.com" || host.ends_with(".dexpaprika.com");
    (!ours).then_some(host)
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub api_key: Option<String>,
}

pub fn config_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".dexpaprika"))
}

pub fn config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.json"))
}

pub fn config_exists() -> bool {
    config_path().map(|p| p.exists()).unwrap_or(false)
}

/// Read the stored config. A missing file is not an error: it means keyless.
pub fn load_config() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }
    let contents = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config file at {}", path.display()))?;
    serde_json::from_str(&contents).context("Failed to parse config file")
}

pub fn save_api_key(key: &str) -> Result<()> {
    let dir = config_dir()?;
    fs::create_dir_all(&dir)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700))?;
    }

    let config = Config {
        api_key: Some(key.to_string()),
    };
    let json = serde_json::to_string_pretty(&config)?;
    let path = config_path()?;

    // 0600 from creation rather than chmod afterwards: a world-readable window,
    // however brief, is still a window on a shared machine.
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(0o600)
            .open(&path)?;
        file.write_all(json.as_bytes())?;
    }

    #[cfg(not(unix))]
    fs::write(&path, &json)?;

    Ok(())
}

pub fn delete_config() -> Result<()> {
    let dir = config_dir()?;
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    Ok(())
}

/// Trim a key and reject anything that could break out of an HTTP header.
///
/// A key carrying CR, LF or NUL is dropped rather than mangled: a mangled key
/// authenticates as nobody, and because the data endpoints ignore an unreadable
/// key instead of rejecting it, the caller would never find out.
pub fn sanitize_key(key: &str) -> Option<String> {
    let trimmed = key.trim();
    if trimmed.is_empty() || trimmed.contains(['\r', '\n', '\0']) {
        return None;
    }
    Some(trimmed.to_string())
}

/// Read a key from a file: the first non-blank line, trimmed.
pub fn read_key_file(path: &str) -> Result<String> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("Could not read the API key file {path:?}"))?;
    let line = raw.lines().map(str::trim).find(|l| !l.is_empty());
    line.and_then(sanitize_key)
        .with_context(|| format!("The API key file {path:?} does not contain a key."))
}

/// Resolve the API key and say where it came from: `--api-key`, then
/// `--api-key-file`, then DEXPAPRIKA_API_KEY, then DEXPAPRIKA_API_KEY_FILE, then
/// the config file, then keyless. A key file that was asked for but cannot be
/// read is an error rather than a silent fall back to keyless.
pub fn resolve_key(
    cli_key: Option<&str>,
    cli_key_file: Option<&str>,
) -> Result<Option<(String, &'static str)>> {
    if let Some(key) = cli_key.and_then(sanitize_key) {
        return Ok(Some((key, "CLI flag (--api-key)")));
    }
    if let Some(path) = cli_key_file.filter(|p| !p.trim().is_empty()) {
        return Ok(Some((read_key_file(path)?, "Key file (--api-key-file)")));
    }
    if let Some(key) = std::env::var(API_KEY_ENV_VAR)
        .ok()
        .as_deref()
        .and_then(sanitize_key)
    {
        return Ok(Some((key, "Environment variable (DEXPAPRIKA_API_KEY)")));
    }
    if let Some(path) = std::env::var(API_KEY_FILE_ENV_VAR)
        .ok()
        .filter(|p| !p.trim().is_empty())
    {
        return Ok(Some((
            read_key_file(&path)?,
            "Key file (DEXPAPRIKA_API_KEY_FILE)",
        )));
    }
    Ok(load_config()
        .ok()
        .and_then(|c| c.api_key)
        .as_deref()
        .and_then(sanitize_key)
        .map(|key| (key, "Config file (~/.dexpaprika/config.json)")))
}

/// The key in use, from whichever source [`resolve_key`] finds first.
pub fn resolve_api_key(
    cli_key: Option<&str>,
    cli_key_file: Option<&str>,
) -> Result<Option<String>> {
    Ok(resolve_key(cli_key, cli_key_file)?.map(|(key, _)| key))
}

/// Where the key in use came from. The first question worth asking when
/// somebody reports that their key is not working.
pub fn key_source(cli_key: Option<&str>, cli_key_file: Option<&str>) -> Result<&'static str> {
    Ok(resolve_key(cli_key, cli_key_file)?
        .map(|(_, source)| source)
        .unwrap_or("not set (running keyless)"))
}

/// Show enough of a key to recognise it, never enough to use it.
pub fn mask_key(key: &str) -> String {
    if key.len() <= 8 {
        return "****".to_string();
    }
    format!("{}...{}", &key[..4], &key[key.len() - 4..])
}

#[cfg(test)]
mod tests {
    use super::*;

    // The send-the-key-alone rule lives in client.rs, but sanitisation is what
    // stops a pasted key from becoming a broken header in the first place.

    #[test]
    fn a_normal_key_survives_untouched() {
        assert_eq!(sanitize_key("api_abc123").as_deref(), Some("api_abc123"));
    }

    #[test]
    fn surrounding_whitespace_is_trimmed() {
        assert_eq!(
            sanitize_key("  api_abc123\n").as_deref(),
            Some("api_abc123")
        );
        assert_eq!(sanitize_key("\tapi_abc123 ").as_deref(), Some("api_abc123"));
    }

    #[test]
    fn an_empty_or_blank_key_is_keyless() {
        for blank in ["", "   ", "\t", "\n"] {
            assert_eq!(
                sanitize_key(blank),
                None,
                "expected {blank:?} to be rejected"
            );
        }
    }

    #[test]
    fn a_key_carrying_control_characters_is_dropped_not_mangled() {
        // Dropped rather than stripped: a mangled key authenticates as nobody,
        // and the data endpoints ignore an unreadable key instead of rejecting
        // it, so the caller would never find out.
        for hostile in ["api_a\r\nX-Evil: 1", "api_a\nb", "api_a\0b"] {
            assert_eq!(
                sanitize_key(hostile),
                None,
                "expected {hostile:?} to be rejected"
            );
        }
    }

    #[test]
    fn a_key_file_gives_its_first_non_blank_line() {
        let path = std::env::temp_dir().join(format!("dp-key-{}", std::process::id()));
        fs::write(&path, "\n  api_from_file  \n# second line\n").unwrap();
        let p = path.to_str().unwrap();
        assert_eq!(read_key_file(p).unwrap(), "api_from_file");
        assert_eq!(
            resolve_key(None, Some(p)).unwrap(),
            Some(("api_from_file".to_string(), "Key file (--api-key-file)"))
        );
        fs::write(&path, "\n \n").unwrap();
        assert!(read_key_file(p).is_err());
        let _ = fs::remove_file(&path);
        assert!(
            resolve_key(None, Some(p)).is_err(),
            "a missing file is an error"
        );
    }

    #[test]
    fn masking_shows_enough_to_recognise_never_enough_to_use() {
        let masked = mask_key("api_abcdefghijklmnop");
        assert_eq!(masked, "api_...mnop");
        assert!(!masked.contains("efghijkl"));
    }

    #[test]
    fn a_short_key_is_masked_entirely() {
        assert_eq!(mask_key("short"), "****");
    }

    #[test]
    fn an_explicit_flag_beats_everything_else() {
        assert_eq!(
            resolve_api_key(Some("api_from_flag"), None)
                .unwrap()
                .as_deref(),
            Some("api_from_flag")
        );
    }

    #[test]
    fn a_blank_flag_falls_through_rather_than_forcing_keyless() {
        // An empty --api-key is a mistake, not an instruction to ignore the
        // environment, so resolution continues past it.
        assert_eq!(
            key_source(Some("   "), None).unwrap(),
            key_source(None, None).unwrap()
        );
    }
}

#[cfg(test)]
mod base_url_tests {
    use super::{pick_base_url, DEFAULT_BASE_URL, PAID_BASE_URL};

    #[test]
    fn free_host_unless_told_otherwise() {
        assert_eq!(pick_base_url(None, None), DEFAULT_BASE_URL);
        assert_eq!(pick_base_url(Some("  "), Some("")), DEFAULT_BASE_URL);
    }

    #[test]
    fn only_dexpaprika_hosts_count_as_ours() {
        use super::foreign_key_host;
        assert_eq!(foreign_key_host(DEFAULT_BASE_URL), None);
        assert_eq!(foreign_key_host(PAID_BASE_URL), None);
        assert_eq!(
            foreign_key_host("https://dexpaprika.com.evil.test").as_deref(),
            Some("dexpaprika.com.evil.test")
        );
        assert_eq!(
            foreign_key_host("http://127.0.0.1:9").as_deref(),
            Some("127.0.0.1")
        );
    }

    #[test]
    fn flag_beats_env_and_trailing_slash_is_dropped() {
        assert_eq!(
            pick_base_url(None, Some("https://api-pro.dexpaprika.com/")),
            PAID_BASE_URL
        );
        assert_eq!(
            pick_base_url(
                Some("https://api-pro.dexpaprika.com"),
                Some("https://example.test")
            ),
            PAID_BASE_URL
        );
    }
}
