use std::borrow::Cow;
use std::sync::{Arc, OnceLock};

use chromiumoxide::Page;
use chromiumoxide::error::CdpError;
use chromiumoxide::types::{Command, Method, MethodId};
use opsis_compile::text::format_duration;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::{Value, json};
use tokio::sync::Mutex;

use crate::playwright::INJECTED_SOURCE;
use crate::take_clock::TakeClock;
use crate::trace::{CallInfo, Trace};

const UTILITY_WORLD: &str = "__opsis_utility_world__";
static PAGE_CHANGING: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"(?i)detached|not attached|Execution context was destroyed|navigat")
        .expect("pattern")
});
const LOST_CONTEXT: [&str; 4] = [
    "Cannot find context",
    "Execution context was destroyed",
    "Could not find object",
    "Cannot find default execution context",
];

#[derive(Debug, thiserror::Error)]
pub enum CaptureError {
    #[error("{0}")]
    Cdp(String),
    #[error("{0}")]
    Script(String),
    #[error("{0}")]
    Failed(String),
    #[error("Couldn't find the element on the page within {}.", format_duration(opsis_types::numbers::millis(*.0)))]
    NotFoundWithin(std::time::Duration),
    #[error("The browser closed before the action finished.")]
    Closed,
    #[error("Cancelled")]
    Cancelled,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Glob(#[from] crate::url_match::GlobError),
}

impl CaptureError {
    pub fn failed(message: impl Into<String>) -> Self {
        Self::Failed(message.into())
    }

    fn lost_context(&self) -> bool {
        matches!(self, Self::Cdp(m) | Self::Script(m) if LOST_CONTEXT.iter().any(|l| m.contains(l)))
    }

    pub fn page_changing(&self) -> bool {
        self.lost_context()
            || matches!(self, Self::Cdp(m) | Self::Script(m) if PAGE_CHANGING.is_match(m))
    }
}

impl From<CdpError> for CaptureError {
    fn from(error: CdpError) -> Self {
        match error {
            CdpError::Ws(_) | CdpError::NoResponse | CdpError::ChannelSendError(_) => Self::Closed,
            other => Self::Cdp(other.to_string()),
        }
    }
}

pub type Result<T> = std::result::Result<T, CaptureError>;

#[derive(Debug)]
pub struct Raw {
    method: Cow<'static, str>,
    params: Value,
}

impl Serialize for Raw {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        self.params.serialize(serializer)
    }
}

impl Method for Raw {
    fn identifier(&self) -> MethodId {
        self.method.clone()
    }
}

impl Command for Raw {
    type Response = Value;
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemoteObject {
    #[serde(default)]
    object_id: Option<String>,
    #[serde(default)]
    value: Option<Value>,
    #[serde(default)]
    subtype: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExceptionDetails {
    text: String,
    #[serde(default)]
    exception: Option<RemoteException>,
}

#[derive(Debug, Deserialize)]
struct RemoteException {
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Evaluated {
    result: RemoteObject,
    #[serde(default)]
    exception_details: Option<ExceptionDetails>,
}

impl Evaluated {
    fn into_result(self) -> Result<RemoteObject> {
        match self.exception_details {
            Some(details) => Err(CaptureError::Script(
                details
                    .exception
                    .and_then(|e| e.description)
                    .unwrap_or(details.text),
            )),
            None => Ok(self.result),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handle(pub String);

#[derive(Debug)]
pub enum Arg<'a> {
    Value(Value),
    Handle(&'a Handle),
}

impl Arg<'_> {
    fn call_argument(&self) -> Value {
        match self {
            Arg::Value(value) => json!({ "value": value }),
            Arg::Handle(handle) => json!({ "objectId": handle.0 }),
        }
    }
}

#[derive(Debug)]
pub struct Tab {
    pub page: Page,
    utility: Mutex<Option<(i64, Handle)>>,
    trace: OnceLock<Arc<Trace>>,
    clock: TakeClock,
}

impl Tab {
    pub fn new(page: Page) -> Self {
        Self {
            page,
            utility: Mutex::new(None),
            trace: OnceLock::new(),
            clock: TakeClock::default(),
        }
    }

    pub fn clock(&self) -> &TakeClock {
        &self.clock
    }

    pub fn set_trace(&self, trace: Arc<Trace>) {
        if self.trace.set(trace).is_err() {
            log::debug!("the take already has a trace");
        }
    }

    pub fn trace(&self) -> Option<&Arc<Trace>> {
        self.trace.get()
    }

    pub async fn traced<T>(
        &self,
        info: impl FnOnce() -> CallInfo,
        run: impl Future<Output = Result<T>>,
    ) -> Result<T> {
        let Some(trace) = self.trace.get() else {
            return run.await;
        };
        let call = trace.begin(info());
        let done = run.await;
        trace.end(call, done.as_ref().err().map(ToString::to_string), None);
        done
    }

    pub async fn send(&self, method: &'static str, params: Value) -> Result<Value> {
        Ok(self
            .page
            .execute(Raw {
                method: method.into(),
                params,
            })
            .await?
            .result)
    }

    pub async fn send_as<T: DeserializeOwned>(
        &self,
        method: &'static str,
        params: Value,
    ) -> Result<T> {
        serde_json::from_value(self.send(method, params).await?)
            .map_err(|e| CaptureError::Cdp(format!("{method}: {e}")))
    }

    pub async fn main_frame_id(&self) -> Result<String> {
        let tree = self.send("Page.getFrameTree", json!({})).await?;
        tree["frameTree"]["frame"]["id"]
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| CaptureError::Cdp("The page has no main frame.".into()))
    }

    pub async fn evaluate_main<T: DeserializeOwned>(&self, expression: &str) -> Result<T> {
        let evaluated: Evaluated = self
            .send_as(
                "Runtime.evaluate",
                json!({ "expression": expression, "returnByValue": true, "awaitPromise": true }),
            )
            .await?;
        let value = evaluated.into_result()?.value.unwrap_or(Value::Null);
        serde_json::from_value(value).map_err(|e| CaptureError::Script(e.to_string()))
    }

    async fn utility(&self) -> Result<(i64, Handle)> {
        let mut held = self.utility.lock().await;
        if let Some(utility) = held.as_ref() {
            return Ok(utility.clone());
        }
        let frame_id = self.main_frame_id().await?;
        let world = self.send("Page.createIsolatedWorld", json!({ "frameId": frame_id, "worldName": UTILITY_WORLD, "grantUniveralAccess": true })).await?;
        let context = world["executionContextId"]
            .as_i64()
            .ok_or_else(|| CaptureError::Cdp("No isolated world was made.".into()))?;
        let options = json!({
            "isUnderTest": false,
            "sdkLanguage": "javascript",
            "frameSeq": 0,
            "testIdAttributeName": crate::locators::TEST_ID_ATTRIBUTE,
            "stableRafCount": 1,
            "browserName": "chromium",
            "shouldPrependErrorPrefix": false,
            "isUtilityWorld": true,
            "customEngines": [],
        });
        let source = format!(
            "(() => {{\nconst module = {{}};\n{INJECTED_SOURCE}\nreturn new (module.exports.InjectedScript())(globalThis, {options});\n}})()"
        );
        let evaluated: Evaluated = self
            .send_as(
                "Runtime.evaluate",
                json!({ "expression": source, "contextId": context, "returnByValue": false }),
            )
            .await?;
        let injected = evaluated
            .into_result()?
            .object_id
            .map(Handle)
            .ok_or_else(|| CaptureError::Cdp("The page script didn't start.".into()))?;
        *held = Some((context, injected.clone()));
        Ok((context, injected))
    }

    async fn call_raw(
        &self,
        on: &Handle,
        function: &str,
        args: &[Arg<'_>],
        by_value: bool,
    ) -> Result<RemoteObject> {
        let arguments: Vec<Value> = args.iter().map(Arg::call_argument).collect();
        let evaluated: Evaluated = self
            .send_as("Runtime.callFunctionOn", json!({ "objectId": on.0, "functionDeclaration": function, "arguments": arguments, "returnByValue": by_value, "awaitPromise": true }))
            .await?;
        evaluated.into_result()
    }

    async fn call_injected(
        &self,
        function: &str,
        args: &[Arg<'_>],
        by_value: bool,
    ) -> Result<RemoteObject> {
        let (_, injected) = self.utility().await?;
        match self
            .call_raw(&injected, function, &with_first(&injected, args), by_value)
            .await
        {
            Err(error) if error.lost_context() => {
                *self.utility.lock().await = None;
                let (_, injected) = self.utility().await?;
                self.call_raw(&injected, function, &with_first(&injected, args), by_value)
                    .await
            }
            done => done,
        }
    }

    pub async fn injected_value<T: DeserializeOwned>(
        &self,
        function: &str,
        args: &[Arg<'_>],
    ) -> Result<T> {
        let value = self
            .call_injected(function, args, true)
            .await?
            .value
            .unwrap_or(Value::Null);
        serde_json::from_value(value).map_err(|e| CaptureError::Script(e.to_string()))
    }

    pub async fn injected_handle(
        &self,
        function: &str,
        args: &[Arg<'_>],
    ) -> Result<Option<Handle>> {
        let object = self.call_injected(function, args, false).await?;
        Ok(if object.subtype.as_deref() == Some("null") {
            None
        } else {
            object.object_id.map(Handle)
        })
    }

    pub async fn call_on<T: DeserializeOwned>(
        &self,
        on: &Handle,
        function: &str,
        args: &[Arg<'_>],
    ) -> Result<T> {
        let value = self
            .call_raw(on, function, args, true)
            .await?
            .value
            .unwrap_or(Value::Null);
        serde_json::from_value(value).map_err(|e| CaptureError::Script(e.to_string()))
    }

    pub async fn release(&self, handle: &Handle) {
        if let Err(error) = self
            .send("Runtime.releaseObject", json!({ "objectId": handle.0 }))
            .await
        {
            log::debug!("releasing a page object: {error}");
        }
    }
}

fn with_first<'a>(injected: &'a Handle, args: &[Arg<'a>]) -> Vec<Arg<'a>> {
    std::iter::once(Arg::Handle(injected))
        .chain(args.iter().map(|a| match a {
            Arg::Value(v) => Arg::Value(v.clone()),
            Arg::Handle(h) => Arg::Handle(h),
        }))
        .collect()
}
