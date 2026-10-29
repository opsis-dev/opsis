use opsis_engine::children::{forget_child, remember_child};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::target::CreateTargetParams;
use futures::StreamExt;
use indexmap::IndexMap;
use opsis_compile::nodes::browser::{ColorScheme, Open};
use opsis_engine::tools::{ToolProblem, browser_version, tool_output, unrecognized};
use opsis_types::actions::Viewport;
use opsis_types::engine::EngineTool;
use opsis_types::numbers::whole;
use serde_json::{Value, json};
use tokio::task::JoinHandle;

use crate::cdp::{CaptureError, Result, Tab};
use crate::page_scripts::{FIT_PAGE_TO_SCALE, call};
use crate::playwright::tables;
use crate::still_layout::{Size, scaled_viewport};

const HEADLESS_SWITCHES: [&str; 6] = [
    "--enable-unsafe-swiftshader",
    "--headless",
    "--hide-scrollbars",
    "--mute-audio",
    "--blink-settings=primaryHoverType=2,availableHoverTypes=2,primaryPointerType=4,availablePointerTypes=4",
    "--no-sandbox",
];
const TAKE_SWITCHES: [&str; 2] = ["--hide-scrollbars", "--disable-features=Translate"];
const STARTUP_SWITCHES: [&str; 1] = ["--no-startup-window"];
const MOBILE_USER_AGENT: &str = "Mozilla/5.0 (iPad; CPU OS 12_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/27.2 Mobile/15E148 Safari/604.1";
const LAUNCH_TIMEOUT: Duration = Duration::from_secs(30);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);
const FALLBACK_LOCALE: &str = "en-US";

fn switch_key(switch: &str) -> &str {
    let bare = switch.trim_start_matches('-');
    bare.split_once('=').map_or(bare, |(key, _)| key)
}

const FEATURE_LISTS: [&str; 2] = ["enable-features", "disable-features"];

fn joined_features(earlier: &str, switch: &str) -> String {
    let mut features: Vec<&str> = Vec::new();
    for list in [earlier, switch] {
        let values = list.split_once('=').map_or("", |(_, v)| v);
        for feature in values.split(',').filter(|f| !f.is_empty()) {
            if !features.contains(&feature) {
                features.push(feature);
            }
        }
    }
    format!("{}={}", switch_key(switch), features.join(","))
}

pub fn effective_switches<'a>(lists: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    let mut last: IndexMap<String, String> = IndexMap::new();
    for switch in lists {
        let key = switch_key(switch).to_owned();
        let bare = switch.trim_start_matches('-');
        let value = match last.get(&key) {
            Some(earlier) if FEATURE_LISTS.contains(&key.as_str()) => {
                joined_features(earlier, bare)
            }
            _ => bare.to_owned(),
        };
        last.insert(key, value);
    }
    last.into_values().collect()
}

fn playwright_switches() -> Vec<String> {
    tables()["switches"]
        .as_array()
        .expect("the launch switches are a list")
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

pub struct Launched {
    pub browser: Browser,
    handler: JoinHandle<()>,
    _profile: tempfile::TempDir,
    platform: &'static str,
    pid: Option<u32>,
}

impl Drop for Launched {
    fn drop(&mut self) {
        if let Some(pid) = self.pid {
            forget_child(pid);
        }
    }
}

impl Launched {
    pub async fn close(mut self) {
        if let Err(error) = self.browser.close().await {
            log::debug!("closing the browser: {error}");
        }
        if let Err(error) = self.browser.wait().await {
            log::debug!("waiting for the browser: {error}");
        }
        self.handler.abort();
    }
}

pub async fn launch(executable: &Path, temp_dir: &Path, extra: &[&str]) -> Result<Launched> {
    let profile = tempfile::Builder::new()
        .prefix("opsis-profile-")
        .tempdir_in(temp_dir)?;
    let defaults = playwright_switches();
    let switches = effective_switches(
        defaults
            .iter()
            .map(String::as_str)
            .chain(HEADLESS_SWITCHES)
            .chain(TAKE_SWITCHES)
            .chain(STARTUP_SWITCHES)
            .chain(extra.iter().copied()),
    );
    let mut config = BrowserConfig::builder()
        .chrome_executable(executable)
        .disable_default_args()
        .with_head()
        .viewport(None)
        .user_data_dir(profile.path())
        .launch_timeout(LAUNCH_TIMEOUT)
        .request_timeout(REQUEST_TIMEOUT);
    for (key, value) in [("TMPDIR", temp_dir), ("TEMP", temp_dir), ("TMP", temp_dir)] {
        config = config.env(key, value.to_string_lossy());
    }
    for switch in switches {
        config = config.arg(switch);
    }
    let config = config.build().map_err(CaptureError::Failed)?;
    let (mut browser, mut handler) = Browser::launch(config).await?;
    let pid = browser.get_mut_child().and_then(|child| child.inner.id());
    if let Some(pid) = pid {
        remember_child(pid);
    }
    let handler = tokio::spawn(async move {
        while let Some(event) = handler.next().await {
            if let Err(error) = event {
                log::debug!("browser connection: {error}");
            }
        }
    });
    let version = browser.version().await?;
    let platform = if version.user_agent.contains("Windows") {
        "win"
    } else if version.user_agent.contains("Macintosh") {
        "mac"
    } else {
        "linux"
    };
    Ok(Launched {
        browser,
        handler,
        _profile: profile,
        platform,
        pid,
    })
}

pub struct Session {
    pub launched: Launched,
    pub tab: Arc<Tab>,
    pub viewport: Viewport,
    pub dpr: f64,
    pub user_agent: String,
    pub locale: String,
}

fn color_scheme(scheme: ColorScheme) -> &'static str {
    match scheme {
        ColorScheme::Light => "light",
        ColorScheme::Dark => "dark",
        ColorScheme::NoPreference => "no-preference",
    }
}

fn system_locale() -> String {
    sys_locale::get_locale().unwrap_or_else(|| FALLBACK_LOCALE.to_owned())
}

pub fn user_agent_metadata(ua: &str, mobile: bool) -> Option<Value> {
    if ua.is_empty() {
        return None;
    }
    let found = |pattern: &str| {
        regex::Regex::new(pattern)
            .expect("pattern")
            .captures(ua)
            .map(|c| c[1].to_owned())
    };
    let (mut platform, mut version, mut architecture) = ("Windows", String::new(), "x86");
    if let Some(v) = found(r"Android (\d+(\.\d+)?(\.\d+)?)") {
        (platform, version, architecture) = ("Android", v, "arm");
    } else if let Some(v) = found(r"iPhone OS (\d+(_\d+)?)") {
        (platform, version, architecture) = ("iOS", v, "arm");
    } else if let Some(v) = found(r"iPad; CPU OS (\d+(_\d+)?)") {
        (platform, version, architecture) = ("iOS", v, "arm");
    } else if let Some(v) = found(r"Mac OS X (\d+(_\d+)?(_\d+)?)") {
        (platform, version) = ("macOS", v);
        if !ua.contains("Intel") {
            architecture = "arm";
        }
    } else if let Some(v) = found(r"Windows\D+(\d+(\.\d+)?(\.\d+)?)") {
        (platform, version) = ("Windows", v);
    } else if ua.to_lowercase().contains("linux") {
        platform = "Linux";
    }
    if ua.contains("ARM") {
        architecture = "arm";
    }
    Some(
        json!({ "mobile": mobile, "model": "", "architecture": architecture, "platform": platform, "platformVersion": version }),
    )
}

impl Session {
    pub async fn open(params: &Open, executable: &Path, temp_dir: &Path) -> Result<Self> {
        Self::open_with_switches(params, executable, temp_dir, &[]).await
    }

    pub async fn open_with_switches(
        params: &Open,
        executable: &Path,
        temp_dir: &Path,
        switches: &[&str],
    ) -> Result<Self> {
        let launched = launch(executable, temp_dir, switches).await?;
        let page = match new_page(&launched).await {
            Ok(page) => page,
            Err(error) => {
                launched.close().await;
                return Err(error);
            }
        };
        let physical = scaled_viewport(
            Size {
                width: f64::from(params.viewport_width),
                height: f64::from(params.viewport_height),
            },
            params.capture_scale,
        );
        let locale = match params.locale.trim() {
            "" => system_locale(),
            given => given.to_owned(),
        };
        let user_agent = if !params.user_agent.is_empty() {
            params.user_agent.clone()
        } else if params.mobile {
            MOBILE_USER_AGENT.to_owned()
        } else {
            String::new()
        };
        let session = Self {
            tab: Arc::new(Tab::new(page)),
            viewport: Viewport {
                width: whole(physical.width),
                height: whole(physical.height),
            },
            dpr: params.dpr,
            user_agent,
            locale,
            launched,
        };
        match session.prepare(params).await {
            Ok(()) => Ok(session),
            Err(error) => {
                session.close().await;
                Err(error)
            }
        }
    }

    async fn prepare(&self, params: &Open) -> Result<()> {
        self.emulate(params).await?;
        if params.capture_scale > 1.0 {
            let fit = call(
                FIT_PAGE_TO_SCALE,
                &json!({ "scale": params.capture_scale, "width": params.viewport_width, "height": params.viewport_height }),
            );
            self.tab
                .send(
                    "Page.addScriptToEvaluateOnNewDocument",
                    json!({ "source": fit, "runImmediately": true }),
                )
                .await?;
        }
        Ok(())
    }

    async fn emulate(&self, params: &Open) -> Result<()> {
        let tab = &self.tab;
        let landscape = self.viewport.width > self.viewport.height;
        let orientation = if params.mobile && !landscape {
            json!({ "angle": 0, "type": "portraitPrimary" })
        } else if params.mobile {
            json!({ "angle": 90, "type": "landscapePrimary" })
        } else {
            json!({ "angle": 0, "type": "landscapePrimary" })
        };
        tab.send(
            "Emulation.setFocusEmulationEnabled",
            json!({ "enabled": true }),
        )
        .await?;
        tab.send(
            "Emulation.setDeviceMetricsOverride",
            json!({
                "mobile": params.mobile,
                "width": self.viewport.width,
                "height": self.viewport.height,
                "screenWidth": self.viewport.width,
                "screenHeight": self.viewport.height,
                "deviceScaleFactor": params.dpr,
                "screenOrientation": orientation,
            }),
        )
        .await?;
        if params.mobile {
            tab.send(
                "Emulation.setTouchEmulationEnabled",
                json!({ "enabled": true }),
            )
            .await?;
        }
        let mut agent = json!({ "userAgent": self.user_agent, "acceptLanguage": self.locale });
        if let Some(metadata) = user_agent_metadata(&self.user_agent, params.mobile) {
            agent["userAgentMetadata"] = metadata;
        }
        tab.send("Emulation.setUserAgentOverride", agent).await?;
        if let Err(error) = tab
            .send(
                "Emulation.setLocaleOverride",
                json!({ "locale": self.locale }),
            )
            .await
            && !error
                .to_string()
                .contains("Another locale override is already in effect")
        {
            return Err(CaptureError::Failed(format!(
                "Can't use the language “{}”.",
                self.locale
            )));
        }
        if !params.timezone.trim().is_empty() {
            tab.send(
                "Emulation.setTimezoneOverride",
                json!({ "timezoneId": params.timezone.trim() }),
            )
            .await
            .map_err(|_| {
                CaptureError::Failed(format!(
                    "Can't use the time zone “{}”.",
                    params.timezone.trim()
                ))
            })?;
        }
        tab.send(
            "Page.setFontFamilies",
            tables()["fontFamilies"][self.launched.platform].clone(),
        )
        .await?;
        let features = json!([
            { "name": "prefers-color-scheme", "value": color_scheme(params.color_scheme) },
            { "name": "prefers-reduced-motion", "value": if params.reduce_motion { "reduce" } else { "" } },
            { "name": "forced-colors", "value": "" },
            { "name": "prefers-contrast", "value": "" },
        ]);
        tab.send(
            "Emulation.setEmulatedMedia",
            json!({ "media": "", "features": features }),
        )
        .await?;
        if params.show_scrollbars {
            tab.send("Emulation.setScrollbarsHidden", json!({ "hidden": false }))
                .await?;
        }
        Ok(())
    }

    pub async fn add_init_script(&self, source: &str) -> Result<()> {
        self.tab
            .send(
                "Page.addScriptToEvaluateOnNewDocument",
                json!({ "source": source }),
            )
            .await
            .map(drop)
    }

    pub async fn close(self) {
        self.launched.close().await;
    }
}

async fn new_page(launched: &Launched) -> Result<chromiumoxide::Page> {
    let context = launched
        .browser
        .create_browser_context(Default::default())
        .await?;
    let target = CreateTargetParams::builder()
        .url("about:blank")
        .browser_context_id(context)
        .build()
        .map_err(CaptureError::Failed)?;
    Ok(launched.browser.new_page(target).await?)
}

pub async fn check_browser(
    executable: &Path,
    temp_dir: &Path,
) -> std::result::Result<String, ToolProblem> {
    let version = browser_version(&tool_output(executable, &["--version"]).await?)
        .ok_or_else(|| unrecognized(executable, EngineTool::Browser))?;
    let launched = launch(executable, temp_dir, &[])
        .await
        .map_err(|e| ToolProblem::Failed(format!("The recording browser didn't start: {e}")))?;
    let opened = launched.browser.new_page("about:blank").await;
    launched.close().await;
    opened.map_err(|e| {
        ToolProblem::Failed(format!("The recording browser couldn't open a page: {e}"))
    })?;
    Ok(version)
}

#[cfg(test)]
mod tests {
    use super::effective_switches;

    #[test]
    fn keeps_the_last_value_of_a_switch_and_joins_feature_lists() {
        assert_eq!(
            effective_switches([
                "--hide-scrollbars",
                "--lang=en",
                "--enable-features=A,B",
                "--disable-features=X",
                "--lang=de",
                "--enable-features=B,C",
                "--disable-features=Y",
            ]),
            [
                "hide-scrollbars",
                "lang=de",
                "enable-features=A,B,C",
                "disable-features=X,Y",
            ]
        );
    }
}
