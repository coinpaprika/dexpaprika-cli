use clap::Parser;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

/// What the shell was started with. The key and the host belong to the
/// session, not to a line: they are fixed when the shell starts
/// (`dexpaprika-cli --base-url ... shell`) and every line runs with them.
pub struct Session {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

/// Parse one shell line into the command it asks for, with the session's key
/// and host applied.
///
/// A line is split into words and handed to the same parser as the command
/// line, so a line built from untrusted text could otherwise carry
/// `--base-url` and send the configured key to another host, or `--api-key`
/// and swap the key. Both are refused here, whichever way they were spelled,
/// because the check looks at what the parser produced rather than at the
/// words.
fn prepare_line(line: &str, session: &Session) -> Result<crate::Cli, String> {
    let mut full_args = vec!["dexpaprika-cli".to_string()];
    full_args.extend(split_args(line));
    let mut cli = crate::Cli::try_parse_from(&full_args).map_err(|e| e.to_string())?;
    if cli.api_key.is_some() || cli.base_url.is_some() {
        return Err(
            "--api-key and --base-url are set when the shell starts, for example \
             `dexpaprika-cli --base-url https://api-pro.dexpaprika.com shell`, not per line. \
             Line ignored.\n"
                .to_string(),
        );
    }
    if matches!(cli.command, crate::Commands::Shell) {
        return Err("Already in the shell.\n".to_string());
    }
    cli.api_key = session.api_key.clone();
    cli.base_url = session.base_url.clone();
    Ok(cli)
}

pub async fn run_shell(session: Session) {
    println!("dexpaprika-cli interactive shell: type commands without the 'dexpaprika-cli' prefix");
    println!("Type 'exit' or 'quit' to leave. Ctrl+D also exits.\n");

    let mut rl = match DefaultEditor::new() {
        Ok(rl) => rl,
        Err(e) => {
            eprintln!("Failed to initialize shell: {e}");
            return;
        }
    };

    loop {
        match rl.readline("dexpaprika> ") {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if line == "exit" || line == "quit" {
                    break;
                }

                let _ = rl.add_history_entry(line);

                match prepare_line(line, &session) {
                    Ok(cli) => {
                        if let Err(e) = crate::run(cli).await {
                            eprintln!(
                                "Error: {}",
                                crate::output::sanitize_for_terminal(&e.to_string())
                            );
                        }
                    }
                    Err(e) => {
                        eprint!("{}", crate::output::sanitize_for_terminal(&e.to_string()));
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("(Ctrl+C received. Type 'exit' to quit.)");
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(e) => {
                eprintln!("Shell error: {e}");
                break;
            }
        }
    }
}

fn split_args(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = ' ';

    for ch in input.chars() {
        if in_quotes {
            if ch == quote_char {
                in_quotes = false;
            } else {
                current.push(ch);
            }
        } else if ch == '"' || ch == '\'' {
            in_quotes = true;
            quote_char = ch;
        } else if ch.is_whitespace() {
            if !current.is_empty() {
                args.push(current.clone());
                current.clear();
            }
        } else {
            current.push(ch);
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    args
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session() -> Session {
        Session {
            api_key: Some("api_session".into()),
            base_url: Some("https://api-pro.dexpaprika.com".into()),
        }
    }

    #[test]
    fn a_line_cannot_set_the_host_or_the_key() {
        for line in [
            "pool ethereum 0xabc --base-url=http://127.0.0.1:9",
            "pool ethereum 0xabc --base-url http://127.0.0.1:9",
            "--base-url http://127.0.0.1:9 pool ethereum 0xabc",
            "pool ethereum '0xabc' \"--base-url=http://127.0.0.1:9\"",
            "pool ethereum 0xabc --api-key api_other",
            "pool ethereum 0xabc --api-key=api_other",
        ] {
            assert!(
                prepare_line(line, &session()).is_err(),
                "expected {line:?} to be refused"
            );
        }
    }

    #[test]
    fn a_line_runs_with_the_session_key_and_host() {
        let cli = match prepare_line("pool ethereum 0xabc -o json", &session()) {
            Ok(cli) => cli,
            Err(e) => panic!("{e}"),
        };
        assert_eq!(cli.api_key.as_deref(), Some("api_session"));
        assert_eq!(
            cli.base_url.as_deref(),
            Some("https://api-pro.dexpaprika.com")
        );
    }

    #[test]
    fn the_shell_does_not_nest() {
        assert!(prepare_line("shell", &session()).is_err());
    }
}
