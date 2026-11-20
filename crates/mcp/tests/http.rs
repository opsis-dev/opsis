use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};

const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"1"}}}"#;

struct Served {
    child: Child,
    address: String,
    token: String,
}

impl Drop for Served {
    fn drop(&mut self) {
        self.child.kill().ok();
        self.child.wait().ok();
    }
}

fn start() -> Served {
    let dir = tempfile::tempdir().unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_opsis-mcp"))
        .args(["http", "--port", "0"])
        .env("OPSIS_API_SOCKET", dir.path().join("api.sock"))
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
    let listening = lines.next().unwrap().unwrap();
    let token = lines.next().unwrap().unwrap();
    let address = listening
        .split("http://")
        .nth(1)
        .and_then(|rest| rest.strip_suffix("/mcp"))
        .unwrap()
        .to_owned();
    Served {
        child,
        address,
        token: token.strip_prefix("token: ").unwrap().to_owned(),
    }
}

fn post(served: &Served, headers: &[(&str, &str)]) -> String {
    let mut stream = TcpStream::connect(&served.address).unwrap();
    let mut request = format!(
        "POST /mcp HTTP/1.1\r\nContent-Type: application/json\r\nAccept: application/json, text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n",
        INITIALIZE.len()
    );
    for (name, value) in headers {
        request.extend([*name, ": ", *value, "\r\n"]);
    }
    request.push_str("\r\n");
    request.push_str(INITIALIZE);
    stream.write_all(request.as_bytes()).unwrap();
    let mut answer = String::new();
    stream.read_to_string(&mut answer).unwrap();
    answer
}

fn status(answer: &str) -> &str {
    answer.split(' ').nth(1).unwrap_or_default()
}

#[test]
fn http_answers_only_a_local_request_with_the_token() {
    let served = start();
    let bearer = format!("Bearer {}", served.token);
    let host = served.address.clone();

    let answer = post(&served, &[("Host", &host), ("Authorization", &bearer)]);
    assert_eq!(status(&answer), "200", "{answer}");
    assert!(answer.contains("\"opsis\""), "{answer}");

    let answer = post(&served, &[("Host", &host)]);
    assert_eq!(status(&answer), "401", "{answer}");
    let answer = post(
        &served,
        &[("Host", &host), ("Authorization", "Bearer wrong")],
    );
    assert_eq!(status(&answer), "401", "{answer}");

    let answer = post(
        &served,
        &[("Host", "evil.example"), ("Authorization", &bearer)],
    );
    assert_eq!(status(&answer), "403", "{answer}");
    let answer = post(
        &served,
        &[
            ("Host", &host),
            ("Authorization", &bearer),
            ("Origin", "https://evil.example"),
        ],
    );
    assert_eq!(status(&answer), "403", "{answer}");
}
