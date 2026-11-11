mod support;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use opsis_types::script::ScriptLineKind;
use support::{FakeHost, run_in, run_with};

struct Place {
    _temp: tempfile::TempDir,
    root: PathBuf,
    outside: PathBuf,
}

fn place() -> Place {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    let outside = temp.path().join("outside.txt");
    std::fs::write(&outside, "outside").unwrap();
    for dir in ["notes", "out", "scripts", ".opsis/cache", ".opsis/out"] {
        std::fs::create_dir_all(root.join(dir)).unwrap();
    }
    std::fs::write(root.join("notes/a.txt"), "first\nsecond\n").unwrap();
    std::fs::write(root.join("project.opsis.json"), "{}").unwrap();
    std::fs::write(root.join(".opsis/cache/c.txt"), "cache").unwrap();
    std::fs::write(
        root.join("scripts/helper.lua"),
        "counter = (counter or 0) + 1\nreturn { twice = function(n) return n * 2 end }\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink("/etc", root.join("etc")).unwrap();
        symlink("notes", root.join("notes-link")).unwrap();
        symlink(&outside, root.join("outside-link")).unwrap();
        symlink(".opsis/cache", root.join("cache-link")).unwrap();
        symlink(root.join("notes"), root.join("absolute-link")).unwrap();
    }
    Place {
        _temp: temp,
        root,
        outside,
    }
}

fn ok(root: &Path, source: &str) -> Vec<String> {
    let ran = run_in(root, source);
    assert!(
        ran.ending.failure.is_none(),
        "{:?}\n{:?}",
        ran.ending.failure,
        ran.lines
    );
    ran.lines.into_iter().map(|(_, text)| text).collect()
}

#[test]
fn reads_inside_the_project_by_every_name_for_it() {
    let p = place();
    let absolute = p.root.join("notes/a.txt");
    ok(
        &p.root,
        &format!(
            r#"
assert(io.open("notes/a.txt"):read("a") == "first\nsecond\n")
assert(io.open({absolute:?}):read("l") == "first")
assert(io.open("notes/../notes/a.txt"):read("L") == "first\n")
assert(io.open("project:notes/a.txt"):read(5) == "first")
local lines = {{}}
for line in io.lines("notes/a.txt") do lines[#lines + 1] = line end
assert(#lines == 2 and lines[2] == "second")
"#
        ),
    );
}

#[cfg(unix)]
#[test]
fn reads_through_a_relative_link_that_stays_inside_and_refuses_an_absolute_one() {
    let p = place();
    ok(
        &p.root,
        r#"
assert(io.open("notes-link/a.txt"):read("l") == "first")
assert(io.open("absolute-link/a.txt") == nil)
"#,
    );
}

#[test]
fn writes_renames_and_removes_inside_the_project() {
    let p = place();
    ok(
        &p.root,
        r#"
local f = assert(io.open("out/new.txt", "w"))
f:write("one ", 2, "\n")
f:close()
assert(os.rename("out/new.txt", "out/moved.txt"))
assert(io.open("out/moved.txt"):read("a") == "one 2\n")
assert(os.remove("out/moved.txt"))
assert(io.open("out/moved.txt") == nil)
local film = assert(io.open(".opsis/out/film.txt", "w"))
film:write("ok")
film:close()
"#,
    );
    assert!(!p.root.join("out/moved.txt").exists());
    assert_eq!(
        std::fs::read_to_string(p.root.join(".opsis/out/film.txt")).unwrap(),
        "ok"
    );
}

#[test]
fn refuses_every_way_out_of_the_project() {
    let p = place();
    let outside = p.outside.to_string_lossy().into_owned();
    ok(
        &p.root,
        &format!(
            r#"
for _, path in ipairs({{ "../outside.txt", "notes/../../outside.txt", "/etc/hostname", {outside:?} }}) do
  local f, message = io.open(path)
  assert(f == nil and message:find("outside"), path .. ": " .. tostring(message))
end
assert(io.open("../created.txt", "w") == nil)
assert(os.remove("../outside.txt") == nil)
assert(os.rename("notes/a.txt", "../moved.txt") == nil)
"#
        ),
    );
    assert_eq!(std::fs::read_to_string(&p.outside).unwrap(), "outside");
    assert!(!p.root.parent().unwrap().join("created.txt").exists());
    assert!(!p.root.parent().unwrap().join("moved.txt").exists());
    assert!(p.root.join("notes/a.txt").exists());
}

#[cfg(unix)]
#[test]
fn refuses_links_that_lead_outside_or_into_the_meta_folder() {
    let p = place();
    ok(
        &p.root,
        r#"
assert(io.open("etc/hostname") == nil)
assert(io.open("etc/created", "w") == nil)
assert(io.open("outside-link") == nil)
assert(io.open("cache-link/c.txt"):read("a") == "cache")
local f, message = io.open("cache-link/c.txt", "w")
assert(f == nil and message:find("can't change"), tostring(message))
assert(io.open("cache-link/new.txt", "w") == nil)
"#,
    );
    assert_eq!(
        std::fs::read_to_string(p.root.join(".opsis/cache/c.txt")).unwrap(),
        "cache"
    );
    assert!(!p.root.join(".opsis/cache/new.txt").exists());
}

#[test]
fn keeps_the_project_file_and_the_meta_folder_read_only() {
    let p = place();
    ok(
        &p.root,
        r#"
assert(io.open("project.opsis.json"):read("a") == "{}")
assert(io.open("project.opsis.json", "w") == nil)
assert(io.open("project.opsis.json", "a") == nil)
assert(io.open(".opsis/cache/c.txt", "r+") == nil)
assert(io.open(".opsis/new.txt", "w") == nil)
assert(os.remove("project.opsis.json") == nil)
assert(os.rename(".opsis/cache/c.txt", "out/c.txt") == nil)
assert(os.rename("notes/a.txt", ".opsis/cache/a.txt") == nil)
"#,
    );
    assert_eq!(
        std::fs::read_to_string(p.root.join("project.opsis.json")).unwrap(),
        "{}"
    );
}

#[test]
fn leaves_out_the_doors_to_the_computer() {
    let p = place();
    ok(
        &p.root,
        r#"
assert(os.execute == nil and os.getenv == nil and os.setlocale == nil and os.tmpname == nil)
assert(io.popen == nil and io.tmpfile == nil)
assert(package == nil and debug == nil and string.dump == nil)
assert(type(os.time()) == "number" and type(os.clock()) == "number" and os.date("%Y") ~= nil)
local f, message = load("\27Lua")
assert(f == nil and message:find("binary"), tostring(message))
assert(load("return 1 + 1")() == 2)
"#,
    );
}

#[test]
fn requires_project_scripts_once_and_loads_files_inside_only() {
    let p = place();
    ok(
        &p.root,
        r#"
local helper = require("helper")
assert(helper.twice(4) == 8)
assert(require("helper") == helper and counter == 1)
local ok, message = pcall(require, "missing")
assert(not ok and message:find("scripts/missing.lua"))
assert(dofile("scripts/helper.lua").twice(1) == 2 and counter == 2)
assert(loadfile("../outside.txt") == nil)
assert(not pcall(dofile, "/etc/hostname"))
"#,
    );
}

#[test]
fn sends_print_warn_and_written_text_to_the_output() {
    let p = place();
    let ran = run_in(
        &p.root,
        r#"
print("a", 1, nil, true)
warn("careful")
io.write("part ")
io.write("whole\nnext")
io.stderr:write("problem\n")
return { done = true }
"#,
    );
    assert_eq!(ran.ending.value, Some(serde_json::json!({ "done": true })));
    assert_eq!(
        ran.lines,
        vec![
            (ScriptLineKind::Print, "a\t1\tnil\ttrue".to_owned()),
            (ScriptLineKind::Warn, "careful".to_owned()),
            (ScriptLineKind::Print, "part whole".to_owned()),
            (ScriptLineKind::Warn, "problem".to_owned()),
            (ScriptLineKind::Print, "next".to_owned()),
        ]
    );
}

#[test]
fn names_the_line_a_script_failed_on() {
    let p = place();
    let ran = run_in(&p.root, "local x = 1\n\nerror(\"broken\")\n");
    let failure = ran.ending.failure.unwrap();
    assert_eq!(failure.line, Some(3));
    assert_eq!(failure.file, "scripts/test.lua");
    assert_eq!(failure.message, "scripts/test.lua:3: broken");
    assert_eq!(ran.lines.last().unwrap().0, ScriptLineKind::Error);
    let syntax = run_in(&p.root, "local x = \nif then")
        .ending
        .failure
        .unwrap();
    assert_eq!(syntax.line, Some(2));
    let host = run_in(&p.root, "\nopsis.nodes.remove(\"no such node\")")
        .ending
        .failure
        .unwrap();
    assert_eq!(host.line, Some(2), "{}", host.message);
    assert!(
        host.message.starts_with("scripts/test.lua:2: "),
        "{}",
        host.message
    );
}

#[test]
fn ends_on_os_exit_as_success_or_failure() {
    let p = place();
    let ran = run_in(&p.root, "print(1)\nos.exit(true)\nprint(2)");
    assert!(ran.ending.failure.is_none());
    assert_eq!(ran.lines.len(), 1);
    let ran = run_in(&p.root, "pcall(os.exit, 3)\nprint(2)");
    assert!(ran.ending.failure.is_some());
    assert!(ran.lines.iter().all(|(_, text)| text != "2"));
}

#[test]
fn passes_its_memory_limit_as_an_error() {
    let p = place();
    let ran = run_in(
        &p.root,
        "local t = {}\nfor i = 1, 1e9 do t[i] = string.rep('x', 1024) .. i end",
    );
    let failure = ran.ending.failure.unwrap();
    assert!(failure.message.contains("memory"), "{}", failure.message);
}

fn stops_soon(source: &str) {
    let p = place();
    let stop = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&stop);
    let started = Instant::now();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        flag.store(true, Ordering::Relaxed);
    });
    let ran = run_with(&p.root, FakeHost::new(), source, stop);
    assert!(ran.ending.stopped, "{source}: {:?}", ran.ending);
    assert!(started.elapsed() < Duration::from_secs(2), "{source}");
}

#[test]
fn stops_a_loop_a_coroutine_a_caught_loop_and_a_sleep() {
    stops_soon("while true do end");
    stops_soon("coroutine.wrap(function() while true do end end)()");
    stops_soon("while true do pcall(function() while true do end end) end");
    stops_soon("opsis.sleep(60)");
}
