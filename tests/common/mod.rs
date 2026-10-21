//! A local stand-in for the REST host, so the binary can be driven end to end
//! without touching the network. It listens on 127.0.0.1, records every request
//! it sees, and answers from a closure.

#![allow(dead_code)]

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone, Debug)]
pub struct Seen {
    /// The request target exactly as it arrived, path and query.
    pub target: String,
    pub headers: Vec<(String, String)>,
}

impl Seen {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
}

pub struct Reply {
    pub status: u16,
    pub body: Vec<u8>,
    pub headers: Vec<(String, Vec<u8>)>,
}

impl Reply {
    pub fn json(status: u16, body: &str) -> Self {
        Self {
            status,
            body: body.as_bytes().to_vec(),
            headers: Vec::new(),
        }
    }
}

pub struct Stub {
    pub base: String,
    seen: Arc<Mutex<Vec<Seen>>>,
}

impl Stub {
    pub fn start<F>(respond: F) -> Self
    where
        F: Fn(&Seen) -> Reply + Send + Sync + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind a local port");
        let base = format!("http://{}", listener.local_addr().unwrap());
        let seen = Arc::new(Mutex::new(Vec::new()));
        let log = Arc::clone(&seen);
        thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap_or(0) == 0 {
                    continue;
                }
                let target = line.split_whitespace().nth(1).unwrap_or("").to_string();
                let mut headers = Vec::new();
                loop {
                    let mut h = String::new();
                    if reader.read_line(&mut h).unwrap_or(0) == 0 || h == "\r\n" {
                        break;
                    }
                    if let Some((k, v)) = h.trim_end().split_once(':') {
                        headers.push((k.trim().to_string(), v.trim().to_string()));
                    }
                }
                let req = Seen { target, headers };
                let reply = respond(&req);
                log.lock().unwrap().push(req);
                let mut out = format!(
                    "HTTP/1.1 {} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n",
                    reply.status,
                    reply.body.len()
                )
                .into_bytes();
                for (k, v) in &reply.headers {
                    out.extend_from_slice(k.as_bytes());
                    out.extend_from_slice(b": ");
                    out.extend_from_slice(v);
                    out.extend_from_slice(b"\r\n");
                }
                out.extend_from_slice(b"\r\n");
                out.extend_from_slice(&reply.body);
                let _ = stream.write_all(&out);
            }
        });
        Self { base, seen }
    }

    pub fn seen(&self) -> Vec<Seen> {
        self.seen.lock().unwrap().clone()
    }
}

/// A HOME of its own, so no test reads or writes the real ~/.dexpaprika.
pub fn scratch_home(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "dexpaprika-cli-test-{}-{}",
        name,
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The CLI binary with a clean environment: no key, no base URL, a scratch HOME.
pub fn cli(home: &PathBuf) -> assert_cmd::Command {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("dexpaprika-cli");
    cmd.env_clear()
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env("PATH", std::env::var("PATH").unwrap_or_default());
    cmd
}

pub fn has_terminal_control(bytes: &[u8]) -> bool {
    String::from_utf8_lossy(bytes).chars().any(|c| {
        (c.is_control() && c != '\n' && c != '\t')
            || matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
    })
}
