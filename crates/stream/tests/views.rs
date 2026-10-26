use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use opsis_capture::executors::LoginRequest;
use opsis_capture::live::FrameSink;
use opsis_compile::nodes::browser::DoneWhen;
use opsis_engine::paths::downloaded_browser;
use opsis_stream::host::{ViewEnv, ViewHost};
use opsis_stream::picker::{PickRequest, PickResult, PickedBy, pick};
use opsis_stream::sign_in::sign_in;
use opsis_stream::view::{MouseButton, StreamView, ViewEvent, ViewInput, ViewSink};
use opsis_types::secrets::StorageState;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::sync::CancellationToken;

const LOGIN: &str = "<!doctype html><body style='margin:0'><script>localStorage.setItem('token', 't')</script><a id=go href='/home' style='position:absolute;left:100px;top:100px;width:120px;height:40px;display:block'>Go</a></body>";
const HOME: &str = "<!doctype html><body>home</body>";
const PICK: &str = "<!doctype html><body style='margin:0'><button style='position:absolute;left:100px;top:100px;width:120px;height:40px'>Save draft</button></body>";
const POPUP: &str = "<!doctype html><body style='margin:0'><button onclick=\"window.open('/home')\" style='position:absolute;left:100px;top:100px;width:120px;height:40px'>Open</button></body>";

fn env(temp: &tempfile::TempDir) -> ViewEnv {
    let engine = PathBuf::from(std::env::var_os("HOME").unwrap()).join(".config/Opsis/engine");
    ViewEnv {
        executable: downloaded_browser(&engine).unwrap(),
        temp_dir: temp.path().to_path_buf(),
    }
}

async fn serve() -> u16 {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            let mut request = [0; 4096];
            let Ok(read) = socket.read(&mut request).await else {
                continue;
            };
            let line = String::from_utf8_lossy(&request[..read])
                .lines()
                .next()
                .unwrap_or_default()
                .to_owned();
            let (page, cookie) = match line.split_whitespace().nth(1).unwrap_or("/") {
                "/login" => (LOGIN, "set-cookie: sid=1; Path=/\r\n"),
                "/pick" => (PICK, ""),
                "/popup" => (POPUP, ""),
                _ => (HOME, ""),
            };
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: text/html\r\n{cookie}content-length: {}\r\nconnection: close\r\n\r\n{page}",
                page.len()
            );
            socket.write_all(response.as_bytes()).await.unwrap();
        }
    });
    port
}

#[derive(Default)]
struct Seen {
    frames: AtomicUsize,
    events: Mutex<Vec<ViewEvent>>,
}

impl FrameSink for Seen {
    fn frame(&self, _: &[u8], _: f64, _: f64, _: f64) {
        self.frames.fetch_add(1, Ordering::Relaxed);
    }
}

impl ViewSink for Seen {
    fn event(&self, event: ViewEvent) {
        self.events.lock().unwrap().push(event);
    }
}

type Script = Box<dyn Fn(Arc<StreamView>) -> futures::future::BoxFuture<'static, ()> + Send + Sync>;

struct Host {
    seen: Arc<Seen>,
    script: Script,
}

impl ViewHost for Host {
    fn show(&self, _: &str, view: Arc<StreamView>, _: CancellationToken) {
        view.attach(self.seen.clone());
        tokio::spawn((self.script)(view));
    }

    fn hide(&self, _: &Arc<StreamView>) {}
}

fn host(
    script: impl Fn(Arc<StreamView>) -> futures::future::BoxFuture<'static, ()> + Send + Sync + 'static,
) -> Host {
    Host {
        seen: Arc::default(),
        script: Box::new(script),
    }
}

async fn click(view: &StreamView, x: f64, y: f64, modifiers: i64) {
    tokio::time::sleep(Duration::from_millis(400)).await;
    for input in [
        ViewInput::MouseMove {
            x,
            y,
            buttons: 0,
            modifiers,
        },
        ViewInput::MouseDown {
            x,
            y,
            button: MouseButton::Left,
            buttons: 1,
            click_count: 1,
            modifiers,
        },
        ViewInput::MouseUp {
            x,
            y,
            button: MouseButton::Left,
            buttons: 0,
            click_count: 1,
            modifiers,
        },
    ] {
        view.input(&input).await.unwrap();
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the dev engine"]
async fn picks_the_element_clicked_in_the_view() {
    let (temp, port) = (tempfile::tempdir().unwrap(), serve().await);
    let host = host(|view| Box::pin(async move { click(&view, 160.0, 120.0, 0).await }));
    let picked = pick(
        &env(&temp),
        &host,
        &PickRequest {
            url: format!("http://127.0.0.1:{port}/pick"),
            viewport: None,
        },
        &CancellationToken::new(),
    )
    .await
    .unwrap();
    assert_eq!(
        picked,
        Some(PickResult {
            target: "Save draft".into(),
            css: "html > body > button".into(),
            role: Some("button".into()),
            selector_used: PickedBy::AccessibleName
        })
    );
    assert!(host.seen.frames.load(Ordering::Relaxed) > 0);
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the dev engine"]
async fn picks_a_spot_on_shift_click() {
    let (temp, port) = (tempfile::tempdir().unwrap(), serve().await);
    let host = host(|view| Box::pin(async move { click(&view, 160.0, 120.0, 8).await }));
    let picked = pick(
        &env(&temp),
        &host,
        &PickRequest {
            url: format!("http://127.0.0.1:{port}/pick"),
            viewport: None,
        },
        &CancellationToken::new(),
    )
    .await
    .unwrap();
    assert_eq!(
        picked,
        Some(PickResult {
            target: "160, 120".into(),
            css: String::new(),
            role: Some(String::new()),
            selector_used: PickedBy::Spot
        })
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the dev engine"]
async fn signs_in_once_the_address_changes_and_keeps_cookies_and_storage() {
    let (temp, port) = (tempfile::tempdir().unwrap(), serve().await);
    let host = host(|view| Box::pin(async move { click(&view, 160.0, 120.0, 0).await }));
    let request = LoginRequest {
        url: format!("http://127.0.0.1:{port}/login"),
        done_when: DoneWhen::UrlChange,
        selector: String::new(),
        timeout_sec: 20,
        viewport: None,
    };
    let started = std::time::Instant::now();
    let state = sign_in(&env(&temp), &host, &request, &CancellationToken::new())
        .await
        .unwrap()
        .unwrap();
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "finished only at the timeout"
    );
    let state: StorageState = serde_json::from_str(&state).unwrap();
    assert_eq!(
        state
            .cookies
            .iter()
            .map(|c| (
                c.name.as_str(),
                c.value.as_str(),
                c.domain.as_str(),
                c.expires
            ))
            .collect::<Vec<_>>(),
        [("sid", "1", "127.0.0.1", -1.0)]
    );
    assert_eq!(state.origins.len(), 1);
    assert_eq!(state.origins[0].origin, format!("http://127.0.0.1:{port}"));
    assert_eq!(
        (
            state.origins[0].local_storage[0].name.as_str(),
            state.origins[0].local_storage[0].value.as_str()
        ),
        ("token", "t")
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the dev engine"]
async fn shows_a_popup_as_a_second_tab() {
    let (temp, port) = (tempfile::tempdir().unwrap(), serve().await);
    let view = env(&temp)
        .open(&format!("http://127.0.0.1:{port}/popup"), None)
        .await
        .unwrap();
    let seen = Arc::new(Seen::default());
    view.attach(seen.clone());
    click(&view, 160.0, 120.0, 0).await;
    tokio::time::sleep(Duration::from_millis(1500)).await;
    let tabs = seen
        .events
        .lock()
        .unwrap()
        .iter()
        .rev()
        .find_map(|e| match e {
            ViewEvent::Tabs { tabs, active } => {
                Some((tabs.len(), *active, tabs[*active].url.clone()))
            }
            _ => None,
        });
    assert_eq!(tabs, Some((2, 1, format!("http://127.0.0.1:{port}/home"))));
    assert!(view.url().await.ends_with("/home"));
    view.close().await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the dev engine"]
async fn signs_in_once_the_chosen_element_shows() {
    let (temp, port) = (tempfile::tempdir().unwrap(), serve().await);
    let host = host(|_| Box::pin(async {}));
    let request = LoginRequest {
        url: format!("http://127.0.0.1:{port}/login"),
        done_when: DoneWhen::Selector,
        selector: "#go".into(),
        timeout_sec: 20,
        viewport: None,
    };
    let started = std::time::Instant::now();
    let state: StorageState = serde_json::from_str(
        &sign_in(&env(&temp), &host, &request, &CancellationToken::new())
            .await
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "finished only at the timeout"
    );
    assert_eq!(state.cookies.len(), 1);
}
