use std::sync::{Arc, Mutex};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chromiumoxide::cdp::browser_protocol::fetch::EventRequestPaused;
use futures::StreamExt;
use serde_json::{Value, json};
use tokio::task::JoinHandle;

use crate::cdp::{CaptureError, Result, Tab};
use crate::url_match::url_matches;

#[derive(Debug, Clone)]
pub struct MockResponse {
    pub url: String,
    pub status: u32,
    pub content_type: String,
    pub body: String,
}

#[derive(Default)]
pub struct Routes {
    list: Arc<Mutex<Vec<MockResponse>>>,
    listener: Option<JoinHandle<()>>,
}

impl Drop for Routes {
    fn drop(&mut self) {
        if let Some(listener) = &self.listener {
            listener.abort();
        }
    }
}

fn fulfilled(id: &str, mock: &MockResponse) -> Value {
    let headers = json!([
        { "name": "content-type", "value": mock.content_type },
        { "name": "content-length", "value": mock.body.len().to_string() },
    ]);
    json!({ "requestId": id, "responseCode": mock.status, "responseHeaders": headers, "body": STANDARD.encode(&mock.body) })
}

impl Routes {
    pub async fn add(&mut self, tab: &Arc<Tab>, mock: MockResponse) -> Result<()> {
        url_matches("", &mock.url).map_err(CaptureError::Failed)?;
        self.list.lock().expect("routes").push(mock);
        if self.listener.is_some() {
            return Ok(());
        }
        let mut paused = tab.page.event_listener::<EventRequestPaused>().await?;
        let (list, intercepting) = (Arc::clone(&self.list), Arc::clone(tab));
        let listener = tokio::spawn(async move {
            while let Some(event) = paused.next().await {
                let id = event.request_id.inner().clone();
                let url = format!(
                    "{}{}",
                    event.request.url,
                    event.request.url_fragment.clone().unwrap_or_default()
                );
                let found = list
                    .lock()
                    .expect("routes")
                    .iter()
                    .rev()
                    .find(|m| url_matches(&url, &m.url).unwrap_or(false))
                    .cloned();
                let answered = match found {
                    Some(mock) => {
                        intercepting
                            .send("Fetch.fulfillRequest", fulfilled(&id, &mock))
                            .await
                    }
                    None => {
                        intercepting
                            .send("Fetch.continueRequest", json!({ "requestId": id }))
                            .await
                    }
                };
                if let Err(error) = answered {
                    log::debug!("answering an intercepted request: {error}");
                }
            }
        });
        self.listener = Some(listener);
        tab.send(
            "Fetch.enable",
            json!({ "patterns": [{ "urlPattern": "*", "requestStage": "Request" }] }),
        )
        .await?;
        Ok(())
    }
}
