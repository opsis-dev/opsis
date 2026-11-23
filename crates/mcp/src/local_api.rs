use std::time::Duration;

use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::client::conn::http1::SendRequest;
use hyper::{Method, Request};
use hyper_util::rt::TokioIo;
use opsis_types::script::SESSION_HEADER;
use serde_json::Value;

const CONNECT_WITHIN: Duration = Duration::from_secs(5);

#[derive(Debug, thiserror::Error)]
pub enum LocalApiError {
    #[error("Opsis isn't running, or the assistant is off.")]
    Unreachable,
    #[error("Opsis answered something that isn't JSON: {0}")]
    NotJson(String),
}

#[derive(Debug, Clone)]
pub struct LocalApi {
    address: String,
    session: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    pub status: u16,
    pub body: Value,
}

impl Answer {
    pub const fn ok(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    pub fn error(&self) -> Option<&str> {
        self.body.get("error").and_then(Value::as_str)
    }
}

#[cfg(unix)]
async fn open_stream(address: &str) -> std::io::Result<tokio::net::UnixStream> {
    tokio::net::UnixStream::connect(address).await
}

#[cfg(windows)]
#[expect(
    clippy::unused_async,
    reason = "connect waits on it under a timeout like the Unix open_stream, which awaits the socket"
)]
async fn open_stream(
    address: &str,
) -> std::io::Result<tokio::net::windows::named_pipe::NamedPipeClient> {
    tokio::net::windows::named_pipe::ClientOptions::new().open(address)
}

impl LocalApi {
    pub const fn new(address: String, session: Option<String>) -> Self {
        Self { address, session }
    }

    async fn connect(&self) -> Result<SendRequest<Full<Bytes>>, LocalApiError> {
        let stream = tokio::time::timeout(CONNECT_WITHIN, open_stream(&self.address))
            .await
            .map_err(|_| LocalApiError::Unreachable)?
            .map_err(|_| LocalApiError::Unreachable)?;
        let (sender, connection) = hyper::client::conn::http1::handshake(TokioIo::new(stream))
            .await
            .map_err(|_| LocalApiError::Unreachable)?;
        tokio::spawn(async move {
            if let Err(error) = connection.await {
                log::debug!("the connection to Opsis ended: {error}");
            }
        });
        Ok(sender)
    }

    fn request(&self, method: Method, path: &str, body: Option<&Value>) -> Request<Full<Bytes>> {
        let mut request = Request::builder()
            .method(method)
            .uri(path)
            .header(hyper::header::HOST, "localhost");
        if let Some(session) = &self.session {
            request = request.header(SESSION_HEADER, session);
        }
        let bytes = body.map_or_else(Bytes::new, |b| Bytes::from(b.to_string()));
        if body.is_some() {
            request = request.header(hyper::header::CONTENT_TYPE, "application/json");
        }
        request
            .body(Full::new(bytes))
            .expect("a request from a fixed method, path and headers")
    }

    async fn send(
        &self,
        method: Method,
        path: &str,
        body: Option<&Value>,
    ) -> Result<hyper::Response<Incoming>, LocalApiError> {
        let mut sender = self.connect().await?;
        sender
            .send_request(self.request(method, path, body))
            .await
            .map_err(|_| LocalApiError::Unreachable)
    }

    pub async fn call(
        &self,
        method: Method,
        path: &str,
        body: Option<&Value>,
    ) -> Result<Answer, LocalApiError> {
        let response = self.send(method, path, body).await?;
        let status = response.status().as_u16();
        let bytes = response
            .into_body()
            .collect()
            .await
            .map_err(|_| LocalApiError::Unreachable)?
            .to_bytes();
        let text = String::from_utf8_lossy(&bytes);
        let body = if text.trim().is_empty() {
            Value::Null
        } else {
            serde_json::from_str(&text).map_err(|_| LocalApiError::NotJson(text.into_owned()))?
        };
        Ok(Answer { status, body })
    }

    pub async fn follow(&self, path: &str) -> Result<EventStream, LocalApiError> {
        let response = self.send(Method::GET, path, None).await?;
        Ok(EventStream {
            body: response.into_body(),
            pending: String::new(),
        })
    }
}

#[derive(Debug)]
pub struct EventStream {
    body: Incoming,
    pending: String,
}

fn take_block(pending: &mut String) -> Option<String> {
    let end = pending.find("\n\n")?;
    let block: String = pending.drain(..end + 2).collect();
    Some(
        block
            .lines()
            .filter_map(|line| line.strip_prefix("data:"))
            .map(str::trim_start)
            .collect(),
    )
}

impl EventStream {
    pub async fn next(&mut self) -> Option<Value> {
        loop {
            while let Some(data) = take_block(&mut self.pending) {
                if let Ok(event) = serde_json::from_str(&data) {
                    return Some(event);
                }
            }
            let frame = self.body.frame().await?.ok()?;
            if let Ok(data) = frame.into_data() {
                self.pending.push_str(&String::from_utf8_lossy(&data));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_server_sent_events_and_skips_comments() {
        let mut pending =
            ": following run events\n\ndata: {\"type\":\"run:start\"}\n\ndata: {\"ty".to_owned();
        assert_eq!(take_block(&mut pending).as_deref(), Some(""));
        assert_eq!(
            take_block(&mut pending).as_deref(),
            Some("{\"type\":\"run:start\"}")
        );
        assert_eq!(take_block(&mut pending), None);
        assert_eq!(pending, "data: {\"ty");
    }
}
