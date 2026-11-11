use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use mlua::chunk::ChunkMode;
use mlua::{Error, Function, Lua, MultiValue, Result, Table, Value, Variadic};
use opsis_types::script::{SCRIPT_EXTENSION, SCRIPTS_DIR, ScriptLineKind};

use crate::folder::Folder;
use crate::output::Streams;

const LOADED_KEY: &str = "opsis.loaded";

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Halt {
    #[error("The script was stopped.")]
    Stopped,
    #[error("The script ended.")]
    Exited,
}

#[derive(Debug)]
pub struct Control {
    pub stop: Arc<AtomicBool>,
    pub exit: Cell<Option<bool>>,
}

impl Control {
    pub fn halted(&self) -> Option<Halt> {
        if self.exit.get().is_some() {
            Some(Halt::Exited)
        } else if self.stop.load(Ordering::Relaxed) {
            Some(Halt::Stopped)
        } else {
            None
        }
    }

    pub fn check(&self) -> Result<()> {
        self.halted()
            .map_or(Ok(()), |halt| Err(Error::external(halt)))
    }
}

fn shown(lua: &Lua, tostring: &Function, values: Variadic<Value>) -> Result<String> {
    let mut parts = Vec::with_capacity(values.len());
    for value in values {
        let text: mlua::LuaString = tostring.call(value)?;
        parts.push(text.to_string_lossy());
    }
    let _ = lua;
    Ok(parts.join("\t"))
}

pub fn chunk_name(path: &str) -> String {
    format!("@{path}")
}

fn load_file(
    lua: &Lua,
    folder: &Folder,
    path: &str,
    env: Option<Table>,
) -> std::result::Result<Function, String> {
    let text = folder.read_text(path).map_err(|e| e.to_string())?;
    let mut chunk = lua
        .load(text)
        .set_name(chunk_name(path))
        .set_mode(ChunkMode::Text);
    if let Some(env) = env {
        chunk = chunk.set_environment(env);
    }
    chunk.into_function().map_err(|e| e.to_string())
}

const GUARD: &str = "local check, pcall, xpcall, resume, close = ...
_G.pcall = function(...) return check(pcall(...)) end
_G.xpcall = function(...) return check(xpcall(...)) end
coroutine.resume = function(...) return check(resume(...)) end
coroutine.close = function(...) return check(close(...)) end";

const PLAIN: &str = "local pcall, error, tostring, pack, unpack = ...
local function plain(f)
  return function(...)
    local result = pack(pcall(f, ...))
    if result[1] then
      return unpack(result, 2, result.n)
    end
    error(tostring(result[2]), 0)
  end
end
for _, name in ipairs({ 'require', 'dofile', 'print', 'warn' }) do
  _G[name] = plain(_G[name])
end
for _, name in ipairs({ 'open', 'lines', 'read', 'write', 'input', 'output', 'close', 'flush' }) do
  io[name] = plain(io[name])
end
os.exit = plain(os.exit)
return plain";

pub const PLAIN_KEY: &str = "opsis.plain";

fn plain_errors(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let table: Table = globals.get("table")?;
    let plain: Function = lua
        .load(PLAIN)
        .set_name("=opsis")
        .set_mode(ChunkMode::Text)
        .call((
            globals.get::<Function>("pcall")?,
            globals.get::<Function>("error")?,
            globals.get::<Function>("tostring")?,
            table.get::<Function>("pack")?,
            table.get::<Function>("unpack")?,
        ))?;
    lua.set_named_registry_value(PLAIN_KEY, plain)
}

fn guard_protected(lua: &Lua, control: &Rc<Control>) -> Result<()> {
    let control = Rc::clone(control);
    let check = lua.create_function(move |_, values: MultiValue| {
        control.check()?;
        Ok(values)
    })?;
    let globals = lua.globals();
    let coroutine: Table = globals.get("coroutine")?;
    lua.load(GUARD)
        .set_name("=opsis")
        .set_mode(ChunkMode::Text)
        .call::<()>((
            check,
            globals.get::<Function>("pcall")?,
            globals.get::<Function>("xpcall")?,
            coroutine.get::<Function>("resume")?,
            coroutine.get::<Function>("close")?,
        ))
}

fn module_paths(name: &str) -> Vec<String> {
    let base = name.replace('.', "/");
    vec![
        format!("{SCRIPTS_DIR}/{base}.{SCRIPT_EXTENSION}"),
        format!("{SCRIPTS_DIR}/{base}/init.{SCRIPT_EXTENSION}"),
    ]
}

fn install_basic(lua: &Lua, folder: &Rc<Folder>, streams: &Rc<Streams>) -> Result<()> {
    let globals = lua.globals();
    let tostring: Function = globals.get("tostring")?;

    let (s, t) = (Rc::clone(streams), tostring.clone());
    globals.set(
        "print",
        lua.create_function(move |lua, values: Variadic<Value>| {
            s.line(ScriptLineKind::Print, &shown(lua, &t, values)?);
            Ok(())
        })?,
    )?;

    let (s, t) = (Rc::clone(streams), tostring);
    globals.set(
        "warn",
        lua.create_function(move |lua, values: Variadic<Value>| {
            let text = shown(lua, &t, values)?.replace('\t', "");
            if !text.starts_with('@') {
                s.line(ScriptLineKind::Warn, &text);
            }
            Ok(())
        })?,
    )?;

    let load: Function = globals.get("load")?;
    globals.set(
        "load",
        lua.create_function(move |lua, args: MultiValue| {
            let mut args: Vec<Value> = args.into_iter().collect();
            while args.len() < 3 {
                args.push(Value::Nil);
            }
            args[2] = Value::String(lua.create_string("t")?);
            load.call::<MultiValue>(MultiValue::from_iter(args))
        })?,
    )?;

    let f = Rc::clone(folder);
    globals.set(
        "loadfile",
        lua.create_function(
            move |lua, (path, _mode, env): (Option<String>, Value, Option<Table>)| {
                let path =
                    path.ok_or_else(|| crate::failed("Scripts have no input to load from."))?;
                Ok(match load_file(lua, &f, &path, env) {
                    Ok(function) => MultiValue::from_iter([Value::Function(function)]),
                    Err(message) => MultiValue::from_iter([
                        Value::Nil,
                        Value::String(lua.create_string(message)?),
                    ]),
                })
            },
        )?,
    )?;

    let f = Rc::clone(folder);
    globals.set(
        "dofile",
        lua.create_function(move |lua, path: Option<String>| {
            let path = path.ok_or_else(|| crate::failed("Scripts have no input to load from."))?;
            load_file(lua, &f, &path, None)
                .map_err(crate::failed)?
                .call::<MultiValue>(())
        })?,
    )?;

    lua.set_named_registry_value(LOADED_KEY, lua.create_table()?)?;
    let f = Rc::clone(folder);
    globals.set(
        "require",
        lua.create_function(move |lua, name: String| {
            let loaded: Table = lua.named_registry_value(LOADED_KEY)?;
            if let Some(value) = loaded.get::<Option<Value>>(name.as_str())? {
                return Ok(value);
            }
            let tried = module_paths(&name);
            let path = tried.iter().find(|p| f.exists(p)).ok_or_else(|| {
                crate::failed(format!(
                    "module '{name}' not found: no file {}",
                    tried.join(", no file ")
                ))
            })?;
            let function = load_file(lua, &f, path, None).map_err(crate::failed)?;
            let value = match function.call::<Value>((name.as_str(), path.as_str()))? {
                Value::Nil => Value::Boolean(true),
                value => value,
            };
            loaded.set(name.as_str(), &value)?;
            Ok(value)
        })?,
    )?;

    let string: Table = globals.get("string")?;
    string.set("dump", Value::Nil)?;
    Ok(())
}

fn install_os(lua: &Lua, folder: &Rc<Folder>, control: &Rc<Control>) -> Result<()> {
    let os: Table = lua.globals().get("os")?;
    for name in ["execute", "getenv", "setlocale", "tmpname"] {
        os.set(name, Value::Nil)?;
    }
    let f = Rc::clone(folder);
    os.set(
        "remove",
        lua.create_function(move |lua, path: String| {
            Ok(match f.remove(&path) {
                Ok(()) => MultiValue::from_iter([Value::Boolean(true)]),
                Err(error) => MultiValue::from_iter([
                    Value::Nil,
                    Value::String(lua.create_string(error.to_string())?),
                    Value::Integer(error.code()),
                ]),
            })
        })?,
    )?;
    let f = Rc::clone(folder);
    os.set(
        "rename",
        lua.create_function(move |lua, (from, to): (String, String)| {
            Ok(match f.rename(&from, &to) {
                Ok(()) => MultiValue::from_iter([Value::Boolean(true)]),
                Err(error) => MultiValue::from_iter([
                    Value::Nil,
                    Value::String(lua.create_string(error.to_string())?),
                    Value::Integer(error.code()),
                ]),
            })
        })?,
    )?;
    let c = Rc::clone(control);
    os.set(
        "exit",
        lua.create_function(move |_, code: Value| -> Result<()> {
            let ok = matches!(code, Value::Nil | Value::Boolean(true) | Value::Integer(0));
            c.exit.set(Some(ok));
            Err(Error::external(Halt::Exited))
        })?,
    )?;
    Ok(())
}

pub fn install(
    lua: &Lua,
    folder: &Rc<Folder>,
    streams: &Rc<Streams>,
    control: &Rc<Control>,
) -> Result<()> {
    install_basic(lua, folder, streams)?;
    install_os(lua, folder, control)?;
    let io = crate::io::install(lua, folder, streams)?;
    lua.globals().set("io", io)?;
    plain_errors(lua)?;
    guard_protected(lua, control)
}
