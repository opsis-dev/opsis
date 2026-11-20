use std::sync::Arc;

use axum::Router;
use axum::extract::{Request, State};
use axum::http::{StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};

use crate::server::OpsisTools;

pub const PATH: &str = "/mcp";
pub const DEFAULT_HOST: &str = "127.0.0.1";
pub const LOOPBACK_HOSTS: [&str; 3] = ["localhost", "127.0.0.1", "::1"];
const TOKEN_BYTES: usize = 32;
const WRONG_TOKEN: &str =
    "This needs the token opsis-mcp printed when it started, as Authorization: Bearer <token>.";

#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error("no random token could be made: {0}")]
    Token(getrandom::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub fn new_token() -> Result<String, HttpError> {
    let mut bytes = [0_u8; TOKEN_BYTES];
    getrandom::fill(&mut bytes).map_err(HttpError::Token)?;
    Ok(hex::encode(bytes))
}

pub fn allowed_hosts(host: &str) -> Vec<String> {
    let mut hosts: Vec<String> = LOOPBACK_HOSTS.iter().map(|h| (*h).to_owned()).collect();
    if !LOOPBACK_HOSTS.contains(&host) {
        hosts.push(host.to_owned());
    }
    hosts
}

fn same_text(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0_u8, |seen, (x, y)| seen | (x ^ y)) == 0
}

pub fn authorized(header: Option<&str>, token: &str) -> bool {
    header
        .and_then(|value| value.strip_prefix("Bearer "))
        .is_some_and(|given| same_text(given.as_bytes(), token.as_bytes()))
}

async fn check_token(State(token): State<Arc<str>>, request: Request, next: Next) -> Response {
    let given = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());
    if authorized(given, &token) {
        return next.run(request).await;
    }
    (
        StatusCode::UNAUTHORIZED,
        [(header::WWW_AUTHENTICATE, "Bearer")],
        WRONG_TOKEN,
    )
        .into_response()
}

pub fn router(tools: OpsisTools, host: &str, token: &str) -> Router {
    let config = StreamableHttpServerConfig::default()
        .with_allowed_hosts(allowed_hosts(host))
        .enforce_origin_validation()
        .with_legacy_session_mode(false);
    let service: StreamableHttpService<OpsisTools, LocalSessionManager> =
        StreamableHttpService::new(move || Ok(tools.clone()), Arc::default(), config);
    Router::new()
        .nest_service(PATH, service)
        .layer(middleware::from_fn_with_state(
            Arc::<str>::from(token),
            check_token,
        ))
}

pub async fn serve(tools: OpsisTools, host: &str, port: u16) -> Result<(), HttpError> {
    let listener = tokio::net::TcpListener::bind((host, port)).await?;
    let address = listener.local_addr()?;
    let token = new_token()?;
    if !LOOPBACK_HOSTS.contains(&host) {
        log::warn!(
            "listening on {host}: anyone on the network who has the token can change the open project"
        );
    }
    println!("opsis-mcp is listening on http://{address}{PATH}");
    println!("token: {token}");
    axum::serve(listener, router(tools, host, &token))
        .with_graceful_shutdown(async {
            if tokio::signal::ctrl_c().await.is_err() {
                std::future::pending::<()>().await;
            }
        })
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn only_the_exact_token_is_let_in(token in "[0-9a-f]{64}", other in "\\PC*") {
            prop_assert!(authorized(Some(&["Bearer ", token.as_str()].concat()), &token));
            prop_assert!(!authorized(None, &token));
            prop_assert!(!authorized(Some(&token), &token));
            prop_assume!(other != token);
            prop_assert!(!authorized(Some(&["Bearer ", other.as_str()].concat()), &token));
        }

        #[test]
        fn loopback_is_always_allowed_and_the_host_is_added_once(host in "[a-z0-9.:]{1,20}") {
            let hosts = allowed_hosts(&host);
            for loopback in LOOPBACK_HOSTS {
                prop_assert!(hosts.iter().any(|h| h == loopback));
            }
            prop_assert_eq!(hosts.iter().filter(|h| **h == host).count(), 1);
        }
    }

    #[test]
    fn a_token_is_new_each_time() {
        let first = new_token().unwrap();
        assert_eq!(first.len(), TOKEN_BYTES * 2);
        assert_ne!(first, new_token().unwrap());
    }
}
