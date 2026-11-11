use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};

use mlua::chunk::ChunkMode;
use mlua::{AnyUserData, Error, Function, Lua, LuaSerdeExt, Result, Table, UserData, Value};
use opsis_types::edits::{TimelineEdit, WorkflowEdit};
use opsis_types::events::{RunEvent, RunTargets};
use opsis_types::shell::PlaybackCall;
use serde::Serialize;

use crate::host::{RunEvents, RunNext, ScriptHost, run_of};
use crate::sandbox::Control;
use crate::values::{from_lua, to_lua};

const PRELUDE: &str = include_str!("opsis.lua");
const EVENT_WAIT: Duration = Duration::from_millis(50);
const SLEEP_SLICE: Duration = Duration::from_millis(20);
const STACK_DEPTH: usize = 64;

pub fn located(lua: &Lua, message: &str) -> Error {
    for level in 0..STACK_DEPTH {
        let place = lua.inspect_stack(level, |debug| {
            let source = debug.source();
            let file = source.source.as_deref()?.strip_prefix('@')?.to_owned();
            Some((file, debug.current_line()?))
        });
        match place {
            None => break,
            Some(Some((file, line))) => return crate::failed(format!("{file}:{line}: {message}")),
            Some(None) => {}
        }
    }
    crate::failed(message.to_owned())
}

struct RunHandle {
    id: String,
    events: RefCell<Box<dyn RunEvents>>,
    missed: Cell<u64>,
    ended: Cell<bool>,
    quiet: Cell<bool>,
}

impl UserData for RunHandle {}

struct Natives {
    host: Rc<dyn ScriptHost>,
    control: Rc<Control>,
}

impl Natives {
    fn answer<T: Serialize>(lua: &Lua, result: std::result::Result<T, String>) -> Result<Value> {
        match result {
            Ok(value) => to_lua(lua, &value),
            Err(message) => Err(located(lua, &message)),
        }
    }

    fn next_event(&self, lua: &Lua, handle: &RunHandle) -> Result<Value> {
        loop {
            self.control.check()?;
            if handle.ended.get() {
                return Ok(Value::Nil);
            }
            let next = handle.events.borrow_mut().next(EVENT_WAIT);
            match next {
                RunNext::Event(event) if run_of(&event) == handle.id => {
                    if matches!(*event, RunEvent::RunDone { .. }) {
                        handle.ended.set(true);
                    }
                    return to_lua(lua, &*event);
                }
                RunNext::Event(_) => {}
                RunNext::Missed(count) => handle.missed.set(handle.missed.get() + count),
                RunNext::Idle => {
                    if self.host.active_run().as_deref() != Some(handle.id.as_str()) {
                        if handle.quiet.get() {
                            handle.ended.set(true);
                        }
                        handle.quiet.set(true);
                    }
                }
            }
        }
    }

    fn sleep(&self, seconds: f64) -> Result<()> {
        let wait = Duration::try_from_secs_f64(seconds.max(0.0)).unwrap_or(Duration::MAX);
        let until = Instant::now().checked_add(wait);
        loop {
            self.control.check()?;
            let left = until.map_or(SLEEP_SLICE, |u| u.saturating_duration_since(Instant::now()));
            if left.is_zero() {
                return Ok(());
            }
            std::thread::sleep(left.min(SLEEP_SLICE));
        }
    }
}

fn edit_of<T: serde::de::DeserializeOwned>(lua: &Lua, value: Value, what: &str) -> Result<T> {
    from_lua(lua, value).map_err(|e| located(lua, &format!("That isn't a valid {what}: {e}")))
}

fn natives(lua: &Lua, host: Rc<dyn ScriptHost>, control: &Rc<Control>) -> Result<Table> {
    let n = Rc::new(Natives {
        host,
        control: Rc::clone(control),
    });
    let table = lua.create_table()?;
    macro_rules! native {
        ($name:literal, |$lua:ident, $natives:ident, $args:ident : $kind:ty| $body:expr) => {{
            let $natives = Rc::clone(&n);
            table.set(
                $name,
                lua.create_function(move |$lua, $args: $kind| {
                    let $natives = &$natives;
                    $natives.control.check()?;
                    $body
                })?,
            )?;
        }};
    }
    native!("version", |lua, n, _a: ()| to_lua(lua, &n.host.version()));
    native!("project", |lua, n, _a: ()| Natives::answer(
        lua,
        n.host.project()
    ));
    native!("workflow", |lua, n, _a: ()| Natives::answer(
        lua,
        n.host.workflow()
    ));
    native!("node", |lua, n, id: String| {
        let workflow = n.host.workflow().map_err(|e| located(lua, &e))?;
        match workflow.nodes.iter().find(|node| node.id == id) {
            Some(node) => to_lua(lua, node),
            None => Ok(Value::Nil),
        }
    });
    native!("edit", |lua, n, edit: Value| {
        let edit: WorkflowEdit = edit_of(lua, edit, "edit")?;
        Natives::answer(lua, n.host.edit(edit))
    });
    native!("timeline_edit", |lua, n, edit: Value| {
        let edit: TimelineEdit = edit_of(lua, edit, "timeline edit")?;
        let reply = n.host.edit_timeline(edit).map_err(|e| located(lua, &e))?;
        match reply.error {
            Some(error) => Err(located(lua, &error)),
            None => to_lua(lua, &reply.message),
        }
    });
    native!("save", |lua, n, _a: ()| Natives::answer(lua, n.host.save()));
    native!("problems", |lua, n, _a: ()| Natives::answer(
        lua,
        n.host.problems()
    ));
    native!("catalog", |lua, n, _a: ()| to_lua(lua, &n.host.catalog()));
    native!("compile", |lua, n, _a: ()| Natives::answer(
        lua,
        n.host.compile()
    ));
    native!("validate", |lua, n, workflow: Value| {
        let workflow: serde_json::Value = edit_of(lua, workflow, "project")?;
        to_lua(lua, &n.host.validate(&workflow))
    });
    native!("fonts", |lua, n, _a: ()| Natives::answer(
        lua,
        n.host.fonts()
    ));
    native!("use_font", |lua, n, id: String| Natives::answer(
        lua,
        n.host.use_font(&id)
    ));
    native!("run_start", |lua, n, targets: Value| {
        let targets: RunTargets = edit_of(lua, targets, "run")?;
        let (id, events) = n.host.start_run(targets).map_err(|e| located(lua, &e))?;
        Ok(Value::UserData(lua.create_userdata(RunHandle {
            id,
            events: RefCell::new(events),
            missed: Cell::new(0),
            ended: Cell::new(false),
            quiet: Cell::new(false),
        })?))
    });
    native!("run_next", |lua, n, handle: AnyUserData| {
        let handle = handle.borrow::<RunHandle>()?;
        n.next_event(lua, &handle)
    });
    native!("run_id", |lua, _n, handle: AnyUserData| {
        to_lua(lua, &handle.borrow::<RunHandle>()?.id)
    });
    native!("run_missed", |_lua, _n, handle: AnyUserData| {
        Ok(Value::Integer(
            i64::try_from(handle.borrow::<RunHandle>()?.missed.get()).unwrap_or(i64::MAX),
        ))
    });
    native!("run_stop", |_lua, n, _a: ()| {
        n.host.stop_run();
        Ok(Value::Nil)
    });
    native!("run_active", |lua, n, _a: ()| to_lua(
        lua,
        &n.host.active_run()
    ));
    native!("run_state", |lua, n, _a: ()| to_lua(
        lua,
        &n.host.run_state()
    ));
    native!("results", |lua, n, _a: ()| to_lua(lua, &n.host.results()));
    native!("clear_results", |lua, n, _a: ()| Natives::answer(
        lua,
        n.host.clear_results()
    ));
    native!("clear_log", |_lua, n, _a: ()| {
        n.host.clear_log();
        Ok(Value::Nil)
    });
    native!("playback", |lua, n, call: Value| {
        let call: PlaybackCall = edit_of(lua, call, "playback call")?;
        Natives::answer(lua, n.host.playback(call))
    });
    native!("shared", |lua, n, _a: ()| to_lua(lua, &n.host.shared()));
    native!("select", |lua, n, ids: Value| {
        let ids: Vec<String> = edit_of(lua, ids, "list of node ids")?;
        Natives::answer(lua, n.host.select(ids))
    });
    native!("sleep", |_lua, n, seconds: f64| {
        n.sleep(seconds)?;
        Ok(Value::Nil)
    });
    native!("list", |lua, _n, _a: ()| {
        let list = lua.create_table()?;
        list.set_metatable(Some(lua.array_metatable()))?;
        Ok(Value::Table(list))
    });
    native!("fail", |lua, _n, message: String| Err::<Value, _>(located(
        lua, &message
    )));
    table.set("null", lua.null())?;
    Ok(table)
}

pub fn install(lua: &Lua, host: Rc<dyn ScriptHost>, control: &Rc<Control>) -> Result<()> {
    let natives = natives(lua, host, control)?;
    let build: Function = lua
        .load(PRELUDE)
        .set_name("=opsis")
        .set_mode(ChunkMode::Text)
        .into_function()?;
    let plain: Function = lua.named_registry_value(crate::sandbox::PLAIN_KEY)?;
    let opsis: Table = build.call((natives, plain))?;
    lua.globals().set("opsis", opsis)
}
