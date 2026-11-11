use std::io::{Read, Seek, SeekFrom, Write};
use std::rc::Rc;

use mlua::{
    AnyUserData, Error, Function, Lua, MultiValue, ObjectLike, Result, Table, UserData,
    UserDataMethods, Value, Variadic,
};

use crate::folder::{FileError, Folder, Mode};
use crate::output::{Stream, Streams};

const CHUNK: usize = 8192;
const NUMBER_LONGEST: usize = 200;
const INPUT_KEY: &str = "opsis.io.input";
const OUTPUT_KEY: &str = "opsis.io.output";

#[derive(Debug)]
struct DiskFile {
    file: std::fs::File,
    buffer: Vec<u8>,
    at: usize,
}

impl DiskFile {
    fn fill(&mut self) -> std::io::Result<bool> {
        if self.at < self.buffer.len() {
            return Ok(true);
        }
        self.buffer.resize(CHUNK, 0);
        let read = self.file.read(&mut self.buffer)?;
        self.buffer.truncate(read);
        self.at = 0;
        Ok(read > 0)
    }

    fn peek(&mut self) -> std::io::Result<Option<u8>> {
        Ok(if self.fill()? {
            Some(self.buffer[self.at])
        } else {
            None
        })
    }

    fn drop_buffer(&mut self) -> std::io::Result<()> {
        let unread = self.buffer.len() - self.at;
        if unread > 0 {
            let back = i64::try_from(unread).map_err(std::io::Error::other)?;
            self.file.seek(SeekFrom::Current(-back))?;
        }
        self.buffer.clear();
        self.at = 0;
        Ok(())
    }

    fn line(&mut self, keep: bool) -> std::io::Result<Option<Vec<u8>>> {
        let mut line = Vec::new();
        let mut any = false;
        while self.fill()? {
            any = true;
            let rest = &self.buffer[self.at..];
            if let Some(end) = rest.iter().position(|b| *b == b'\n') {
                line.extend_from_slice(&rest[..end]);
                if keep {
                    line.push(b'\n');
                }
                self.at += end + 1;
                return Ok(Some(line));
            }
            line.extend_from_slice(rest);
            self.at = self.buffer.len();
        }
        Ok(any.then_some(line))
    }

    fn all(&mut self) -> std::io::Result<Vec<u8>> {
        let mut all = self.buffer[self.at..].to_vec();
        self.buffer.clear();
        self.at = 0;
        self.file.read_to_end(&mut all)?;
        Ok(all)
    }

    fn count(&mut self, count: usize) -> std::io::Result<Option<Vec<u8>>> {
        if count == 0 {
            return Ok(self.fill()?.then(Vec::new));
        }
        let mut read = Vec::new();
        while read.len() < count && self.fill()? {
            let rest = &self.buffer[self.at..];
            let take = rest.len().min(count - read.len());
            read.extend_from_slice(&rest[..take]);
            self.at += take;
        }
        Ok((!read.is_empty()).then_some(read))
    }

    fn number_text(&mut self) -> std::io::Result<String> {
        while let Some(byte) = self.peek()? {
            if !byte.is_ascii_whitespace() {
                break;
            }
            self.at += 1;
        }
        let mut text = String::new();
        while let Some(byte) = self.peek()? {
            let allowed = byte.is_ascii_hexdigit() || b"+-.xXpP".contains(&byte);
            if !allowed || text.len() >= NUMBER_LONGEST {
                break;
            }
            text.push(char::from(byte));
            self.at += 1;
        }
        Ok(text)
    }
}

#[derive(Debug)]
enum Target {
    Disk(DiskFile),
    Stream(Stream),
}

pub struct LuaFile {
    target: Option<Target>,
    streams: Rc<Streams>,
}

impl std::fmt::Debug for LuaFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LuaFile")
            .field("target", &self.target)
            .finish_non_exhaustive()
    }
}

fn io_failed(error: &std::io::Error) -> Error {
    crate::failed(error.to_string())
}

fn fail(lua: &Lua, message: String, code: i64) -> Result<MultiValue> {
    Ok(MultiValue::from_iter([
        Value::Nil,
        Value::String(lua.create_string(message)?),
        Value::Integer(code),
    ]))
}

fn file_failure(lua: &Lua, error: &FileError) -> Result<MultiValue> {
    fail(lua, error.to_string(), error.code())
}

enum Format {
    Number,
    All,
    Line(bool),
    Count(usize),
}

fn format_of(value: &Value) -> Result<Format> {
    match value {
        Value::Integer(n) => Ok(Format::Count(usize::try_from(*n).unwrap_or(0))),
        Value::Number(n) => Ok(Format::Count(opsis_types::numbers::truncate::<usize>(
            n.max(0.0),
        ))),
        Value::String(text) => {
            let text = text.to_str()?;
            match text.trim_start_matches('*').chars().next() {
                Some('n') => Ok(Format::Number),
                Some('a') => Ok(Format::All),
                Some('l') => Ok(Format::Line(false)),
                Some('L') => Ok(Format::Line(true)),
                _ => Err(crate::failed("bad argument to 'read' (invalid format)")),
            }
        }
        _ => Err(crate::failed("bad argument to 'read' (invalid format)")),
    }
}

fn number_value(lua: &Lua, text: &str) -> Result<Value> {
    if let Ok(whole) = text.parse::<i64>() {
        return Ok(Value::Integer(whole));
    }
    if text.is_empty() {
        return Ok(Value::Nil);
    }
    Ok(lua
        .coerce_number(Value::String(lua.create_string(text)?))?
        .map_or(Value::Nil, Value::Number))
}

impl LuaFile {
    fn disk(&mut self) -> Result<&mut DiskFile> {
        match self.target.as_mut() {
            Some(Target::Disk(file)) => Ok(file),
            Some(Target::Stream(_)) => Err(crate::failed("Scripts have no input to read from.")),
            None => Err(crate::failed("attempt to use a closed file")),
        }
    }

    fn read(&mut self, lua: &Lua, formats: &[Value]) -> Result<MultiValue> {
        let default = [Value::String(lua.create_string("l")?)];
        let formats = if formats.is_empty() {
            &default[..]
        } else {
            formats
        };
        let mut values = MultiValue::new();
        for format in formats {
            let format = format_of(format)?;
            let file = self.disk()?;
            let value = match format {
                Format::Number => {
                    let text = file.number_text().map_err(|e| io_failed(&e))?;
                    number_value(lua, &text)?
                }
                Format::All => {
                    Value::String(lua.create_string(file.all().map_err(|e| io_failed(&e))?)?)
                }
                Format::Line(keep) => match file.line(keep).map_err(|e| io_failed(&e))? {
                    Some(line) => Value::String(lua.create_string(line)?),
                    None => Value::Nil,
                },
                Format::Count(count) => match file.count(count).map_err(|e| io_failed(&e))? {
                    Some(bytes) => Value::String(lua.create_string(bytes)?),
                    None => Value::Nil,
                },
            };
            let ended = value.is_nil();
            values.push_back(value);
            if ended {
                break;
            }
        }
        Ok(values)
    }

    fn write(&mut self, lua: &Lua, parts: &[Value]) -> Result<()> {
        let mut text = Vec::new();
        for part in parts {
            match part {
                Value::String(s) => text.extend_from_slice(&s.as_bytes()),
                Value::Integer(_) | Value::Number(_) => {
                    let shown = lua.coerce_string(part.clone())?.ok_or_else(|| {
                        crate::failed("bad argument to 'write' (string expected)")
                    })?;
                    text.extend_from_slice(&shown.as_bytes());
                }
                other => {
                    return Err(crate::failed(format!(
                        "bad argument to 'write' (string expected, got {})",
                        other.type_name()
                    )));
                }
            }
        }
        match self.target.as_mut() {
            Some(Target::Disk(file)) => {
                file.drop_buffer().map_err(|e| io_failed(&e))?;
                file.file.write_all(&text).map_err(|e| io_failed(&e))
            }
            Some(Target::Stream(stream)) => {
                self.streams.write(*stream, &String::from_utf8_lossy(&text));
                Ok(())
            }
            None => Err(crate::failed("attempt to use a closed file")),
        }
    }

    fn close(&mut self) -> std::result::Result<(), String> {
        match self.target.take() {
            Some(Target::Disk(mut file)) => file.file.flush().map_err(|e| e.to_string()),
            Some(Target::Stream(stream)) => {
                self.target = Some(Target::Stream(stream));
                Err("cannot close standard file".into())
            }
            None => Err("attempt to use a closed file".into()),
        }
    }
}

fn lines_iterator(
    lua: &Lua,
    file: AnyUserData,
    formats: Vec<Value>,
    close_at_end: bool,
) -> Result<Function> {
    lua.create_function(move |lua, ()| {
        let mut handle = file.borrow_mut::<LuaFile>()?;
        if handle.target.is_none() {
            return Err(crate::failed("file is already closed"));
        }
        let values = handle.read(lua, &formats)?;
        if close_at_end && values.front().is_none_or(Value::is_nil) {
            drop(handle.close());
        }
        Ok(values)
    })
}

impl UserData for LuaFile {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("read", |lua, this, formats: Variadic<Value>| {
            this.read(lua, &formats)
        });
        methods.add_function(
            "write",
            |lua, (file, parts): (AnyUserData, Variadic<Value>)| {
                file.borrow_mut::<Self>()?.write(lua, &parts)?;
                Ok(file)
            },
        );
        methods.add_function(
            "lines",
            |lua, (file, formats): (AnyUserData, Variadic<Value>)| {
                lines_iterator(lua, file, formats.to_vec(), false)
            },
        );
        methods.add_method_mut("close", |lua, this, ()| match this.close() {
            Ok(()) => Ok(MultiValue::from_iter([Value::Boolean(true)])),
            Err(message) => fail(lua, message, 0),
        });
        methods.add_method_mut("flush", |_, this, ()| {
            match this.target.as_mut() {
                Some(Target::Disk(file)) => file.file.flush().map_err(|e| io_failed(&e))?,
                Some(Target::Stream(stream)) => this.streams.flush(*stream),
                None => return Err(crate::failed("attempt to use a closed file")),
            }
            Ok(true)
        });
        methods.add_method_mut(
            "seek",
            |lua, this, (whence, offset): (Option<String>, Option<i64>)| {
                let offset = offset.unwrap_or(0);
                let to = match whence.as_deref().unwrap_or("cur") {
                    "set" => SeekFrom::Start(u64::try_from(offset).unwrap_or(0)),
                    "cur" => SeekFrom::Current(offset),
                    "end" => SeekFrom::End(offset),
                    _ => {
                        return Err(crate::failed("bad argument to 'seek' (invalid option)"));
                    }
                };
                let file = this.disk()?;
                let at = file.drop_buffer().and_then(|()| file.file.seek(to));
                match at {
                    Ok(at) => Ok(MultiValue::from_iter([Value::Integer(
                        i64::try_from(at).unwrap_or(i64::MAX),
                    )])),
                    Err(error) => fail(lua, error.to_string(), 0),
                }
            },
        );
        methods.add_method("setvbuf", |_, _, _: Variadic<Value>| Ok(true));
        methods.add_meta_method("__tostring", |_, this, ()| {
            Ok(if this.target.is_some() {
                "file"
            } else {
                "file (closed)"
            })
        });
    }
}

fn file_value(lua: &Lua, streams: &Rc<Streams>, target: Target) -> Result<AnyUserData> {
    lua.create_userdata(LuaFile {
        target: Some(target),
        streams: Rc::clone(streams),
    })
}

fn open(
    lua: &Lua,
    folder: &Folder,
    streams: &Rc<Streams>,
    path: &str,
    mode: &str,
) -> Result<std::result::Result<AnyUserData, MultiValue>> {
    let mode =
        Mode::parse(mode).ok_or_else(|| crate::failed("bad argument to 'open' (invalid mode)"))?;
    Ok(match folder.open_file(path, mode) {
        Ok(file) => Ok(file_value(
            lua,
            streams,
            Target::Disk(DiskFile {
                file,
                buffer: Vec::new(),
                at: 0,
            }),
        )?),
        Err(error) => Err(file_failure(lua, &error)?),
    })
}

fn default_file(lua: &Lua, key: &str) -> Result<AnyUserData> {
    match lua.named_registry_value::<Value>(key)? {
        Value::UserData(file) => Ok(file),
        _ => Err(crate::failed(
            "Scripts have no input to read from; open a file with io.open or io.input.",
        )),
    }
}

fn choose_default(
    lua: &Lua,
    folder: &Rc<Folder>,
    streams: &Rc<Streams>,
    key: &str,
    mode: &str,
    given: Value,
) -> Result<Value> {
    match given {
        Value::Nil => lua.named_registry_value::<Value>(key),
        Value::String(path) => match open(lua, folder, streams, &path.to_str()?, mode)? {
            Ok(file) => {
                lua.set_named_registry_value(key, &file)?;
                Ok(Value::UserData(file))
            }
            Err(failed) => Err(crate::failed(
                failed
                    .get(1)
                    .and_then(|v| v.as_string().map(mlua::LuaString::to_string_lossy))
                    .unwrap_or_default(),
            )),
        },
        Value::UserData(file) if file.is::<LuaFile>() => {
            lua.set_named_registry_value(key, &file)?;
            Ok(Value::UserData(file))
        }
        _ => Err(crate::failed("bad argument (file expected)")),
    }
}

pub fn install(lua: &Lua, folder: &Rc<Folder>, streams: &Rc<Streams>) -> Result<Table> {
    let io = lua.create_table()?;
    let stdout = file_value(lua, streams, Target::Stream(Stream::Out))?;
    let stderr = file_value(lua, streams, Target::Stream(Stream::Err))?;
    io.set("stdout", &stdout)?;
    io.set("stderr", &stderr)?;
    lua.set_named_registry_value(OUTPUT_KEY, &stdout)?;
    lua.set_named_registry_value(INPUT_KEY, Value::Nil)?;

    let (f, s) = (Rc::clone(folder), Rc::clone(streams));
    io.set(
        "open",
        lua.create_function(move |lua, (path, mode): (String, Option<String>)| {
            Ok(
                match open(lua, &f, &s, &path, mode.as_deref().unwrap_or("r"))? {
                    Ok(file) => MultiValue::from_iter([Value::UserData(file)]),
                    Err(failed) => failed,
                },
            )
        })?,
    )?;

    let (f, s) = (Rc::clone(folder), Rc::clone(streams));
    io.set(
        "lines",
        lua.create_function(
            move |lua, (path, formats): (Option<String>, Variadic<Value>)| match path {
                None => lines_iterator(lua, default_file(lua, INPUT_KEY)?, formats.to_vec(), false),
                Some(path) => match open(lua, &f, &s, &path, "r")? {
                    Ok(file) => lines_iterator(lua, file, formats.to_vec(), true),
                    Err(failed) => Err(crate::failed(
                        failed
                            .get(1)
                            .and_then(|v| v.as_string().map(mlua::LuaString::to_string_lossy))
                            .unwrap_or_default(),
                    )),
                },
            },
        )?,
    )?;

    io.set(
        "read",
        lua.create_function(|lua, formats: Variadic<Value>| {
            default_file(lua, INPUT_KEY)?
                .borrow_mut::<LuaFile>()?
                .read(lua, &formats)
        })?,
    )?;

    io.set(
        "write",
        lua.create_function(|lua, parts: Variadic<Value>| {
            let file = default_file(lua, OUTPUT_KEY)?;
            file.borrow_mut::<LuaFile>()?.write(lua, &parts)?;
            Ok(file)
        })?,
    )?;

    io.set(
        "flush",
        lua.create_function(|lua, ()| {
            default_file(lua, OUTPUT_KEY)?.call_method::<Value>("flush", ())
        })?,
    )?;

    io.set(
        "close",
        lua.create_function(|lua, file: Option<AnyUserData>| {
            let file = match file {
                Some(file) => file,
                None => default_file(lua, OUTPUT_KEY)?,
            };
            file.call_method::<MultiValue>("close", ())
        })?,
    )?;

    let (f, s) = (Rc::clone(folder), Rc::clone(streams));
    io.set(
        "input",
        lua.create_function(move |lua, given: Value| {
            choose_default(lua, &f, &s, INPUT_KEY, "r", given)
        })?,
    )?;

    let (f, s) = (Rc::clone(folder), Rc::clone(streams));
    io.set(
        "output",
        lua.create_function(move |lua, given: Value| {
            choose_default(lua, &f, &s, OUTPUT_KEY, "w", given)
        })?,
    )?;

    io.set(
        "type",
        lua.create_function(|_, value: Value| {
            Ok(match value {
                Value::UserData(file) => match file.borrow::<LuaFile>() {
                    Ok(file) if file.target.is_some() => Some("file"),
                    Ok(_) => Some("closed file"),
                    Err(_) => None,
                },
                _ => None,
            })
        })?,
    )?;

    Ok(io)
}
