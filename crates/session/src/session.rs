use std::fmt;
use std::sync::Arc;

use opsis_store::keys::{KeyError, Keyring};
use tokio::sync::broadcast;

use crate::analysis::Mixes;
use crate::api::Api;
use crate::events::Addressed;
use crate::exit::Exit;
use crate::node_results::NodeKeys;
use crate::paths::AppPaths;
use crate::platform::Platform;
use crate::run::RunHost;
use crate::shared::Shared;
use crate::startup::StartupError;
use crate::state::AppState;
use crate::terminal_live::LiveTapes;
use crate::updates::Updates;
use crate::views::Views;

const EVENTS_KEPT: usize = 1024;

struct Parts {
    state: AppState,
    run_host: RunHost,
    shared: Shared,
    exit: Exit,
    views: Views,
    live_tapes: LiveTapes,
    mixes: Mixes,
    node_keys: NodeKeys,
    api: Api,
    updates: Updates,
    events: broadcast::Sender<Addressed>,
    platform: Arc<dyn Platform>,
}

#[derive(Clone)]
pub struct Session(Arc<Parts>);

impl fmt::Debug for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Session")
            .field("paths", &self.0.state.paths)
            .finish_non_exhaustive()
    }
}

pub trait Part {
    fn of(session: &Session) -> &Self;
}

macro_rules! parts {
    ($($kind:ty => $field:ident),* $(,)?) => {
        $(impl Part for $kind {
            fn of(session: &Session) -> &Self {
                &session.0.$field
            }
        })*
    };
}

parts!(
    AppState => state,
    RunHost => run_host,
    Shared => shared,
    Exit => exit,
    Views => views,
    LiveTapes => live_tapes,
    Mixes => mixes,
    NodeKeys => node_keys,
    Api => api,
    Updates => updates,
);

impl Session {
    pub async fn start(
        paths: AppPaths,
        keys: Result<Keyring, KeyError>,
        platform: Arc<dyn Platform>,
    ) -> Result<Self, StartupError> {
        crate::runtime::adopt(tokio::runtime::Handle::current());
        let state = AppState::new(paths, keys).await?;
        let api = Api::new(&state);
        Ok(Self(Arc::new(Parts {
            state,
            run_host: RunHost::default(),
            shared: Shared::default(),
            exit: Exit::default(),
            views: Views::default(),
            live_tapes: LiveTapes::default(),
            mixes: Mixes::default(),
            node_keys: NodeKeys::default(),
            api,
            updates: Updates::default(),
            events: broadcast::Sender::new(EVENTS_KEPT),
            platform,
        })))
    }

    pub fn begin(&self) {
        crate::api::follow_setting(self);
        #[cfg(unix)]
        crate::exit::close_on_signals(self);
        crate::engine::verify_on_start(self);
        crate::updates::begin(self);
    }

    pub fn end(&self) {
        self.state::<RunHost>().cancel();
        let ended = opsis_engine::children::end_remembered_children();
        if !ended.is_empty() {
            log::info!("ended the browsers still open: {ended:?}");
        }
        crate::api::stop(self);
        opsis_engine::temp::remove_child_temp(&self.state::<AppState>().paths.views_temp);
    }

    pub fn state<T: Part>(&self) -> &T {
        T::of(self)
    }

    pub fn platform(&self) -> &dyn Platform {
        self.0.platform.as_ref()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Addressed> {
        self.0.events.subscribe()
    }

    pub(crate) fn send(&self, addressed: Addressed) -> bool {
        self.0.events.send(addressed).is_ok()
    }
}
