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

/// Resolve the API key: `--api-key` flag, then the environment, then the config
/// file, then keyless.
pub fn resolve_api_key(cli_key: Option<&str>) -> Option<String> {
    if let Some(key) = cli_key.and_then(sanitize_key) {
        return Some(key);
    }
    if let Some(key) = std::env::var(API_KEY_ENV_VAR)
        .ok()
        .as_deref()
        .and_then(sanitize_key)
    {
        return Some(key);
    }
    load_config()
        .ok()
        .and_then(|c| c.api_key)
        .as_deref()
        .and_then(sanitize_key)
}

/// Where the key in use came from. The first question worth asking when
/// somebody reports that their key is not working.
pub fn key_source(cli_key: Option<&str>) -> &'static str {
    if cli_key.and_then(sanitize_key).is_some() {
        return "CLI flag (--api-key)";
    }
    if std::env::var(API_KEY_ENV_VAR)
        .ok()
        .as_deref()
        .and_then(sanitize_key)
        .is_some()
    {
        return "Environment variable (DEXPAPRIKA_API_KEY)";
    }
    if load_config()
        .ok()
        .and_then(|c| c.api_key)
        .as_deref()
        .and_then(sanitize_key)
        .is_some()
    {
        return "Config file (~/.dexpaprika/config.json)";
    }
    "not set (running keyless)"
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

    // The Bearer rule lives in client.rs, but sanitisation is what stops a
    // pasted key from becoming a broken header in the first place.

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
            resolve_api_key(Some("api_from_flag")).as_deref(),
            Some("api_from_flag")
        );
    }

    #[test]
    fn a_blank_flag_falls_through_rather_than_forcing_keyless() {
        // An empty --api-key is a mistake, not an instruction to ignore the
        // environment, so resolution continues past it.
        assert_eq!(key_source(Some("   ")), key_source(None));
    }
}
