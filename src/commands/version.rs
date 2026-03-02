use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::output::OutputFormat;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const GITHUB_RELEASES_URL: &str =
    "https://api.github.com/repos/coinpaprika/dexpaprika-cli/releases/latest";

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
}

#[derive(Debug, Serialize)]
pub struct VersionCheck {
    pub current: String,
    pub latest: String,
    pub up_to_date: bool,
}

pub async fn execute(output: OutputFormat, raw: bool) -> Result<()> {
    let client = reqwest::Client::builder()
        .user_agent(format!(
            "dexpaprika-cli/{} ({}/{})",
            CURRENT_VERSION,
            std::env::consts::OS,
            std::env::consts::ARCH,
        ))
        .build()?;

    eprintln!("Checking for updates...");

    let result = client.get(GITHUB_RELEASES_URL).send().await;

    let check = match result {
        Ok(resp) if resp.status().is_success() => {
            let release: GitHubRelease = resp.json().await?;
            let latest = release.tag_name.trim_start_matches('v').to_string();
            let up_to_date = latest == CURRENT_VERSION;
            VersionCheck {
                current: CURRENT_VERSION.to_string(),
                latest,
                up_to_date,
            }
        }
        _ => {
            // No releases yet or network error — assume up to date
            VersionCheck {
                current: CURRENT_VERSION.to_string(),
                latest: CURRENT_VERSION.to_string(),
                up_to_date: true,
            }
        }
    };

    match output {
        OutputFormat::Table => {
            if check.up_to_date {
                println!("dexpaprika-cli v{} — up to date.", check.current);
            } else {
                println!(
                    "dexpaprika-cli v{} — update available: v{}\n\n  \
                     Update:  cargo install dexpaprika-cli\n  \
                     Release: https://github.com/coinpaprika/dexpaprika-cli/releases/latest",
                    check.current, check.latest
                );
            }
        }
        OutputFormat::Json => {
            crate::output::print_json_wrapped(
                &check,
                crate::output::ResponseMeta::dexpaprika("/version"),
                raw,
            )?;
        }
    }

    Ok(())
}
