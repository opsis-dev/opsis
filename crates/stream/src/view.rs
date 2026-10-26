use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use chromiumoxide::cdp::browser_protocol::page::{
    EventFileChooserOpened, EventFrameNavigated, EventJavascriptDialogOpening,
    EventNavigatedWithinDocument,
};
use chromiumoxide::cdp::browser_protocol::target::{
    EventTargetCreated, EventTargetDestroyed, TargetId,
};
use futures::StreamExt;
use opsis_capture::browser::Session;
use opsis_capture::cdp::{CaptureError, Result, Tab};
use opsis_capture::live::FrameSink;
use opsis_capture::screencast::{FrameSinks, Screencast};
use opsis_compile::nodes::browser::Open;
use opsis_types::actions::Viewport;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::task::JoinHandle;

const POPUP_ATTACH_TRIES: usize = 20;
const POPUP_ATTACH_WAIT: std::time::Duration = std::time::Duration::from_millis(50);
const UNBRANDED: (&str, &str) = ("Not.A/Brand", "99");

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ViewSize {
    pub width: f64,
    pub height: f64,
    pub dpr: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct TabInfo {
    pub id: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, specta::Type)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ViewEvent {
    Tabs {
        tabs: Vec<TabInfo>,
        active: usize,
    },
    Dialog {
        dialog: String,
        message: String,
        default_prompt: String,
    },
    FileChooser {
        multiple: bool,
    },
}

pub trait ViewSink: FrameSink {
    fn event(&self, event: ViewEvent);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum MouseButton {
    None,
    Left,
    Middle,
    Right,
    Back,
    Forward,
}

#[derive(Debug, Clone, PartialEq, Deserialize, specta::Type)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ViewInput {
    MouseDown {
        x: f64,
        y: f64,
        button: MouseButton,
        buttons: i64,
        click_count: i64,
        modifiers: i64,
    },
    MouseUp {
        x: f64,
        y: f64,
        button: MouseButton,
        buttons: i64,
        click_count: i64,
        modifiers: i64,
    },
    MouseMove {
        x: f64,
        y: f64,
        buttons: i64,
        modifiers: i64,
    },
    Wheel {
        x: f64,
        y: f64,
        delta_x: f64,
        delta_y: f64,
        modifiers: i64,
    },
    KeyDown {
        key: String,
        code: String,
        text: String,
        key_code: i64,
        location: i64,
        modifiers: i64,
        repeat: bool,
    },
    KeyUp {
        key: String,
        code: String,
        key_code: i64,
        location: i64,
        modifiers: i64,
    },
    Text {
        text: String,
    },
    Composition {
        text: String,
        selection_start: i64,
        selection_end: i64,
    },
}

const KEYPAD: i64 = 3;

fn mouse_event(
    kind: &str,
    x: f64,
    y: f64,
    button: MouseButton,
    buttons: i64,
    click_count: i64,
    modifiers: i64,
) -> Value {
    json!({ "type": kind, "x": x, "y": y, "button": button, "buttons": buttons, "clickCount": click_count, "modifiers": modifiers })
}

impl ViewInput {
    fn command(&self) -> (&'static str, Value) {
        match self {
            Self::MouseDown {
                x,
                y,
                button,
                buttons,
                click_count,
                modifiers,
            } => (
                "Input.dispatchMouseEvent",
                mouse_event(
                    "mousePressed",
                    *x,
                    *y,
                    *button,
                    *buttons,
                    *click_count,
                    *modifiers,
                ),
            ),
            Self::MouseUp {
                x,
                y,
                button,
                buttons,
                click_count,
                modifiers,
            } => (
                "Input.dispatchMouseEvent",
                mouse_event(
                    "mouseReleased",
                    *x,
                    *y,
                    *button,
                    *buttons,
                    *click_count,
                    *modifiers,
                ),
            ),
            Self::MouseMove {
                x,
                y,
                buttons,
                modifiers,
            } => (
                "Input.dispatchMouseEvent",
                json!({ "type": "mouseMoved", "x": x, "y": y, "buttons": buttons, "modifiers": modifiers }),
            ),
            Self::Wheel {
                x,
                y,
                delta_x,
                delta_y,
                modifiers,
            } => (
                "Input.dispatchMouseEvent",
                json!({ "type": "mouseWheel", "x": x, "y": y, "deltaX": delta_x, "deltaY": delta_y, "modifiers": modifiers }),
            ),
            Self::KeyDown {
                key,
                code,
                text,
                key_code,
                location,
                modifiers,
                repeat,
            } => (
                "Input.dispatchKeyEvent",
                json!({
                    "type": if text.is_empty() { "rawKeyDown" } else { "keyDown" },
                    "key": key,
                    "code": code,
                    "text": text,
                    "unmodifiedText": text,
                    "windowsVirtualKeyCode": key_code,
                    "nativeVirtualKeyCode": key_code,
                    "location": location,
                    "isKeypad": *location == KEYPAD,
                    "modifiers": modifiers,
                    "autoRepeat": repeat,
                }),
            ),
            Self::KeyUp {
                key,
                code,
                key_code,
                location,
                modifiers,
            } => (
                "Input.dispatchKeyEvent",
                json!({ "type": "keyUp", "key": key, "code": code, "windowsVirtualKeyCode": key_code, "nativeVirtualKeyCode": key_code, "location": location, "isKeypad": *location == KEYPAD, "modifiers": modifiers }),
            ),
            Self::Text { text } => ("Input.insertText", json!({ "text": text })),
            Self::Composition {
                text,
                selection_start,
                selection_end,
            } => (
                "Input.imeSetComposition",
                json!({ "text": text, "selectionStart": selection_start, "selectionEnd": selection_end }),
            ),
        }
    }
}

#[derive(Default)]
struct Slot {
    sink: Option<Arc<dyn ViewSink>>,
    last: Option<(Vec<u8>, f64, f64, f64)>,
}

#[derive(Default)]
struct SinkSlot(Mutex<Slot>);

impl SinkSlot {
    fn sink(&self) -> Option<Arc<dyn ViewSink>> {
        self.0.lock().expect("view sink").sink.clone()
    }

    fn event(&self, event: ViewEvent) {
        if let Some(sink) = self.sink() {
            sink.event(event);
        }
    }
}

impl FrameSink for SinkSlot {
    fn frame(&self, jpeg: &[u8], width: f64, height: f64, t: f64) {
        let sink = {
            let mut slot = self.0.lock().expect("view sink");
            match &slot.sink {
                Some(sink) => Arc::clone(sink),
                None => {
                    slot.last = Some((jpeg.to_vec(), width, height, t));
                    return;
                }
            }
        };
        sink.frame(jpeg, width, height, t);
    }
}

struct ViewTab {
    tab: Arc<Tab>,
    url: Arc<Mutex<String>>,
    listeners: Vec<JoinHandle<()>>,
}

impl Drop for ViewTab {
    fn drop(&mut self) {
        for listener in &self.listeners {
            listener.abort();
        }
    }
}

struct Tabs {
    list: Vec<ViewTab>,
    active: usize,
    screencast: Option<(Arc<Tab>, Screencast)>,
    size: ViewSize,
    file_chooser: Option<i64>,
}

pub struct StreamView {
    session: Session,
    slot: Arc<SinkSlot>,
    tabs: tokio::sync::Mutex<Tabs>,
    quality: i64,
    user_agent: Value,
    watcher: Mutex<Option<JoinHandle<()>>>,
}

fn platform_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "Windows"
    } else if cfg!(target_os = "macos") {
        "macOS"
    } else {
        "Linux"
    }
}

pub fn desktop_user_agent(headless: &str) -> Value {
    let user_agent = headless.replace("HeadlessChrome", "Chrome");
    let full = user_agent
        .split("Chrome/")
        .nth(1)
        .and_then(|rest| rest.split_whitespace().next())
        .unwrap_or_default()
        .to_owned();
    let major = full.split('.').next().unwrap_or_default().to_owned();
    let brands = json!([{ "brand": "Chromium", "version": major }, { "brand": UNBRANDED.0, "version": UNBRANDED.1 }]);
    let full_versions = json!([{ "brand": "Chromium", "version": full }, { "brand": UNBRANDED.0, "version": format!("{}.0.0.0", UNBRANDED.1) }]);
    let architecture = if cfg!(target_arch = "aarch64") {
        "arm"
    } else {
        "x86"
    };
    json!({
        "userAgent": user_agent,
        "userAgentMetadata": { "brands": brands, "fullVersionList": full_versions, "fullVersion": full, "platform": platform_name(), "platformVersion": "", "architecture": architecture, "model": "", "mobile": false },
    })
}

async fn prepare_tab(tab: &Tab, size: ViewSize, user_agent: &Value) -> Result<()> {
    tab.send("Emulation.setDeviceMetricsOverride", json!({ "mobile": false, "width": size.width, "height": size.height, "deviceScaleFactor": size.dpr })).await?;
    tab.send("Emulation.setUserAgentOverride", user_agent.clone())
        .await?;
    tab.send(
        "Page.setInterceptFileChooserDialog",
        json!({ "enabled": true }),
    )
    .await?;
    Ok(())
}

impl StreamView {
    pub async fn open(
        executable: &Path,
        temp_dir: &Path,
        url: &str,
        size: ViewSize,
        quality: i64,
        switches: &[&str],
    ) -> Result<Arc<Self>> {
        let params = Open {
            viewport_width: size.width as i64,
            viewport_height: size.height as i64,
            dpr: size.dpr,
            ..Open::default()
        };
        let session = Session::open_with_switches(&params, executable, temp_dir, switches).await?;
        let user_agent = match session.launched.browser.user_agent().await {
            Ok(headless) => desktop_user_agent(&headless),
            Err(error) => {
                session.close().await;
                return Err(error.into());
            }
        };
        let tab = Arc::clone(&session.tab);
        let view = Arc::new(Self {
            session,
            slot: Arc::default(),
            tabs: tokio::sync::Mutex::new(Tabs {
                list: Vec::new(),
                active: 0,
                screencast: None,
                size,
                file_chooser: None,
            }),
            quality,
            user_agent,
            watcher: Mutex::new(None),
        });
        let opened = async {
            view.add_tab(tab).await?;
            view.watch_popups().await?;
            view.navigate(url).await
        }
        .await;
        match opened {
            Ok(()) => Ok(view),
            Err(error) => {
                view.close().await;
                Err(error)
            }
        }
    }

    pub fn attach(&self, sink: Arc<dyn ViewSink>) {
        let last = {
            let mut slot = self.slot.0.lock().expect("view sink");
            slot.sink = Some(Arc::clone(&sink));
            slot.last.take()
        };
        if let Some((jpeg, width, height, t)) = last {
            sink.frame(&jpeg, width, height, t);
        }
    }

    pub async fn active_tab(&self) -> Arc<Tab> {
        let tabs = self.tabs.lock().await;
        Arc::clone(&tabs.list[tabs.active].tab)
    }

    pub async fn url(&self) -> String {
        let tabs = self.tabs.lock().await;
        tabs.list[tabs.active]
            .url
            .lock()
            .expect("tab address")
            .clone()
    }

    async fn tab_info(&self) {
        let tabs = self.tabs.lock().await;
        let list = tabs
            .list
            .iter()
            .map(|t| TabInfo {
                id: t.tab.page.target_id().inner().clone(),
                url: t.url.lock().expect("tab address").clone(),
            })
            .collect();
        self.slot.event(ViewEvent::Tabs {
            tabs: list,
            active: tabs.active,
        });
    }

    async fn add_tab(self: &Arc<Self>, tab: Arc<Tab>) -> Result<()> {
        let size = self.tabs.lock().await.size;
        prepare_tab(&tab, size, &self.user_agent).await?;
        let url = Arc::new(Mutex::new(String::new()));
        let listeners = self.listen(&tab, &url).await?;
        let tree = tab.send("Page.getFrameTree", json!({})).await?;
        let frame = &tree["frameTree"]["frame"];
        let current = format!(
            "{}{}",
            frame["url"].as_str().unwrap_or_default(),
            frame["urlFragment"].as_str().unwrap_or_default()
        );
        {
            let mut known = url.lock().expect("tab address");
            if known.is_empty() {
                *known = current;
            }
        }
        let mut tabs = self.tabs.lock().await;
        tabs.list.push(ViewTab {
            tab,
            url,
            listeners,
        });
        tabs.active = tabs.list.len() - 1;
        self.restream(&mut tabs).await?;
        drop(tabs);
        self.tab_info().await;
        Ok(())
    }

    async fn listen(
        self: &Arc<Self>,
        tab: &Arc<Tab>,
        url: &Arc<Mutex<String>>,
    ) -> Result<Vec<JoinHandle<()>>> {
        let mut navigated = tab.page.event_listener::<EventFrameNavigated>().await?;
        let mut within = tab
            .page
            .event_listener::<EventNavigatedWithinDocument>()
            .await?;
        let mut dialogs = tab
            .page
            .event_listener::<EventJavascriptDialogOpening>()
            .await?;
        let mut choosers = tab.page.event_listener::<EventFileChooserOpened>().await?;
        let mut listeners = Vec::new();
        let (view, address) = (Arc::downgrade(self), Arc::clone(url));
        listeners.push(tokio::spawn(async move {
            while let Some(event) = navigated.next().await {
                if event.frame.parent_id.is_some() {
                    continue;
                }
                *address.lock().expect("tab address") = format!(
                    "{}{}",
                    event.frame.url,
                    event.frame.url_fragment.clone().unwrap_or_default()
                );
                let Some(view) = view.upgrade() else { return };
                view.tab_info().await;
            }
        }));
        let (view, address) = (Arc::downgrade(self), Arc::clone(url));
        listeners.push(tokio::spawn(async move {
            while let Some(event) = within.next().await {
                *address.lock().expect("tab address") = event.url.clone();
                let Some(view) = view.upgrade() else { return };
                view.tab_info().await;
            }
        }));
        let slot = Arc::clone(&self.slot);
        listeners.push(tokio::spawn(async move {
            while let Some(event) = dialogs.next().await {
                slot.event(ViewEvent::Dialog {
                    dialog: event.r#type.as_ref().to_owned(),
                    message: event.message.clone(),
                    default_prompt: event.default_prompt.clone().unwrap_or_default(),
                });
            }
        }));
        let view = Arc::downgrade(self);
        listeners.push(tokio::spawn(async move {
            while let Some(event) = choosers.next().await {
                let Some(view) = view.upgrade() else { return };
                view.tabs.lock().await.file_chooser =
                    event.backend_node_id.as_ref().map(|id| *id.inner());
                view.slot.event(ViewEvent::FileChooser {
                    multiple: event.mode.as_ref() == "selectMultiple",
                });
            }
        }));
        Ok(listeners)
    }

    async fn watch_popups(self: &Arc<Self>) -> Result<()> {
        let browser = &self.session.launched.browser;
        let mut created = browser.event_listener::<EventTargetCreated>().await?;
        let mut destroyed = browser.event_listener::<EventTargetDestroyed>().await?;
        let view = Arc::downgrade(self);
        let watcher = tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(event) = created.next() => {
                        let info = &event.target_info;
                        let Some(view) = view.upgrade() else { return };
                        if info.r#type != "page" || info.opener_id.is_none() || !view.owns(info.opener_id.as_ref()).await {
                            continue;
                        }
                        if let Err(error) = view.adopt(info.target_id.clone()).await {
                            log::debug!("showing a popup: {error}");
                        }
                    }
                    Some(event) = destroyed.next() => {
                        let Some(view) = view.upgrade() else { return };
                        view.forget(&event.target_id).await;
                    }
                    else => return,
                }
            }
        });
        *self.watcher.lock().expect("popup watcher") = Some(watcher);
        Ok(())
    }

    async fn owns(&self, target: Option<&TargetId>) -> bool {
        let tabs = self.tabs.lock().await;
        target.is_some_and(|id| tabs.list.iter().any(|t| t.tab.page.target_id() == id))
    }

    async fn adopt(self: &Arc<Self>, target: TargetId) -> Result<()> {
        for _ in 0..POPUP_ATTACH_TRIES {
            if let Ok(page) = self.session.launched.browser.get_page(target.clone()).await {
                return self.add_tab(Arc::new(Tab::new(page))).await;
            }
            tokio::time::sleep(POPUP_ATTACH_WAIT).await;
        }
        Err(CaptureError::failed("The popup couldn't be shown."))
    }

    async fn forget(&self, target: &TargetId) {
        let mut tabs = self.tabs.lock().await;
        let Some(index) = tabs
            .list
            .iter()
            .position(|t| t.tab.page.target_id() == target)
        else {
            return;
        };
        if tabs.list.len() == 1 {
            return;
        }
        tabs.list.remove(index);
        if tabs.active >= index {
            tabs.active = tabs.active.saturating_sub(1);
            if let Err(error) = self.restream(&mut tabs).await {
                log::debug!("streaming the tab left open: {error}");
            }
        }
        drop(tabs);
        self.tab_info().await;
    }

    async fn restream(&self, tabs: &mut Tabs) -> Result<()> {
        if let Some((streamed, screencast)) = tabs.screencast.take()
            && let Err(error) = screencast.stop(&streamed).await
        {
            log::debug!("stopping the stream: {error}");
        }
        let tab = &tabs.list[tabs.active].tab;
        let size = Viewport {
            width: tabs.size.width * tabs.size.dpr,
            height: tabs.size.height * tabs.size.dpr,
        };
        let sink: Arc<dyn FrameSink> = self.slot.clone();
        tabs.screencast = Some((
            Arc::clone(tab),
            Screencast::start(
                tab,
                size,
                self.quality,
                FrameSinks {
                    view: Some(sink),
                    ..FrameSinks::default()
                },
            )
            .await?,
        ));
        Ok(())
    }

    pub async fn input(&self, input: &ViewInput) -> Result<()> {
        let (method, params) = input.command();
        self.active_tab().await.send(method, params).await.map(drop)
    }

    pub async fn resize(&self, size: ViewSize) -> Result<()> {
        let mut tabs = self.tabs.lock().await;
        tabs.size = size;
        for tab in &tabs.list {
            prepare_tab(&tab.tab, size, &self.user_agent).await?;
        }
        self.restream(&mut tabs).await
    }

    pub async fn switch_tab(&self, id: &str) -> Result<()> {
        let mut tabs = self.tabs.lock().await;
        let Some(index) = tabs
            .list
            .iter()
            .position(|t| t.tab.page.target_id().inner() == id)
        else {
            return Ok(());
        };
        tabs.active = index;
        self.restream(&mut tabs).await?;
        drop(tabs);
        self.tab_info().await;
        Ok(())
    }

    pub async fn navigate(&self, url: &str) -> Result<()> {
        self.active_tab()
            .await
            .send("Page.navigate", json!({ "url": url }))
            .await
            .map(drop)
    }

    pub async fn reload(&self) -> Result<()> {
        self.active_tab()
            .await
            .send("Page.reload", json!({}))
            .await
            .map(drop)
    }

    pub async fn go(&self, steps: i64) -> Result<()> {
        let tab = self.active_tab().await;
        let history = tab.send("Page.getNavigationHistory", json!({})).await?;
        let index = history["currentIndex"].as_i64().unwrap_or(0) + steps;
        let Some(entry) = history["entries"]
            .as_array()
            .and_then(|e| usize::try_from(index).ok().and_then(|i| e.get(i)))
        else {
            return Ok(());
        };
        tab.send(
            "Page.navigateToHistoryEntry",
            json!({ "entryId": entry["id"] }),
        )
        .await
        .map(drop)
    }

    pub async fn answer_dialog(&self, accept: bool, prompt: &str) -> Result<()> {
        self.active_tab()
            .await
            .send(
                "Page.handleJavaScriptDialog",
                json!({ "accept": accept, "promptText": prompt }),
            )
            .await
            .map(drop)
    }

    pub async fn choose_files(&self, files: &[PathBuf]) -> Result<()> {
        let node = self.tabs.lock().await.file_chooser.take();
        let Some(node) = node else {
            return Ok(());
        };
        let files: Vec<String> = files
            .iter()
            .map(|f| f.to_string_lossy().into_owned())
            .collect();
        self.active_tab()
            .await
            .send(
                "DOM.setFileInputFiles",
                json!({ "files": files, "backendNodeId": node }),
            )
            .await
            .map(drop)
    }

    pub fn browser(&self) -> &chromiumoxide::Browser {
        &self.session.launched.browser
    }

    pub async fn close(self: Arc<Self>) {
        if let Some(watcher) = self.watcher.lock().expect("popup watcher").take() {
            watcher.abort();
        }
        let mut tabs = self.tabs.lock().await;
        tabs.screencast = None;
        tabs.list.clear();
        drop(tabs);
        match Arc::try_unwrap(self) {
            Ok(view) => view.session.close().await,
            Err(view) => log::debug!(
                "the view is still in use ({} holders)",
                Arc::strong_count(&view)
            ),
        }
    }
}
