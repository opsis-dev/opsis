use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use chromiumoxide::cdp::browser_protocol::page::EventFrameNavigated;
use futures::StreamExt;
use indexmap::IndexMap;
use opsis_types::actions::Point;
use serde_json::{Value, json};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::cdp::{Arg, Result, Tab};
use opsis_compile::nodes::browser::Chapter;

const ADD: &str = "(injected, id, html) => injected.addUserOverlay(id, html)";
const REMOVE: &str = "(injected, id) => injected.removeUserOverlay(id)";
const FADE_OUT: &str = "(injected, id) => { injected.getUserOverlay(id)?.querySelector('#background')?.classList.add('fade-out'); }";
const ANNOTATE: &str = "(injected, annotation) => injected.setScreencastAnnotation(annotation)";
const RESTORE_CURSOR: &str = "(injected, point) => injected.restoreScreencastCursor(point)";

#[derive(Default)]
struct Shown {
    overlays: IndexMap<String, String>,
    cursor: Option<Point>,
}

pub struct Overlays {
    tab: Arc<Tab>,
    shown: Arc<Mutex<Shown>>,
    next_id: AtomicU64,
    annotation: Arc<AtomicU64>,
    show_actions: bool,
    label_ms: u64,
    restorer: JoinHandle<()>,
}

impl Drop for Overlays {
    fn drop(&mut self) {
        self.restorer.abort();
    }
}

fn faded(html: &str, fade_ms: u32) -> String {
    format!(
        "<style>@keyframes opsis-fade-in {{ from {{ opacity: 0; }} to {{ opacity: 1; }} }}
@keyframes opsis-fade-out {{ from {{ opacity: 1; }} to {{ opacity: 0; }} }}
#background {{ position: absolute; inset: 0; animation: opsis-fade-in {fade_ms}ms ease-out forwards; }}
#background.fade-out {{ animation: opsis-fade-out {fade_ms}ms ease-in forwards; }}</style><div id=\"background\">{html}</div>"
    )
}

fn chapter_html(p: &Chapter) -> String {
    let (title, description) = (p.title.as_str(), p.description.as_str());
    let (fade, blur, width) = (p.fade_ms, p.blur, p.card_width);
    let (card, edge, text) = (&p.card_color, &p.edge_color, &p.text_color);
    let (title_size, description_size) = (p.title_size, p.description_size);
    let place = p.position.flex();
    let description = if description.is_empty() {
        String::new()
    } else {
        format!(
            r#"<div id="description">{}</div>"#,
            html_escape::encode_text(description)
        )
    };
    let style = format!(
        "@keyframes pw-chapter-fade-in {{ from {{ opacity: 0; }} to {{ opacity: 1; }} }}
@keyframes pw-chapter-fade-out {{ from {{ opacity: 1; }} to {{ opacity: 0; }} }}
#background {{ position: absolute; inset: 0; display: flex; align-items: {place}; justify-content: center; padding: 48px 0; box-sizing: border-box; backdrop-filter: blur({blur}px); animation: pw-chapter-fade-in {fade}ms ease-out forwards; }}
#background.fade-out {{ animation: pw-chapter-fade-out {fade}ms ease-in forwards; }}
#content {{ background: {card}; border: 1px solid {edge}; border-radius: 16px; padding: 40px 56px; max-width: {width}px; box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4); }}
#title {{ color: {text}; font-family: system-ui, -apple-system, sans-serif; font-size: {title_size}px; font-weight: 600; line-height: 1.3; text-align: center; letter-spacing: -0.01em; }}
#description {{ color: color-mix(in srgb, {text} 70%, transparent); font-family: system-ui, -apple-system, sans-serif; font-size: {description_size}px; line-height: 1.5; margin-top: 12px; text-align: center; }}"
    );
    format!(
        r#"<style>{style}</style><div id="background"><div id="content"><div id="title">{}</div>{description}</div></div>"#,
        html_escape::encode_text(title)
    )
}

async fn add(tab: &Tab, id: &str, html: &str) {
    if let Err(error) = tab
        .injected_value::<Value>(ADD, &[Arg::Value(json!(id)), Arg::Value(json!(html))])
        .await
    {
        log::debug!("showing an overlay: {error}");
    }
}

impl Overlays {
    pub async fn start(tab: &Arc<Tab>, show_actions: bool, label_ms: u32) -> Result<Self> {
        let shown: Arc<Mutex<Shown>> = Arc::default();
        let mut navigated = tab.page.event_listener::<EventFrameNavigated>().await?;
        let (t, s) = (Arc::clone(tab), Arc::clone(&shown));
        let restorer = tokio::spawn(async move {
            while let Some(event) = navigated.next().await {
                if event.frame.parent_id.is_some() {
                    continue;
                }
                let (overlays, cursor) = {
                    let shown = s.lock().await;
                    (shown.overlays.clone(), shown.cursor)
                };
                for (id, html) in &overlays {
                    add(&t, id, html).await;
                }
                if let Some(point) = cursor
                    && let Err(error) = t
                        .injected_value::<Value>(RESTORE_CURSOR, &[Arg::Value(json!(point))])
                        .await
                {
                    log::debug!("restoring the action cursor: {error}");
                }
            }
        });
        Ok(Self {
            tab: Arc::clone(tab),
            shown,
            next_id: AtomicU64::new(0),
            annotation: Arc::default(),
            show_actions,
            label_ms: u64::from(label_ms),
            restorer,
        })
    }

    pub async fn show(&self, html: &str) -> String {
        let id = format!(
            "opsis-overlay-{}",
            self.next_id.fetch_add(1, Ordering::Relaxed)
        );
        self.shown
            .lock()
            .await
            .overlays
            .insert(id.clone(), html.to_owned());
        add(&self.tab, &id, html).await;
        id
    }

    pub async fn remove(&self, id: &str) {
        self.shown.lock().await.overlays.shift_remove(id);
        if let Err(error) = self
            .tab
            .injected_value::<Value>(REMOVE, &[Arg::Value(json!(id))])
            .await
        {
            log::debug!("removing an overlay: {error}");
        }
    }

    pub async fn show_faded(&self, html: &str, fade_ms: u32) -> String {
        if fade_ms == 0 {
            return self.show(html).await;
        }
        self.show(&faded(html, fade_ms)).await
    }

    async fn fade_out(&self, id: &str, fade_ms: u32) {
        if fade_ms == 0 {
            return;
        }
        if let Err(error) = self
            .tab
            .injected_value::<Value>(FADE_OUT, &[Arg::Value(json!(id))])
            .await
        {
            log::debug!("fading out an overlay: {error}");
            return;
        }
        self.tab
            .clock()
            .wait(Duration::from_millis(u64::from(fade_ms)))
            .await;
    }

    pub fn remove_after(self: &Arc<Self>, id: String, after: Duration, fade_ms: u32) {
        let overlays = Arc::clone(self);
        tokio::spawn(async move {
            overlays.tab.clock().wait(after).await;
            overlays.fade_out(&id, fade_ms).await;
            overlays.remove(&id).await;
        });
    }

    pub async fn chapter(&self, p: &Chapter) {
        let id = self.show(&chapter_html(p)).await;
        self.tab
            .clock()
            .wait(Duration::from_millis(u64::from(p.duration_ms)))
            .await;
        self.fade_out(&id, p.fade_ms).await;
        self.remove(&id).await;
    }

    pub fn annotate(&self, title: &str, point: Option<Point>) {
        if !self.show_actions {
            return;
        }
        let generation = self.annotation.fetch_add(1, Ordering::Relaxed) + 1;
        let (tab, shown, current) = (
            Arc::clone(&self.tab),
            Arc::clone(&self.shown),
            Arc::clone(&self.annotation),
        );
        let title = title.to_owned();
        let label_ms = self.label_ms;
        tokio::spawn(async move {
            let last = {
                let mut shown = shown.lock().await;
                let last = shown.cursor;
                if point.is_some() {
                    shown.cursor = point;
                }
                last
            };
            let mut annotation =
                json!({ "actionTitle": title, "duration": label_ms, "cursor": "pointer" });
            if let Some(point) = point {
                annotation["point"] = json!(point);
            }
            if let Some(last) = last {
                annotation["lastCursorPoint"] = json!(last);
            }
            if let Err(error) = tab
                .injected_value::<Value>(ANNOTATE, &[Arg::Value(annotation)])
                .await
            {
                log::debug!("showing an action: {error}");
                return;
            }
            tab.clock().wait(Duration::from_millis(label_ms)).await;
            if current.load(Ordering::Relaxed) == generation
                && let Err(error) = tab
                    .injected_value::<Value>(ANNOTATE, &[Arg::Value(Value::Null)])
                    .await
            {
                log::debug!("hiding an action: {error}");
            }
        });
    }
}
