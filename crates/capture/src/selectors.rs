use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

use opsis_compile::nodes::browser::parse_point;
use opsis_types::actions::Point;
use serde::Deserialize;
use serde_json::json;
use tokio_util::sync::CancellationToken;

use crate::cdp::{CaptureError, Result, Tab};
use crate::locators::{auto_candidates, first};
use crate::page::NAVIGATION_TIMEOUT;
use crate::page_scripts::{LIST_ON_SCREEN, call, interactive_selector};

pub const CSS_PREFIX: &str = "css=";
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(15);
const POLL: Duration = Duration::from_millis(250);
const DUMP_LIMIT: usize = 24;

#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    Point(Point),
    Element(String),
}

impl Target {
    pub fn element(&self) -> Result<&str> {
        match self {
            Self::Element(selector) => Ok(selector),
            Self::Point(_) => Err(CaptureError::failed(
                "A point on the page isn't an element. Pick an element instead.",
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Resolved {
    pub target: Target,
    pub by: String,
}

#[derive(Debug, thiserror::Error)]
#[error("Couldn't find “{label}” on the page.\n  Looked for: {}\n{dump}", tried.join(" | "))]
pub struct SelectorError {
    pub label: String,
    pub tried: Vec<String>,
    pub dump: String,
}

pub struct ResolveOptions<'a> {
    pub log: &'a (dyn Fn(&str) + Send + Sync),
    pub cancel: &'a CancellationToken,
    pub hoisted: &'a HashMap<String, String>,
    pub timeout: Option<Duration>,
}

impl fmt::Debug for ResolveOptions<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResolveOptions")
            .field("cancel", &self.cancel)
            .field("hoisted", &self.hoisted)
            .field("timeout", &self.timeout)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub enum ResolveError {
    NotFound(SelectorError),
    Other(CaptureError),
}

impl From<CaptureError> for ResolveError {
    fn from(error: CaptureError) -> Self {
        Self::Other(error)
    }
}

async fn visible_now(tab: &Tab, selector: &str) -> Result<bool> {
    match tab.is_visible(selector).await {
        Err(error) if error.page_changing() => Ok(false),
        done => done,
    }
}

pub async fn resolve_target(
    tab: &Tab,
    raw: &str,
    opts: &ResolveOptions<'_>,
) -> std::result::Result<Resolved, ResolveError> {
    let timeout = opts.timeout.unwrap_or(DEFAULT_TIMEOUT);
    if let Some(hit) = opts.hoisted.get(raw) {
        return Ok(Resolved {
            target: Target::Element(hit.clone()),
            by: "hoisted".to_owned(),
        });
    }
    if let Some(css) = raw.strip_prefix(CSS_PREFIX) {
        let selector = first(css);
        let handle = tab.wait_visible(&selector, timeout).await?;
        tab.release(&handle).await;
        return Ok(Resolved {
            target: Target::Element(selector),
            by: raw.to_owned(),
        });
    }
    if let Some(point) = parse_point(raw) {
        return Ok(Resolved {
            target: Target::Point(point),
            by: "point".to_owned(),
        });
    }
    let candidates = auto_candidates(raw);
    let clock = tab.clock();
    let start = clock.now();
    let timeout_ms = timeout.as_secs_f64() * 1000.0;
    while clock.now() - start < timeout_ms {
        if opts.cancel.is_cancelled() {
            return Err(CaptureError::Cancelled.into());
        }
        for candidate in &candidates {
            let selector = first(&candidate.selector);
            if visible_now(tab, &selector).await? {
                (opts.log)(&format!("✓ Found {raw}"));
                return Ok(Resolved {
                    target: Target::Element(selector),
                    by: candidate.by.clone(),
                });
            }
        }
        clock.wait(POLL).await;
    }
    Err(ResolveError::NotFound(SelectorError {
        label: raw.to_owned(),
        tried: candidates.into_iter().map(|c| c.by).collect(),
        dump: dump_visible(tab).await,
    }))
}

#[derive(Debug, Deserialize)]
struct OnScreen {
    tag: String,
    role: String,
    #[serde(rename = "type")]
    kind: String,
    text: String,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

fn listed(e: &OnScreen) -> String {
    let role = if e.role.is_empty() {
        String::new()
    } else {
        format!(" role={}", e.role)
    };
    let kind = if e.kind.is_empty() {
        String::new()
    } else {
        format!(" type={}", e.kind)
    };
    format!(
        "    · <{}{role}{kind}> \"{}\" @{},{} {}×{}",
        e.tag, e.text, e.x, e.y, e.w, e.h
    )
}

pub async fn dump_visible(tab: &Tab) -> String {
    let expression = call(
        LIST_ON_SCREEN,
        &json!({ "max": DUMP_LIMIT, "selector": format!("{},.ant-select", interactive_selector()) }),
    );
    let listing = tokio::time::timeout(
        NAVIGATION_TIMEOUT,
        tab.evaluate_main::<Vec<OnScreen>>(&expression),
    )
    .await;
    match listing {
        Ok(Ok(items)) => {
            let lines: Vec<String> = items.iter().map(listed).collect();
            let body = if lines.is_empty() {
                "    (no visible interactive elements)".to_owned()
            } else {
                lines.join("\n")
            };
            format!("  on screen now:\n{body}")
        }
        Ok(Err(error)) => format!("  (couldn't list what's on screen: {error})"),
        Err(_) => "  (couldn't list what's on screen: the page didn't answer)".to_owned(),
    }
}
