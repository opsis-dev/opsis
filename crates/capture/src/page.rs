use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chromiumoxide::cdp::browser_protocol::page::{
    EventFrameNavigated, EventLifecycleEvent, EventNavigatedWithinDocument,
};
use futures::StreamExt;
use opsis_compile::nodes::browser::WaitUntil;
use opsis_compile::text::format_duration;
use opsis_types::actions::{PageAddress, Rect};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tokio::time::Instant;

use crate::cdp::{Arg, CaptureError, Handle, Result, Tab};
use crate::clock::as_ms;
use crate::take_clock::TakeClock;
use crate::trace::CallInfo;
use crate::url_match::url_matches;

pub const NAVIGATION_TIMEOUT: Duration = Duration::from_secs(30);
const VISIBLE_POLL: Duration = Duration::from_millis(16);
const STEPPED_POLL: Duration = Duration::from_millis(1);
const RETRY_WAITS: [Duration; 5] = [
    Duration::ZERO,
    Duration::from_millis(20),
    Duration::from_millis(100),
    Duration::from_millis(100),
    Duration::from_millis(500),
];
const RETRY_LIMIT: Duration = Duration::from_secs(30);
const QUERY: &str = "(injected, selector) => injected.querySelector(injected.parseSelector(selector), document, false)";
const IS_VISIBLE: &str = "(injected, selector) => { const el = injected.querySelector(injected.parseSelector(selector), document, false); return !!el && injected.elementState(el, 'visible', true).matches; }";
const STABLE: &str = "(injected, node) => injected.checkElementStates(node, ['stable'], true)";

#[derive(Debug, Default)]
struct PageState {
    loader_id: String,
    fired: HashSet<String>,
    url: String,
    visited: Vec<(Duration, String)>,
}

impl PageState {
    fn moved_to(&mut self, url: String, at: Duration) {
        if url == self.url {
            return;
        }
        if !url.starts_with("about:") {
            self.visited.push((at, url.clone()));
        }
        self.url = url;
    }
}

#[derive(Debug)]
pub struct Lifecycle {
    clock: TakeClock,
    state: Arc<Mutex<PageState>>,
    changed: Arc<Notify>,
    tasks: Vec<JoinHandle<()>>,
}

impl Drop for Lifecycle {
    fn drop(&mut self) {
        for task in &self.tasks {
            task.abort();
        }
    }
}

const fn lifecycle_name(wait_until: WaitUntil) -> &'static str {
    match wait_until {
        WaitUntil::Load => "load",
        WaitUntil::Domcontentloaded => "DOMContentLoaded",
        WaitUntil::Networkidle => "networkIdle",
        WaitUntil::Commit => "commit",
    }
}

fn didnt_respond(timeout: Duration) -> CaptureError {
    CaptureError::failed(format!(
        "The page didn't respond within {}.",
        format_duration(opsis_types::numbers::millis(timeout))
    ))
}

fn network_error(error_text: &str) -> CaptureError {
    let code = error_text.trim_start_matches("net::");
    if code == "ERR_NAME_NOT_RESOLVED" {
        CaptureError::failed("Couldn't reach the site. Check the address.")
    } else {
        CaptureError::failed(format!("Couldn't load the page ({code})."))
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Navigated {
    #[serde(default)]
    loader_id: Option<String>,
    #[serde(default)]
    error_text: Option<String>,
}

impl Lifecycle {
    pub async fn start(tab: &Tab) -> Result<Self> {
        let frame_id = tab.main_frame_id().await?;
        let state: Arc<Mutex<PageState>> = Arc::default();
        let changed = Arc::new(Notify::new());
        let mut lifecycle = tab.page.event_listener::<EventLifecycleEvent>().await?;
        let mut navigated = tab.page.event_listener::<EventFrameNavigated>().await?;
        let mut within = tab
            .page
            .event_listener::<EventNavigatedWithinDocument>()
            .await?;
        let (s, c, f) = (Arc::clone(&state), Arc::clone(&changed), frame_id.clone());
        let on_lifecycle = tokio::spawn(async move {
            while let Some(event) = lifecycle.next().await {
                if event.frame_id.inner() != &f {
                    continue;
                }
                let mut page = s.lock().expect("page state");
                if event.name == "init" || page.loader_id != *event.loader_id.inner() {
                    page.loader_id.clone_from(event.loader_id.inner());
                    page.fired.clear();
                }
                page.fired.insert(event.name.clone());
                drop(page);
                c.notify_waiters();
            }
        });
        let (s, c, f) = (Arc::clone(&state), Arc::clone(&changed), frame_id.clone());
        let clock = tab.clock().clone();
        let on_navigated = tokio::spawn(async move {
            while let Some(event) = navigated.next().await {
                if event.frame.id.inner() != &f {
                    continue;
                }
                let mut page = s.lock().expect("page state");
                page.moved_to(
                    format!(
                        "{}{}",
                        event.frame.url,
                        event.frame.url_fragment.clone().unwrap_or_default()
                    ),
                    clock.now(),
                );
                if page.loader_id != *event.frame.loader_id.inner() {
                    page.loader_id.clone_from(event.frame.loader_id.inner());
                    page.fired.clear();
                }
                page.fired.insert("commit".to_owned());
                drop(page);
                c.notify_waiters();
            }
        });
        let (s, c) = (Arc::clone(&state), Arc::clone(&changed));
        let clock = tab.clock().clone();
        let on_within = tokio::spawn(async move {
            while let Some(event) = within.next().await {
                if event.frame_id.inner() != &frame_id {
                    continue;
                }
                s.lock()
                    .expect("page state")
                    .moved_to(event.url.clone(), clock.now());
                c.notify_waiters();
            }
        });
        "about:blank".clone_into(&mut state.lock().expect("page state").url);
        Ok(Self {
            clock: tab.clock().clone(),
            state,
            changed,
            tasks: vec![on_lifecycle, on_navigated, on_within],
        })
    }

    pub fn url(&self) -> String {
        self.state.lock().expect("page state").url.clone()
    }

    pub fn addresses_since(&self, t0: Duration) -> Vec<PageAddress> {
        let page = self.state.lock().expect("page state");
        let first = page
            .visited
            .iter()
            .rposition(|(at, _)| *at <= t0)
            .unwrap_or(0);
        page.visited[first..]
            .iter()
            .map(|(at, url)| PageAddress {
                t: as_ms(at.saturating_sub(t0)),
                url: url.clone(),
            })
            .collect()
    }

    async fn until(
        &self,
        timeout: Duration,
        mut done: impl FnMut(&PageState) -> bool,
    ) -> Result<()> {
        if self.clock.is_stepped() {
            return self
                .clock
                .until(timeout, STEPPED_POLL, || {
                    let finished = done(&self.state.lock().expect("page state"));
                    async move { finished.then_some(()) }
                })
                .await
                .ok_or_else(|| didnt_respond(timeout));
        }
        let deadline = Instant::now() + timeout;
        loop {
            let notified = self.changed.notified();
            if done(&self.state.lock().expect("page state")) {
                return Ok(());
            }
            if tokio::time::timeout_at(deadline, notified).await.is_err() {
                return Err(didnt_respond(timeout));
            }
        }
    }

    pub async fn navigate(
        &self,
        tab: &Tab,
        url: &str,
        wait_until: WaitUntil,
        timeout: Duration,
    ) -> Result<()> {
        tab.traced(
            || CallInfo::new("goto", "Navigate").target(url),
            self.navigated(tab, url, wait_until, timeout),
        )
        .await
    }

    async fn navigated(
        &self,
        tab: &Tab,
        url: &str,
        wait_until: WaitUntil,
        timeout: Duration,
    ) -> Result<()> {
        let frame_id = tab.main_frame_id().await?;
        let navigated: Navigated = tokio::time::timeout(
            timeout,
            tab.send_as("Page.navigate", json!({ "url": url, "frameId": frame_id })),
        )
        .await
        .map_err(|_| didnt_respond(timeout))??;
        if let Some(error) = navigated.error_text.filter(|e| !e.is_empty()) {
            return Err(network_error(&error));
        }
        let Some(loader_id) = navigated.loader_id else {
            return Ok(());
        };
        let name = lifecycle_name(wait_until);
        self.until(timeout, |page| {
            page.loader_id == loader_id && page.fired.contains(name)
        })
        .await
    }

    pub async fn reload(&self, tab: &Tab) -> Result<()> {
        tab.traced(|| CallInfo::new("reload", "Reload"), self.reloaded(tab))
            .await
    }

    async fn reloaded(&self, tab: &Tab) -> Result<()> {
        let before = self.state.lock().expect("page state").loader_id.clone();
        tab.send("Page.reload", json!({})).await?;
        self.until(NAVIGATION_TIMEOUT, |page| {
            page.loader_id != before && page.fired.contains("load")
        })
        .await
    }

    pub async fn wait_for_load_state(
        &self,
        tab: &Tab,
        wait_until: WaitUntil,
        timeout: Duration,
    ) -> Result<()> {
        let name = lifecycle_name(wait_until);
        tab.traced(
            || CallInfo::new("waitForLoadState", "Wait for load state").target(name),
            self.until(timeout, |page| page.fired.contains(name)),
        )
        .await
    }

    pub async fn wait_for_url(&self, tab: &Tab, glob: &str, timeout: Duration) -> Result<()> {
        tab.traced(
            || CallInfo::new("waitForURL", "Wait for address").target(glob),
            self.waited_for_url(glob, timeout),
        )
        .await
    }

    async fn waited_for_url(&self, glob: &str, timeout: Duration) -> Result<()> {
        url_matches("", glob)?;
        let start = self.clock.now();
        self.until(timeout, |page| {
            url_matches(&page.url, glob).unwrap_or(false)
        })
        .await?;
        let spent = self.clock.now().saturating_sub(start);
        let left = timeout.saturating_sub(spent);
        self.until(left, |page| page.fired.contains("load"))
            .await
            .map_err(|_| didnt_respond(timeout))
    }
}

#[derive(Debug, Deserialize)]
struct BoxModel {
    border: Vec<f64>,
}

#[derive(Debug, Deserialize)]
struct BoxModelResult {
    model: BoxModel,
}

impl Tab {
    pub async fn query(&self, selector: &str) -> Result<Option<Handle>> {
        self.injected_handle(QUERY, &[Arg::Value(json!(selector))])
            .await
    }

    pub async fn is_visible(&self, selector: &str) -> Result<bool> {
        self.traced(
            || CallInfo::new("isVisible", "Check visible").target(selector),
            self.injected_value(IS_VISIBLE, &[Arg::Value(json!(selector))]),
        )
        .await
    }

    pub async fn wait_visible(&self, selector: &str, timeout: Duration) -> Result<Handle> {
        self.traced(
            || CallInfo::new("waitForSelector", "Wait for").target(selector),
            self.waited_visible(selector, timeout),
        )
        .await
    }

    async fn waited_visible(&self, selector: &str, timeout: Duration) -> Result<Handle> {
        let found = self
            .clock()
            .until(timeout, VISIBLE_POLL, || async {
                if !self
                    .injected_value::<bool>(IS_VISIBLE, &[Arg::Value(json!(selector))])
                    .await
                    .unwrap_or(false)
                {
                    return None;
                }
                self.query(selector).await.transpose()
            })
            .await;
        found.unwrap_or(Err(CaptureError::NotFoundWithin(timeout)))
    }

    pub async fn bounding_box(&self, handle: &Handle) -> Result<Option<Rect>> {
        let Ok(found) = self
            .send_as::<BoxModelResult>("DOM.getBoxModel", json!({ "objectId": handle.0 }))
            .await
        else {
            return Ok(None);
        };
        let quad = found.model.border;
        let xs = [quad[0], quad[2], quad[4], quad[6]];
        let ys = [quad[1], quad[3], quad[5], quad[7]];
        let x = xs.iter().copied().fold(f64::INFINITY, f64::min);
        let y = ys.iter().copied().fold(f64::INFINITY, f64::min);
        let w = xs.iter().copied().fold(f64::NEG_INFINITY, f64::max) - x;
        let h = ys.iter().copied().fold(f64::NEG_INFINITY, f64::max) - y;
        Ok(Some(Rect { x, y, w, h }))
    }

    pub async fn scroll_into_view_if_needed(&self, handle: &Handle) -> Result<()> {
        self.traced(
            || CallInfo::new("scrollIntoViewIfNeeded", "Scroll into view"),
            self.scrolled_into_view(handle),
        )
        .await
    }

    async fn scrolled_into_view(&self, handle: &Handle) -> Result<()> {
        let deadline = self.clock().now() + RETRY_LIMIT;
        for retry in 0.. {
            if retry > 0 {
                self.clock()
                    .wait(RETRY_WAITS[(retry - 1).min(RETRY_WAITS.len() - 1)])
                    .await;
            }
            let missing: Value = self.injected_value(STABLE, &[Arg::Handle(handle)]).await?;
            if missing == json!("error:notconnected") {
                return Err(CaptureError::failed(
                    "The element left the page before it could be scrolled to.",
                ));
            }
            if missing.is_null() {
                self.send(
                    "DOM.scrollIntoViewIfNeeded",
                    json!({ "objectId": handle.0 }),
                )
                .await?;
                return Ok(());
            }
            if self.clock().now() >= deadline {
                break;
            }
        }
        Err(CaptureError::failed(
            "The element kept moving, so it couldn't be scrolled into view.",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explains_page_timeouts_in_words() {
        assert_eq!(
            didnt_respond(Duration::from_secs(30)).to_string(),
            "The page didn't respond within 30 s."
        );
        assert_eq!(
            didnt_respond(Duration::from_secs(90)).to_string(),
            "The page didn't respond within 1 min 30 s."
        );
    }

    #[test]
    fn explains_network_failures() {
        assert_eq!(
            network_error("net::ERR_NAME_NOT_RESOLVED").to_string(),
            "Couldn't reach the site. Check the address."
        );
        assert_eq!(
            network_error("net::ERR_CONNECTION_REFUSED").to_string(),
            "Couldn't load the page (ERR_CONNECTION_REFUSED)."
        );
    }
}
