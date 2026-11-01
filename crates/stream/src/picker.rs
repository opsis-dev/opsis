use std::sync::Arc;

use chromiumoxide::cdp::browser_protocol::page::EventLoadEventFired;
use chromiumoxide::cdp::js_protocol::runtime::EventBindingCalled;
use futures::StreamExt;
use opsis_capture::cdp::{Result, Tab};
use opsis_capture::page_scripts::interactive_selector;
use opsis_types::actions::Viewport;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio_util::sync::CancellationToken;

use crate::host::{ViewEnv, ViewHost};

const PICKER: &str = include_str!("picker.js");
const BINDING: &str = "opsisPick";
const TITLE: &str = "Pick an element — click it, or Shift-click a spot";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub enum PickedBy {
    #[serde(rename = "accessible name")]
    AccessibleName,
    #[serde(rename = "css")]
    Css,
    #[serde(rename = "spot")]
    Spot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PickResult {
    pub target: String,
    pub css: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    pub selector_used: PickedBy,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, specta::Type)]
pub struct PickRequest {
    pub url: String,
    #[serde(default)]
    pub viewport: Option<Viewport>,
}

fn picker_script() -> String {
    format!(
        "(() => {{\n{PICKER}\ninstallPicker({});\n}})()",
        json!(format!("{},label", interactive_selector()))
    )
}

async fn install(tab: &Tab) {
    if let Err(error) = tab.evaluate_main::<Value>(&picker_script()).await {
        log::debug!("starting the picker: {error}");
    }
}

pub async fn pick(
    env: &ViewEnv,
    host: &dyn ViewHost,
    request: &PickRequest,
    cancel: &CancellationToken,
) -> Result<Option<PickResult>> {
    let view = env.open("about:blank", request.viewport).await?;
    let tab = view.active_tab().await;
    let picked = async {
        let mut calls = tab.page.event_listener::<EventBindingCalled>().await?;
        let mut loads = tab.page.event_listener::<EventLoadEventFired>().await?;
        tab.send("Runtime.addBinding", json!({ "name": BINDING }))
            .await?;
        view.navigate(&request.url).await?;
        let closed = CancellationToken::new();
        host.show(TITLE, Arc::clone(&view), closed.clone());
        install(&tab).await;
        loop {
            tokio::select! {
                () = cancel.cancelled() => return Ok(None),
                () = closed.cancelled() => return Ok(None),
                Some(_) = loads.next() => install(&tab).await,
                Some(event) = calls.next() => {
                    if event.name != BINDING {
                        continue;
                    }
                    match serde_json::from_str::<PickResult>(&event.payload) {
                        Ok(result) => return Ok(Some(result)),
                        Err(error) => log::debug!("an unreadable pick: {error}"),
                    }
                }
                else => return Ok(None),
            }
        }
    }
    .await;
    host.hide(&view);
    drop(tab);
    view.close().await;
    picked
}
