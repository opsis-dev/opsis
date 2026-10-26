use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chromiumoxide::cdp::browser_protocol::page::{
    EventLoadEventFired, EventNavigatedWithinDocument,
};
use futures::StreamExt;
use indexmap::IndexMap;
use opsis_capture::browser::shell_in;
use opsis_capture::cdp::{CaptureError, Result, Tab};
use opsis_capture::executors::{Login, LoginRequest};
use opsis_compile::nodes::browser::DoneWhen;
use opsis_runner::context::{BoxFuture, ExecError};
use opsis_types::secrets::{SameSite, SavedCookie, SavedItem, SavedOrigin, StorageState};
use serde::Deserialize;
use serde_json::json;
use tokio_util::sync::CancellationToken;

use crate::host::{ViewEnv, ViewHost};
use crate::view::StreamView;

const TITLE: &str = "Log in — close this window when you're done";
const SELECTOR_POLL: Duration = Duration::from_millis(500);
const LOCAL_STORAGE: &str = "Object.entries(localStorage)";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Cookie {
    name: String,
    value: String,
    domain: String,
    path: String,
    expires: f64,
    http_only: bool,
    secure: bool,
    session: bool,
    #[serde(default)]
    same_site: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Cookies {
    cookies: Vec<Cookie>,
}

fn same_site(value: Option<&str>) -> SameSite {
    match value {
        Some("Strict") => SameSite::Strict,
        Some("None") => SameSite::None,
        _ => SameSite::Lax,
    }
}

fn saved(cookie: Cookie) -> SavedCookie {
    SavedCookie {
        expires: if cookie.session || cookie.expires <= 0.0 {
            -1.0
        } else {
            cookie.expires
        },
        same_site: same_site(cookie.same_site.as_deref()),
        name: cookie.name,
        value: cookie.value,
        domain: cookie.domain,
        path: cookie.path,
        http_only: cookie.http_only,
        secure: cookie.secure,
    }
}

async fn read_local_storage(view: &StreamView, origins: &mut IndexMap<String, Vec<SavedItem>>) {
    let Ok(url) = url::Url::parse(&view.url().await) else {
        return;
    };
    if !matches!(url.scheme(), "http" | "https") {
        return;
    }
    match view
        .active_tab()
        .await
        .evaluate_main::<Vec<(String, String)>>(LOCAL_STORAGE)
        .await
    {
        Ok(entries) => {
            origins.insert(
                url.origin().ascii_serialization(),
                entries
                    .into_iter()
                    .map(|(name, value)| SavedItem { name, value })
                    .collect(),
            );
        }
        Err(error) => log::debug!(
            "reading local storage of {}: {error}",
            url.origin().ascii_serialization()
        ),
    }
}

async fn selector_shown(tab: &Tab, selector: &str) -> bool {
    tab.evaluate_main::<bool>(&format!("!!document.querySelector({})", json!(selector)))
        .await
        .unwrap_or(false)
}

async fn storage_state(
    view: &StreamView,
    origins: IndexMap<String, Vec<SavedItem>>,
) -> Result<String> {
    let found: Cookies = view
        .active_tab()
        .await
        .send_as("Network.getAllCookies", json!({}))
        .await?;
    let state = StorageState {
        cookies: found.cookies.into_iter().map(saved).collect(),
        origins: origins
            .into_iter()
            .map(|(origin, local_storage)| SavedOrigin {
                origin,
                local_storage,
            })
            .collect(),
    };
    serde_json::to_string(&state).map_err(|e| CaptureError::Failed(e.to_string()))
}

async fn watch(
    view: &StreamView,
    request: &LoginRequest,
    closed: &CancellationToken,
    cancel: &CancellationToken,
) -> Result<Option<IndexMap<String, Vec<SavedItem>>>> {
    let tab = view.active_tab().await;
    let mut loads = tab.page.event_listener::<EventLoadEventFired>().await?;
    let mut within = tab
        .page
        .event_listener::<EventNavigatedWithinDocument>()
        .await?;
    let deadline = tokio::time::sleep(Duration::from_secs(u64::from(request.timeout_sec)));
    tokio::pin!(deadline);
    let mut poll = tokio::time::interval(SELECTOR_POLL);
    let mut origins = IndexMap::new();
    let mut landed: Option<String> = None;
    let mut started = false;
    loop {
        if started {
            tokio::select! {
                () = cancel.cancelled() => return Ok(None),
                () = closed.cancelled() => break,
                () = &mut deadline => break,
                _ = poll.tick(), if request.done_when == DoneWhen::Selector => {}
                Some(_) = loads.next() => {}
                Some(_) = within.next() => {}
            }
        }
        started = true;
        read_local_storage(view, &mut origins).await;
        let url = view.url().await;
        if url.is_empty() || url == "about:blank" {
            continue;
        }
        match &landed {
            None => landed = Some(url),
            Some(first) if request.done_when == DoneWhen::UrlChange && *first != url => break,
            Some(_) => {}
        }
        if request.done_when == DoneWhen::Selector
            && !request.selector.is_empty()
            && selector_shown(&tab, &request.selector).await
        {
            break;
        }
    }
    read_local_storage(view, &mut origins).await;
    Ok(Some(origins))
}

pub async fn sign_in(
    env: &ViewEnv,
    host: &dyn ViewHost,
    request: &LoginRequest,
    cancel: &CancellationToken,
) -> Result<Option<String>> {
    let view = env.open(&request.url, request.viewport).await?;
    let closed = CancellationToken::new();
    host.show(TITLE, Arc::clone(&view), closed.clone());
    let state = match watch(&view, request, &closed, cancel).await {
        Ok(Some(origins)) => storage_state(&view, origins).await.map(Some),
        other => other.map(|_| None),
    };
    host.hide(&view);
    view.close().await;
    state
}

pub struct StreamLogin {
    pub engine_dir: PathBuf,
    pub temp_dir: PathBuf,
    pub host: Arc<dyn ViewHost>,
}

impl Login for StreamLogin {
    fn login(
        &self,
        request: LoginRequest,
        cancel: CancellationToken,
    ) -> BoxFuture<std::result::Result<String, ExecError>> {
        let host = Arc::clone(&self.host);
        let env = shell_in(&self.engine_dir).map(|executable| ViewEnv {
            executable,
            temp_dir: self.temp_dir.clone(),
        });
        Box::pin(async move {
            let env = env.map_err(|e| ExecError::new(e.to_string()))?;
            match sign_in(&env, host.as_ref(), &request, &cancel).await {
                Ok(Some(state)) => Ok(state),
                Ok(None) => Err(ExecError::Cancelled),
                Err(error) => Err(ExecError::new(error.to_string())),
            }
        })
    }
}
