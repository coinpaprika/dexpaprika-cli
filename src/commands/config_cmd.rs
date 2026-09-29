//! `dexpaprika-cli config` — inspect and store the optional API key.
//!
//! Keyless is the default. Everything here exists so somebody who has a key can
//! stop pasting it, not because a key is needed to use the CLI.

use anyhow::{bail, Result};

use crate::client::ApiClient;
use crate::config;

/// Show which key is in use, where it came from, and what the API makes of it.
pub async fn show(cli_key: Option<&str>, base_url: String) -> Result<()> {
    let key = config::resolve_api_key(cli_key);
    println!("Source:      {}", config::key_source(cli_key));
    match &key {
        Some(k) => println!("Key:         {}", config::mask_key(k)),
        None => println!("Key:         none, running keyless"),
    }
    println!("Host:        {base_url}");
    println!("Config file: {}", config::config_path()?.display());
    println!();

    // /usage is the only endpoint that tells the truth about a key. On the data
    // endpoints an unreadable key is ignored rather than rejected: the call
    // returns 200 with real data while quietly serving the keyless tier, so a
    // broken key looks exactly like a working one.
    let client = ApiClient::with_api_key(key.clone(), base_url);
    match client
        .dexpaprika_get::<serde_json::Value>("/usage", &[])
        .await
    {
        Ok(usage) => {
            let plan = usage
                .get("plan")
                .and_then(|p| p.as_str())
                .unwrap_or("unknown");
            println!("API reports: plan \"{plan}\"");
            if let Some(hint) = paid_host_hint(plan, client.base_url()) {
                println!();
                println!("{hint}");
            }
            if key.is_some() && plan == "keyless" {
                println!();
                println!("A key is configured but the API still sees an anonymous caller,");
                println!("so it is not reaching us. Check the value is the key on its own.");
            }
        }
        Err(err) => {
            println!("API check failed: {err}");
            if key.is_some() {
                println!();
                println!("The key reached the API and was rejected. Check that the whole key");
                println!("was pasted, with nothing in front of it.");
            }
        }
    }
    Ok(())
}

/// Validate a key against /usage, then store it with 0600 permissions.
pub async fn set_key(key: &str, base_url: String) -> Result<()> {
    let Some(clean) = config::sanitize_key(key) else {
        bail!("That does not look like a usable key: it is empty or contains a newline.");
    };

    println!("Validating...");
    let client = ApiClient::with_api_key(Some(clean.clone()), base_url);
    match client
        .dexpaprika_get::<serde_json::Value>("/usage", &[])
        .await
    {
        Ok(usage) => {
            let plan = usage
                .get("plan")
                .and_then(|p| p.as_str())
                .unwrap_or("unknown");
            if plan == "keyless" {
                bail!(
                    "The API still reports the keyless plan, so this key never reached it. \
                     Nothing was saved."
                );
            }
            config::save_api_key(&clean)?;
            println!("Key validated. The API reports plan \"{plan}\".");
            if let Some(hint) = paid_host_hint(plan, client.base_url()) {
                println!("{hint}");
            }
        }
        Err(err) => {
            // Refuse rather than save. Saving a rejected key means every later
            // call fails for a reason the user has already been told once and
            // will not see again.
            bail!(
                "The API rejected this key, so nothing was saved.\n\
                 {err}\n\n\
                 Check that the whole key was pasted, with nothing in front of it."
            );
        }
    }

    println!("Saved to {}", config::config_path()?.display());
    println!("Key:      {}", config::mask_key(&clean));
    Ok(())
}

/// Forget the stored key. Leaves the environment variable alone.
pub fn delete() -> Result<()> {
    if !config::config_exists() {
        println!("Nothing stored. Already keyless.");
        return Ok(());
    }
    config::delete_config()?;
    println!("Stored key removed. The CLI is keyless again.");
    println!();
    println!(
        "If {} is set in your environment it still applies.",
        config::API_KEY_ENV_VAR
    );
    Ok(())
}

/// Paid plans have their own host. Say which one to use when a paid key is
/// pointed at the free host.
fn paid_host_hint(plan: &str, base_url: &str) -> Option<String> {
    let paid = matches!(plan, "dev" | "pro" | "enterprise");
    (paid && base_url != config::PAID_BASE_URL).then(|| {
        format!(
            "A {plan} key belongs on {}. Set {}={} or pass --base-url {}.",
            config::PAID_BASE_URL,
            config::BASE_URL_ENV_VAR,
            config::PAID_BASE_URL,
            config::PAID_BASE_URL,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::paid_host_hint;
    use crate::config::{DEFAULT_BASE_URL, PAID_BASE_URL};

    #[test]
    fn paid_key_on_the_free_host_is_told_where_to_go() {
        for plan in ["dev", "pro", "enterprise"] {
            let hint = paid_host_hint(plan, DEFAULT_BASE_URL).expect(plan);
            assert!(hint.contains(PAID_BASE_URL), "{hint}");
            assert!(hint.contains("DEXPAPRIKA_API_BASE_URL"), "{hint}");
        }
    }

    #[test]
    fn no_hint_on_the_paid_host_or_for_free_plans() {
        assert!(paid_host_hint("pro", PAID_BASE_URL).is_none());
        assert!(paid_host_hint("free", DEFAULT_BASE_URL).is_none());
        assert!(paid_host_hint("keyless", DEFAULT_BASE_URL).is_none());
    }
}
