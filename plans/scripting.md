# Scripting

Lua scripts that drive the open project: the graph, the timeline, the mixer, runs and exports, with file access held to the project folder. A Script panel in the editor writes and runs them, and the Local API gets a route that runs one. Lua as a language for node packs is a later part that waits on [node-packs.md](node-packs.md).

Status: plan only. Nothing here is built yet, except the copy rule in `AGENTS.md` and `CONTRIBUTING.md`, which already allows naming Lua. Needs nothing from the other plans, except the last part. Facts were checked on 11 November 2026, on x86_64 Linux with rustc 1.101.0-nightly (2026-10-28).

## How to read this plan

- **Runtime**, **Sandbox**, **Document batch**, **Script API** and **Running scripts** make the scripting layer. They build in that order.
- **Endpoint** adds the Local API route. It needs Running scripts.
- **Script panel** and **Editor behaviour** make the editor's view. They need Running scripts.
- **Rules, docs and notices** closes the work.
- **Later: Lua as a pack language** is findings only, kept so they are not derived again.
- Each part has the same sections: **What it changes**, **Fit with Opsis** (In place / Missing), **Findings** where something was measured or read from source, **Work**, **Decisions**, **Risks**, **Sources**.
- **Findings were measured with two throwaway probes** built outside the repository and deleted afterwards. Their method is in [Appendix: how the findings were measured](#appendix-how-the-findings-were-measured), so a result can be reproduced.

## Conventions

These restate rules from `AGENTS.md` and `agent-docs/`; when they disagree with this plan, they win. When a part ships, its rules move into a new `agent-docs/scripting.md`, with pointers from `api.md`, `document.md`, `editor.md` and `ui.md`.

- **No comments** in Rust, C++, or any `.lua` file in the repository (test scripts included).
- **`unsafe_code` stays denied** in the new crate; `mlua`, `cap-std` and `tree-sitter` keep their `unsafe` inside themselves.
- **Every new dependency's pin goes in the root `Cargo.toml` by hand, then `cargo add -p`**, at its loose major.
- **Rust owns the logic and the state; C++ draws and forwards input** ([ui.md](../agent-docs/ui.md)). Highlighting, indentation and bracket rules are pure Rust functions with property tests.
- **Nothing uses the internet unless the user asks**, so no script call reaches the network.
- **User-facing copy follows AGENTS.md**: the panel is "Script"; Lua is named where a person needs to know the language.
- **Checks when a part is done:** `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`.

## Decided

- **The language is PUC-Rio Lua 5.5, linked statically.** Not LuaJIT, not Luau.
- **A script gets Lua's standard library and the `opsis` module, nothing else.** No C modules, no package manager; `require` loads only the project's own `.lua` files.
- **Scripts run only on an open project** and change its document directly, as the editor's own views do.
- **The script surface is wider than the Local API**: nodes, timeline, mixer, runs and exports, everything the editor does with the open project.
- **A script reaches only the files of the open project.** Lua's `os` and `io` are constrained to it.
- **Importing a file from outside the project is refused from scripts.**
- **An absolute path in a node's file setting is refused when a script sets it.**
- **The project file and `.opsis/` are read-only to scripts, except `.opsis/out`.**
- **Opening or creating a project, the app's own settings and anything that uses the network are left out of the first version.**
- **The Local API gets a route that runs Lua, off by default behind its own switch.**
- **Scripts are `.lua` files in a `scripts/` folder inside the project.** The project file format does not change.
- **A script that fails or is stopped rolls the document back automatically.** Files it wrote and runs it started remain.
- **The script editor is a tab in the bottom dock and pops out like the other tabs.**
- **Highlighting uses tree-sitter through its Rust crates**, not its C compiled into the C++ view.
- **The app names Lua where a person needs it.** The rule in `AGENTS.md` (User-facing copy) and its line in `CONTRIBUTING.md` already say so.

## Summary

| Part                        | What it changes                                                             | Needs                             |
| --------------------------- | --------------------------------------------------------------------------- | --------------------------------- |
| Runtime                     | `opsis-script`: Lua 5.5 through `mlua`, values across serde, limits, errors | —                                 |
| Sandbox                     | the constrained standard library, project-confined files, `require`         | Runtime                           |
| Document batch              | one undo step or a full rollback for everything a script edits              | —                                 |
| Script API                  | the `opsis` module over a `ScriptHost` trait                                | Runtime, Document batch           |
| Running scripts             | the session's script run: thread, stop, output, events, tasks               | Script API, Sandbox               |
| Endpoint                    | `settings.api.scripts`, the script routes, OpenAPI, the Windows pipe        | Running scripts                   |
| Script panel                | the Script dock, its view, files, output                                    | Running scripts                   |
| Editor behaviour            | highlighting, error marks, indentation, brackets                            | Script panel                      |
| Rules, docs and notices     | `AGENTS.md`, `agent-docs/scripting.md`, notices, user docs                  | every part above                  |
| Later: Lua as pack language | findings only                                                               | node-packs.md's Contract and Host |

## Runtime

### What it changes

- **A new crate, `crates/script` (`opsis-script`)**, holds the Lua runtime, the sandbox, the `opsis` module and the editor's text logic. It does not depend on `opsis-session`.
  - It defines a `ScriptHost` trait and the session implements it, the way `opsis-api` defines `ApiHost` and `session/src/api.rs` implements it (`AppApi`). That keeps the dependency one way: the session needs to start scripts for the endpoint, and scripts need the session's calls.
- **Lua is `mlua` 0.12 with the features `lua55`, `vendored` and `serde`.** `vendored` compiles Lua's C sources into the binary, so nothing ships beside the program and the packager stages nothing.
  - Not `async` and not `send`: a script runs on its own thread and blocks there (Running scripts), and the Lua state never leaves that thread.
- **Each run gets a fresh Lua state**, made on the script's thread and dropped when the run ends. Nothing carries over between runs.
- **Rust values cross as Lua tables through serde** (`LuaSerdeExt::to_value` and `from_value`), so every type that already derives `Serialize` and `Deserialize` (`Workflow`, `WorkflowEdit`, `TimelineEdit`, `TimelineSpec`, `RunEvent`) needs no hand-written conversion.
- **A script is loaded as text only** (`Chunk::set_mode(ChunkMode::Text)`) under the name `@scripts/<file>.lua` (`Chunk::set_name`), so an error reads `scripts/<file>.lua:12: …`.
- **A memory limit is set on every state** (`Lua::set_memory_limit`); passing it raises `Error::MemoryError` in the script.
- **There is no time limit.** A script may wait for a render. It stops through Stop: a hook (`Lua::set_hook` with `HookTriggers::new().every_nth_instruction(n)`) reads a stop flag and returns an error, which ends the script.

### Fit with Opsis

- **In place:** the `ApiHost` pattern to copy (`crates/api/src/routes.rs`, `crates/session/src/api.rs`); a C compiler in every build, since Qt and the terminal engine need one.
- **Missing:** the crate, its row in the crate table of `AGENTS.md`, the root `Cargo.toml` pins.

### Findings

- **`mlua` 0.12.2 is MIT, needs Rust 1.88**, and its features include `lua55`, `vendored`, `serde`, `async`, `send` (`cargo info mlua`). Its `mlua-sys` 0.13.0 takes `lua-src` 551.0.2, which carries Lua 5.5.1.
- **Lua 5.5.0 was released on 22 December 2025 and 5.5.1 on 3 August 2026** (lua.org).
- **Static Lua adds under 600 KiB.** One probe built with the repository's release profile (`opt-level = 3`, `lto = true`, `codegen-units = 1`, `strip = true`), `mlua` with `vendored`, `serde` and `async`, loading only `table`, `string`, `math` and `coroutine`:

| Build      | Program size    | Added to the 327,016-byte baseline | Build time |
| ---------- | --------------- | ---------------------------------- | ---------- |
| no Lua     | 327,016 bytes   | —                                  | 8 s        |
| Lua 5.4.9  | 905,296 bytes   | 565 KiB                            | 21 s       |
| Lua 5.5.1  | 915,200 bytes   | 574 KiB                            | 17 s       |
| LuaJIT 2.1 | 1,167,704 bytes | 821 KiB                            | —          |
| Luau 0.740 | 2,299,560 bytes | 1.88 MiB                           | 44 s       |

- **The plan drops `async`, so the 574 KiB is an upper figure** for what Lua 5.5 adds.
- **`set_memory_limit` was accepted on all four** and a serde value crossed into Lua and back on all four.
- **The Lua probe links `libm` dynamically**, so `math.sin` and the like use the computer's maths library. That is fine for scripts and is the reason native Lua is wrong for drawing (Later: Lua as a pack language).
- **Why not LuaJIT:** it is the Lua 5.1 dialect, it has rolling releases with no tarballs, and its hooks are not called from compiled code unless it is built with `LUAJIT_ENABLE_CHECKHOOK` (`lj_record.c`), which would break Stop. Its speed buys nothing when a script is mostly host calls.
- **Why not Luau:** a different dialect, C++, and the largest build.
- **Why not `piccolo`** (a Lua VM in Rust, 0.3.3): its README calls it experimental, with `io`, `os`, `package`, `string`, `table` and `utf8` missing or sparse.
- **`mlua`'s error type** names what the views need: `SyntaxError { message, incomplete_input }`, `RuntimeError`, `MemoryError`, `CallbackError { traceback, cause }` (`mlua-0.12.2/src/error.rs`).
- **`mlua`'s serde options** (`src/serde/ser.rs`, `src/serde/de.rs`) include `serialize_none_to_null`, `set_array_metatable`, `deny_unsupported_types`, `deny_recursive_tables`, `sort_keys` and `encode_empty_tables_as_array`.

### Work

1. Add `mlua`, pinned at `"0.12"` in the root table with `lua55`, `vendored` and `serde`, and the crate `opsis-script` with its error enum (`ScriptError`, `thiserror`).
2. Make the state: the libraries of **Sandbox**, the memory limit, the stop hook, `print` and `warn` sent to the run's output.
3. Turn an `mlua::Error` into a `ScriptFailure { message, line }`, reading the line from the `scripts/<file>.lua:<line>:` prefix.
4. Property tests: any JSON value a node's settings can hold goes into Lua and comes back equal; any `WorkflowEdit` does the same.

### Decisions

- **The memory limit**, set by measuring a script that reads a whole compiled timeline of the largest example.
- **The instruction count between stop checks**, set by measuring how long a tight loop takes to stop.

### Risks

- **An empty Lua table is neither a list nor a map.** Where Rust reads into `serde_json::Value` (a node's `params`), `{}` becomes a map unless `encode_empty_tables_as_array` is set, and then an empty map becomes a list. Lists that come from Rust keep `mlua`'s array metatable; a script that needs a new empty list gets `opsis.list()`, which carries it.
- **Lua 5.5 has integers and floats.** `fps = 60.0` is a float, and a `u32` setting refuses it; the settings check already answers with the field's problem, and the script error must carry it.
- **`lua-src` 551.1.0 exists** while `mlua-sys` 0.13.0 takes 551.0.2; the Lua version moves when `mlua-sys` does.

### Sources

- `mlua` 0.12.2 (https://github.com/mlua-rs/mlua): `src/state.rs` (`new_with`, `set_memory_limit`, `set_hook`), `src/chunk.rs`, `src/error.rs`, `src/serde/`, `README.md`.
- Lua versions (https://www.lua.org/versions.html), the Lua 5.5 manual (https://www.lua.org/manual/5.5/manual.html).
- LuaJIT status, install and extensions (https://luajit.org/status.html, https://luajit.org/install.html, https://luajit.org/extensions.html); `luajit-src` 210.7.4 `luajit2/src/lj_record.c`.
- Luau sandbox (https://luau.org/sandbox); `piccolo` (https://github.com/kyren/piccolo).

## Sandbox

### What it changes

- **Only these libraries are loaded** (`Lua::new_with`): basic, `coroutine`, `table`, `string`, `utf8`, `math` and `os`. `io` is written in Rust, `package` and `debug` are never loaded.

| Library     | Treatment                                                                                                   |
| ----------- | ----------------------------------------------------------------------------------------------------------- |
| basic       | loaded; `dofile` and `loadfile` replaced by project-confined ones; `load` forced to text; `print` to output |
| `coroutine` | loaded as it is                                                                                             |
| `table`     | loaded as it is                                                                                             |
| `string`    | loaded; `string.dump` removed                                                                               |
| `utf8`      | loaded as it is                                                                                             |
| `math`      | loaded as it is                                                                                             |
| `os`        | loaded, then changed in place (table below)                                                                 |
| `io`        | not loaded; an `io` table written in Rust over the project folder                                           |
| `package`   | not loaded; a `require` written in Rust that reads `scripts/` through the same confined folder              |
| `debug`     | not loaded (`mlua` refuses it in a safe state)                                                              |

- **`os` is changed in place, on the table Lua made**, never by setting a new global:

| Function                            | Treatment                                                                                      |
| ----------------------------------- | ---------------------------------------------------------------------------------------------- |
| `clock`, `date`, `difftime`, `time` | kept as they are                                                                               |
| `remove`, `rename`                  | replaced by project-confined versions                                                          |
| `exit`                              | replaced: ends the script, as success for `true`, `0` or nothing and as a failure for the rest |
| `execute`                           | removed: it runs a shell                                                                       |
| `getenv`                            | removed: it reads the environment, secrets included                                            |
| `setlocale`                         | removed: it changes the C locale of the whole app                                              |
| `tmpname`                           | removed: it names a file in the system temp folder                                             |

- **`io` has `open`, `lines`, `read`, `write`, `close`, `input`, `output`, `type`, `stdout` and `stderr`**, and a file has `read`, `write`, `lines`, `seek`, `flush`, `close` and `setvbuf`. `io.write` and the two streams go to the run's output; there is no input stream. `io.popen` and `io.tmpfile` are absent.
- **Every path resolves beneath one handle on the project folder** (`cap_std::fs::Dir`, opened once per run from the open document's folder), never by comparing path text.
  - A relative path is relative to the project folder, not the process's working folder.
  - An absolute path is accepted only when it lies in the project folder, and is made relative first.
  - A `project:` ref is accepted as the path it names.
- **Reading is allowed anywhere in the project. Writing, removing and renaming are refused for `project.opsis.json` and for `.opsis/`, except `.opsis/out`.**
  - The check runs on the resolved path (`Dir::canonicalize`), so a link inside the project that points into `.opsis/` does not get around it.
- **`require "name"` reads `scripts/name.lua`, then `scripts/name/init.lua`**, as text, once per run.

### Fit with Opsis

- **In place:** the folder names (`crates/types/src/project.rs`: `META_DIR` `.opsis`, `OUT_DIR` `.opsis/out`, `CACHE_DIR`, `ASSETS_DIR`), the project file name (`WORKFLOW_FILE`, `crates/types/src/workflow.rs`), ref reading (`ref_target`, `crates/compile/src/asset_ref.rs`), `Apache-2.0 WITH LLVM-exception` and `MIT` in `deny.toml`.
- **Missing:** `cap-std` in the workspace, the Rust `io`, the `os` changes, the confined `require`, `dofile` and `loadfile`.

### Findings

- **`mlua`'s safe state guards memory safety only.** In `Lua::new()`, a script in the probe:
  - read `/etc/hostname` through `io.open`;
  - opened it through `loadfile` and `dofile`, whose syntax error quotes part of the file;
  - ran a command through `os.execute` and another through `io.popen`;
  - read `HOME` through `os.getenv`;
  - ran bytecode through `load(string.dump(f))`;
  - found a file through `package.searchpath("hostname", "/etc/?")`.
- **What the safe state does block:** `debug` and LuaJIT's `ffi` cannot be loaded, `package.loadlib` raises an error, and the C module searchers are replaced (`disable_c_modules`, `src/state.rs`).
- **Replacing the `os` global is not enough when `package` is loaded**: `package.loaded.os.execute` was still a function. After the in-place change it was `nil`. This plan never loads `package`, and still changes `os` in place.
- **With `package` loaded, `package.searchpath` still found `/etc/hostname`** after `os` and `io` were constrained, and `require` runs any `.lua` on disk through `package.path`. Hence no `package`.
- **Lua 5.5.1's `os` has eleven functions**: `clock`, `date`, `difftime`, `execute`, `exit`, `getenv`, `remove`, `rename`, `setlocale`, `time`, `tmpname` (`loslib.c`). Its `io` has `close`, `flush`, `input`, `lines`, `open`, `output`, `popen`, `read`, `tmpfile`, `type`, `write` (`liolib.c`). The basic library has `dofile`, `loadfile` and `load` (`lbaselib.c`).
- **Confinement by a folder handle held against every escape tried.** With `io.open`, `os.remove` and `os.rename` written over `cap_std::fs::Dir`:

| Attempt                                                                   | Result  |
| ------------------------------------------------------------------------- | ------- |
| read `notes/a.txt`                                                        | allowed |
| read it by its absolute path inside the project                           | allowed |
| read through a link that stays inside the project                         | allowed |
| read `notes/../notes/a.txt`                                               | allowed |
| write `out/new.txt`, rename it, remove it                                 | allowed |
| read `../outside.txt` and `notes/../../outside.txt`                       | refused |
| read `/etc/hostname`                                                      | refused |
| read and write through a link to `/etc`                                   | refused |
| read a link to a file outside                                             | refused |
| write `../created.txt`                                                    | refused |
| `os.remove("../outside.txt")`, `os.rename("notes/a.txt", "../moved.txt")` | refused |

- **Afterwards the outside file was untouched and nothing had been created outside the project.** The refusals read "a path led outside of the filesystem".
- **`cap-std` 4.0.3 is `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT`.** On Linux it opens with `openat2` and `RESOLVE_BENEATH` (kernel 5.6 and later) and falls back otherwise (`cap-primitives-4.0.3/src/rustix/linux/fs/open_impl.rs`); it has a `windows` implementation. `Dir` has `open_with`, `canonicalize`, `metadata`, `read_dir`, `create_dir_all`, `remove_file` and `rename`.
- **The existing `is_inside` (`crates/project/src/paths.rs`) is lexical** (`path_clean` and `std::path::absolute`), so a link that points out of the project passes it. It is right for refs and wrong for this.
- **Only Linux was run.** macOS and Windows were not.

### Work

1. Add `cap-std`, pinned at `"4"`.
2. The project folder handle and the path policy as one pure function from a given path to "read", "write" or a refusal, with property tests: no accepted path resolves outside the root, and no accepted write resolves to the project file or under `.opsis/` outside `.opsis/out`.
3. The Rust `io` with its file type, the `os` changes, and the confined `require`, `dofile` and `loadfile`.
4. Tests with a real folder for every row of the table above, plus a link into `.opsis/cache` and a write to the project file.
5. Run those tests on macOS and Windows in CI.

### Decisions

- None.

### Risks

- **A hard link inside the project to a file outside it cannot be told apart from an ordinary file.** A script cannot make one; a person can.
- **Between resolving a path and opening it, a link can change.** Both steps go through the same folder handle, so the worst case is a write landing in a read-only part of the project, never outside it.
- **`os.date` uses the C locale and the computer's time zone**, as in any Lua.

### Sources

- `lua-src` 551.0.2 `lua-5.5.1/loslib.c`, `liolib.c`, `lbaselib.c`, `loadlib.c`.
- `mlua` 0.12.2 `src/state.rs` (`new`, `new_with`, `disable_c_modules`), `src/stdlib.rs`.
- `cap-std` 4.0.3 `src/fs/dir.rs`; `cap-primitives` 4.0.3 `src/rustix/linux/fs/open_impl.rs`.
- Aseprite's scripting page (https://www.aseprite.org/docs/scripting/): `os.execute` and `io.open` ask the user first there.

## Document batch

### What it changes

- **`opsis-document` gains a batch**: everything edited between its start and its end is one undo step when it is kept, and is undone completely when it is dropped.
  - `Document::begin_batch` keeps the workflow and the redo list as they stand.
  - While a batch is open, `commit` records no undo step, as during a gesture.
  - `Document::end_batch(keep)`: kept, it records the starting workflow as one undo step when the workflow changed; dropped, it puts the starting workflow and the redo list back and records nothing.
- **A batch is not a gesture.** Timeline edits in a batch build on the current workflow, and compiles run as usual between edits.
- **`undo` and `redo` are refused while a batch is open**, with the message that a script is running.

### Fit with Opsis

- **In place:** `Document` keeps whole workflows in `past` and `future` and a `gesture` (`crates/document/src/lib.rs`); `commit` skips `remember` inside a gesture and `end_gesture` remembers the start once; the session's `begin_gesture` and `end_gesture` (`crates/session/src/document.rs`).
- **Missing:** the batch in `opsis-document`, `begin_batch` and `end_batch` in the session, the refusal of `undo` and `redo`.

### Findings

- **A gesture cannot hold a script.** `Document::edit_timeline` starts from `gesture_start` while a gesture is open, so each timeline edit replaces the one before it; that is what makes a drag one edit. A script's second timeline edit would undo its first.
- **A gesture also holds compiles**: while one is open the keys worked out last are kept (`held`, `crates/session/src/node_results.rs`, [runs.md](../agent-docs/runs.md)), and timeline edits run against the film the gesture began with (`gesture_film`). A script needs a fresh compile after each graph edit.
- **`undo` and `redo` close an open gesture** (`end_open_gesture`); a batch refuses them instead, so a rollback always has its starting point.
- **`Document::undo` after a failed script would leave the failed edits in `future`**, where redo brings them back. Dropping a batch restores `future` as it was.

### Work

1. The batch in `Document`, with property tests over generated edit lists: dropped, the workflow, `past` and `future` equal what they were; kept, undo returns to the start in one step; a batch that changes nothing records nothing.
2. `begin_batch` and `end_batch` in `crates/session/src/document.rs`, announcing `DocumentChanged` and asking for a compile at the end.
3. The refusal of `undo` and `redo` (`DocumentError`), mapped to 409 in `document_failure` (`crates/session/src/api.rs`).
4. [document.md](../agent-docs/document.md): the batch beside the gesture.

### Decisions

- None.

### Risks

- **Edits a person makes by hand while a script runs belong to its batch**, and a failed script rolls them back too. The panel's running state says so.
- **Autosave and a run's save still write the project file during a batch** (`run::start` saves before it runs), so an app that is killed mid-script leaves the script's partial edits on disk with no undo step.

### Sources

- `crates/document/src/lib.rs`, `crates/session/src/document.rs`, [document.md](../agent-docs/document.md), [runs.md](../agent-docs/runs.md).

## Script API

### What it changes

- **One global table, `opsis`, built over a `ScriptHost` trait** that the session implements. The trait is small, because most changes are the existing serde edits: `WorkflowEdit` (which already carries `Mix`, `Markers` and `Keyframes`) and `TimelineEdit`.
- **Project data is read live and set directly**, as in Blender's `bpy.data`: `opsis.nodes[id].params.url = "…"` becomes one `WorkflowEdit::UpdateNode`. A handle to a node that is gone raises an error on use.
- **Everything the Local API can do has a Lua call**, except opening a project, undo and redo.

| Lua               | What it gives                                                                                 | Rust behind it                                                                                                |
| ----------------- | --------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| `opsis.project`   | name, folder, revision, saved state, problems, `save()`, project settings (size, rate, theme) | `document_state`, `save_project`, `ApiHost::problems`, `WorkflowEdit::SetSettings` and `Rename`               |
| `opsis.catalog`   | every node type with its ports, defaults and settings schema                                  | `compile::editor_catalog`, the data behind `/nodes` and `/nodes/{type}`                                       |
| `opsis.nodes`     | read, add, change, move, remove, group, ungroup, paste                                        | `WorkflowEdit::AddNode`, `UpdateNode`, `MoveNodes`, `RemoveNodes`, `RemoveParts`, `Group`, `Ungroup`, `Paste` |
| `opsis.edges`     | read, connect, disconnect                                                                     | `WorkflowEdit::AddEdge`, `RemoveEdges`; a refused connection is the error                                     |
| `opsis.variables` | read, set, rename                                                                             | `WorkflowEdit::SetVariables`, `RenameVariable`                                                                |
| `opsis.fonts`     | the library's families, use one in the project, set a role                                    | `fonts::list_fonts`, `use_font_in_project`, `WorkflowEdit::AddProjectFont`, `SetFontRole`                     |
| `opsis.timeline`  | the flat timeline and the spec as tables; the fifteen timeline edits; markers; keyframes      | `EditorCompile.flat` and `.levelled`, `edit_timeline`, `WorkflowEdit::Markers` and `Keyframes`                |
| `opsis.mixer`     | channels, buses and the master as tables; the seven mix edits                                 | `EditorCompile.mix` and `.plan`, `WorkflowEdit::Mix`                                                          |
| `opsis.run`       | start for chosen nodes or from a node, stop, state, log, events, wait, results, clear results | `run::start`, `cancel_run`, `run_state`, `clear_run_log`, `projects::clear_cache`, the run events             |
| `opsis.playback`  | seek, play, pause, rate; the playhead                                                         | `shared::control_playback` (`PlaybackCall`), `shared_state`                                                   |
| `opsis.selection` | read and set what is selected                                                                 | `shared::shared_state`, `share_selection`                                                                     |
| `opsis.compile`   | wait for the compile of the current revision; its issues and outdated nodes                   | `compile::latest_compile`                                                                                     |
| `opsis.validate`  | check a workflow table without applying it                                                    | `check_workflow` (`crates/compile/src/check.rs`)                                                              |
| `opsis.sleep`     | wait some seconds; Stop interrupts it                                                         | the script thread                                                                                             |
| `opsis.list`      | a new empty list (Runtime, Risks)                                                             | `mlua`'s array metatable                                                                                      |
| `opsis.version`   | the app's version                                                                             | `ApiHost::version`                                                                                            |

- **An export is a run of an output node**: `opsis.run.start { nodes = { render.id } }`, and the result names the file under `.opsis/out`, which `io` can read.
- **Run events are an iterator**: `for event in run:events() do … end` yields each `RunEvent` as a table until `run:done`.
- **Timeline reads and edits wait for the compile of the current revision first**, because `edit_timeline` works against the film of the last compile.
- **An edit a script makes is checked as the editor's are**, plus one check of its own: a file setting may not name an absolute path.
- **Not bound in the first version:** opening, creating and closing a project, importing files, the app's settings, engine setup, font downloads and removals, updates, sign-in and the streamed views, the terminal window, licences, undo and redo.

```lua
local shot = opsis.nodes.add { type = "…", params = { url = "https://example.com" } }
shot.label = "Landing page"
shot.params.width = 1280

local render = opsis.nodes.add { type = "…" }
opsis.edges.connect(shot, "video", render, "video")

local run = opsis.run.start { nodes = { render.id } }
for event in run:events() do
  if event.type == "node:error" then
    error(event.message)
  end
end

local file = io.open(run:result(render.id).file, "rb")
print(#file:read("a"))
file:close()
```

The node types, port names and field names in this sketch stand for the real ones; the call shapes are what the plan fixes.

### Fit with Opsis

- **In place:**
  - The edits: `WorkflowEdit` (20 kinds), `TimelineEdit` (15), `MixEdit` (7), `MarkerEdit` (3), `KeyframeEdit` (7), all in `crates/types/src/edits.rs`, all `Serialize` and `Deserialize`.
  - The session calls of the table above. The editor's own actions are thin wrappers over them: `edit_mix`, `edit_markers`, `run_node` and `import_files` in `crates/ui/src/ui.rs` each forward to one session call or one `WorkflowEdit`.
  - The compile result as data: `EditorCompile` (`crates/session/src/compile.rs`) with `flat`, `levelled: Option<TimelineSpec>`, `plan`, `mix`, `issues`, `outdated`.
  - The run events: `run:start`, `node:start`, `node:progress`, `node:log`, `node:done`, `node:error`, `node:skipped`, `run:done` (`crates/types/src/events.rs`).
  - The catalogue with settings schemas, already served by `/nodes` and `/nodes/{type}`.
- **Missing:** the `ScriptHost` trait and its session implementation, a call that waits for the compile of a given revision, the check on file settings, the proxies.

### Findings

- **The `Host` trait of the editor has 195 methods** (`crates/qt/src/host.rs`), and most of them are view plumbing (pointer, keys, frames, style, layout). The script surface is the session, not `Host`.
- **`opsis-session` has about 130 public functions in 20 files**; the ones the table names are in `document.rs`, `run.rs`, `shared.rs`, `compile.rs`, `fonts.rs` and `projects.rs`.
- **Playback sound lives in `opsis-ui`** (`Ui.audio`, `opsis_audio::output::Output`), not the session. `shared::control_playback` sends a `PlaybackCall` (`Seek`, `Play`, `Pause`, `Rate`) to the main window, so playback from a script needs an editor window.
- **A node setting may name an absolute path today**: `ref_target` answers `RefTarget::Absolute` and `ref_to_path` (`crates/project/src/paths.rs`) returns it. File settings are the ones whose schema carries `x-accept` or the `asset` widget ([inspector.md](../agent-docs/inspector.md)).
- **`import_assets` copies any outside path into the project** ([assets.md](../agent-docs/assets.md)), after which `io` could read it. It stays unbound.
- **The run events channel keeps 256 events** (`RUN_EVENTS_KEPT`, `crates/session/src/api.rs`); a reader that falls further behind loses some.
- **Blender's shape, for comparison:** `bpy.data` is the data, `bpy.ops` the commands, `bpy.app.handlers` plain lists of callbacks (`render_pre`, `render_post`, `render_complete`, `render_cancel`, `frame_change_post`), `bpy.app.timers` runs a function later. `bpy.msgbus` subscribes to single properties and misses changes made in the viewport and by animation, which is why this plan has one event stream and no property subscriptions.

### Work

1. `ScriptHost` in `opsis-script` and its implementation in `crates/session/src/script.rs`.
2. The proxies for nodes, edges, variables and settings, each write one edit; reading a node that is gone raises "This node is gone."
3. `opsis.timeline` and `opsis.mixer`, with the wait for the current revision's compile.
4. `opsis.run`, with events read from the session's run events and a value that says how many were missed when the script fell behind.
5. The file-setting check: an edit from a script whose `params` name an absolute path in a file setting is refused with the field's name.
6. Tests against a fake `ScriptHost`: every edit kind is reachable from Lua and arrives as the same value Rust would build.
7. A test that lists the Local API's routes and fails when one has no Lua call, apart from the ones left out by name.

### Decisions

- None.

### Risks

- **A script that falls 256 run events behind loses some**; `run:wait()` reads the run's state at the end, so the outcome is never lost.
- **Playback calls do nothing without an editor window**, for example from the endpoint with every window closed; the call answers that no window is playing.
- **The proxies make one edit per assignment.** A loop over a thousand nodes makes a thousand edits and a thousand compile requests; compiles already merge (one running, one waiting), and the batch keeps undo at one step.

### Sources

- `crates/types/src/edits.rs`, `crates/types/src/events.rs`, `crates/session/src/` (the files named above), `crates/qt/src/host.rs`, `crates/ui/src/ui.rs`.
- Blender's API pages: overview (https://docs.blender.org/api/current/info_overview.html), handlers (https://docs.blender.org/api/current/bpy.app.handlers.html), timers (https://docs.blender.org/api/current/bpy.app.timers.html), msgbus (https://docs.blender.org/api/current/bpy.msgbus.html). These were read as summaries, not word for word.
- FreeCAD scripting basics (https://wiki.freecad.org/FreeCAD_Scripting_Basics), from search summaries; the wiki refused the fetch.

## Running scripts

### What it changes

- **The session runs one script at a time, on its own thread.** A second start is refused with "A script is already running."
  - The thread is a plain one, not one of the async runtime's, and its host calls block on the runtime from there, as the editor's host does (`self.runtime.block_on`, `crates/ui/src/ui.rs`).
- **A run needs an open project**, and its file root is that project's folder for the whole run.
- **A run opens a document batch and closes it at the end**: kept when the script ends normally, dropped when it fails, is stopped, passes its memory limit, or the app or the project closes.
- **Stop sets the stop flag and wakes any wait** (a run's events, `opsis.sleep`, a compile).
- **Closing the project or opening another stops the running script first.**
- **A run is listed in `AppState.tasks`** ("Running a script") so the exit dialog names it.
- **Output is kept as lines** (`print`, `warn`, `io.write`, the error), capped like the run log, and sent to the windows as it arrives.
- **The outcome is one value**: whether it ended well, the value the script returned (as JSON), the output, the failure with its line, and whether the document was rolled back.

### Fit with Opsis

- **In place:** `AppState.tasks` (`begin`, `end`, `crates/session/src/state.rs`), the session's events (`session_events!`, `crates/session/src/events.rs`), the event pump to the views (`start_pump`, `event_kind`, `crates/ui/src/ui.rs`; `EventHub`), `runtime::spawn_blocking` (`crates/session/src/runtime.rs`).
- **Missing:** `crates/session/src/script.rs` (`run_script`, `stop_script`, `script_state`), two events (`ScriptChanged`, `ScriptLogged`) with their editor kinds, the stop in `document::open` and `adopt`.

### Findings

- **`mlua`'s `Lua` is not `Send` without its `send` feature**, so the state is made and dropped on the script's thread.
- **`Lua::set_hook` takes `Fn(&Lua, &Debug) -> Result<VmState>`**; returning an error raises it in the running script (`src/state.rs`).
- **Yazi, a Rust app with Lua plugins**, gives each asynchronous plugin its own isolated context and keeps one shared context only for start-up code.

### Work

1. `run_script(app, source)` where the source is a file under `scripts/` or text with a name, `stop_script`, `script_state`.
2. The two events, their `EventKind`s and `EventHub` signals.
3. The stop in `open`, `adopt` and the exit decision.
4. Tests: a script that fails leaves the document as it was; a stopped script does too; a second start is refused; a script that waits on run events stops within the stop check's interval.

### Decisions

- **How many output lines are kept**, the run log's cap unless measuring says otherwise.

### Risks

- **Never hold the editor lock across a call into Lua** ([AGENTS.md](../AGENTS.md) §7): each host call takes the lock, copies what it needs and lets go before Lua runs again.
- **A script that never yields to a host call still stops**, through the instruction hook, but only as often as the hook fires.

### Sources

- `crates/session/src/state.rs`, `events.rs`, `runtime.rs`, `document.rs`; [ui.md](../agent-docs/ui.md).
- Yazi's plugin overview (https://yazi-rs.github.io/docs/plugins/overview).

## Endpoint

### What it changes

- **A second switch, `settings.api.scripts`, off by default**, beside `settings.api.enabled` in `ApiSection` (`crates/types/src/settings.rs`). Settings → API shows it under the API switch and disables it while the API is off.
- **Three routes, all under `/project`:**
  - `POST /project/script` with `{ "source": "…" }` or `{ "file": "scripts/name.lua" }` runs a script and answers when it ends with the outcome and the editor's problems.
  - `POST /project/script/stop` stops it.
  - `GET /project/script` answers whether one runs.
- **Answers:** 403 while the scripts switch is off, 409 with no project or while a script runs, 400 for a body that is neither form, 200 with `ok: false` and the failure for a script that failed.
- **A script from the endpoint gets the same `opsis` module and the same file rules as one from the panel.**
- **On Windows the scripts switch stays unavailable until the named pipe opens only for the current user.**

### Fit with Opsis

- **In place:** the routes and `ApiHost` (`crates/api/src/routes.rs`), bodies (`bodies.rs`), `MAX_BODY`, the socket at mode 0600, the setting's follower (`follow_setting`, `crates/session/src/api.rs`), the API page (`SettingsWindow::ApiPage`, `crates/qt/cpp/settings_window.cc`), the OpenAPI document (`crates/api/src/openapi.rs`) and its route list in `documents_every_route_it_serves` (`crates/api/tests/routes.rs`), the `api` e2e scenario (`e2e_api.cc`).
- **Missing:** the setting, the three routes, three `ApiHost` methods, their OpenAPI entries, the pipe's access list on Windows.

### Findings

- **The Local API today can only make the edits `WorkflowEdit` allows.** With scripts on, the socket also reads and writes project files and drives runs.
- **On Unix the socket is the user's alone** (mode 0600), and that user can already run code.
- **On Windows the named pipe `\\.\pipe\opsis-api-<user>` has no current-user-only access list**, although `DESCRIPTION` in `openapi.rs` says only the user's account can open it ([api.md](../agent-docs/api.md), Deliberate limits). With scripts on, another account on the same computer could read and change the project's files.

### Work

1. The setting, its switch and its copy ("Run Lua scripts", with one line saying what a script can reach).
2. The pipe's access list on Windows, and `DESCRIPTION` made true; then the switch on Windows.
3. The routes, the `ApiHost` methods and `AppApi`'s implementations over `run_script`.
4. OpenAPI entries and the route list in the test; the document stays under 100 KB.
5. Route tests with a fake host, and the `api` scenario running a script that adds a node and one that fails.
6. [api.md](../agent-docs/api.md): the routes, the switch, the removed Windows limit.

### Decisions

- None.

### Risks

- **A long script holds its request open.** A client that goes away does not stop the script; `POST /project/script/stop` does.

### Sources

- [api.md](../agent-docs/api.md), `crates/api/src/routes.rs`, `crates/session/src/api.rs`, `crates/types/src/settings.rs`.

## Script panel

### What it changes

- **A fourth tab in the bottom dock, "Script", after Run**, made like the others (`MakeDock`, `addDockWidgetAsTab` in `MainWindow::BuildDocks`). The View menu gains "Script" on `CmdOrCtrl+4` beside Timeline, Mixer and Run.
- **It pops out and docks back like the other tabs**, with no code of its own: the bottom group's corner button does it (`DockStack`), and the arrangement is saved per project.
- **The panel holds an editor and the output, split by one `QSplitter`**: side by side while the panel is wide, stacked while it is tall. The switch follows the panel's shape, not whether it is popped out, since a popped-out window can be dragged wide.
- **Above them: the project's scripts in a list, New, Run and Stop**, and a badge while a script runs. Run is `CmdOrCtrl+Enter` while the editor has the focus.
- **The text saves to its file a moment after typing stops and before every run.** There is no unsaved state and no Save button, as with the project.
- **A failure selects its line in the editor and shows it in the output**; clicking an output line that names a line goes there.
- **The view is a Rust struct** (`ScriptView`: the files, the open file, running, the output lines, the failure) that C++ draws; the text itself is in the Qt text document while it is edited, as in every text field.

### Fit with Opsis

- **In place:**
  - The docks: `BuildDocks`, `MakeDock`, `SyncPanels` (`crates/qt/cpp/main_window.cc`), `DockStack` (`dock_style.cc`), `SaveLayout` and `load_layout`.
  - A plain text widget already in use: the Run panel's log is a `QPlainTextEdit` (`run_panel.h`).
  - The menu (`crates/session/src/menu.rs`: `ViewFocusTimeline` `CmdOrCtrl+1`, `ViewFocusMixer` `CmdOrCtrl+2`, `ViewFocusRun` `CmdOrCtrl+3`).
  - Adding a view and its calls ([ui.md](../agent-docs/ui.md), How it works), the `views` and `reopen` scenarios (`e2e_views.cc`, `crates/ui/tests/views.rs`).
  - Colour tokens (`crates/ui/src/look.rs` `TOKENS`, `App::Token`) and the bundled terminal fonts ([fonts.md](../agent-docs/fonts.md)).
- **Missing:** `script_panel.cc` and `.h` (in `SOURCES` and `MOC_HEADERS`, `crates/qt/build.rs`), `ScriptView` and its calls in the bridge and `Host`, `MenuCommand::ViewFocusScript`, the syntax colour tokens, a watch on the open script's file.

### Findings

- **The bottom dock's tabs are Timeline, Mixer and Run**, Timeline first, at `kBottomShare` of the height ([editor.md](../agent-docs/editor.md)).
- **Only groups with more than one tab show the pop-out button**, so a Script tab in the bottom group gets it and a lone dock would not.
- **The app sets `Flag_HideTitleBarWhenTabsVisible`, `Flag_AllowReorderTabs` and `Flag_TitleBarHasMaximizeButton`** (`app.cc`).
- **The saved arrangement is restored with `kdw::LayoutSaver`'s default options.** KDDockWidgets 2.5.1 has `RestoreOption_CloseUnknownDockWidgets`, which the app does not set.
- **The project watcher watches the project folder itself, not its subfolders** (`RecursiveMode::NonRecursive`, `crates/project/src/watch.rs`), and follows only the files the film loads. A change to `scripts/x.lua` from another editor is not seen today.
- **Qt's linked modules are Core, Gui, Widgets, Svg and Test** (`QT_MODULES`, `crates/qt/build.rs`); `QPlainTextEdit` and `QSyntaxHighlighter` are in them.

### Work

1. `ScriptView` and its calls: list the scripts, read one, save one, make one, run, stop.
2. `ScriptPanel` with the splitter, the list, the buttons and the output.
3. The dock, the menu command and its shortcut, the panel in `SyncPanels`.
4. A watch on the open script's file, so a change made elsewhere reloads it when the editor's text is the saved text.
5. The `views` scenario: the tab exists, a script that adds a node adds it, a script that fails marks its line and leaves the graph as it was, the splitter turns when the panel turns tall.
6. The `reopen` scenario: a saved arrangement without the Script dock still opens with the tab in the bottom group.
7. [editor.md](../agent-docs/editor.md): the tab, the splitter rule, the saving.

### Decisions

- **The width-to-height ratio at which the splitter turns**, set by trying it at the bottom dock's default size and popped out.

### Risks

- **An arrangement saved before the Script dock existed has no place for it.** Work 6 checks what restoring does; before the first release, deleting local test projects' view state is the way out.
- **Two windows could show the same script**, once popped out and once more if the panel is ever shown twice; there is one Script dock, so there is one editor.

### Sources

- [editor.md](../agent-docs/editor.md), [ui.md](../agent-docs/ui.md), [e2e.md](../agent-docs/e2e.md); `crates/qt/cpp/main_window.cc`, `app.cc`, `dock_style.cc`; `kddockwidgets-2.5.1/src/LayoutSaver.cpp`, `KDDockWidgets.h`.

## Editor behaviour

### What it changes

- **Highlighting comes from tree-sitter in Rust.** After each change the view sends the text, Rust parses it and returns coloured ranges, and a `QSyntaxHighlighter` applies them line by line.
  - The ranges are in the text's UTF-16 units, the units Qt counts in.
  - Each range has a kind from a fixed set (keyword, string, number, comment, function, the `opsis` module, operator, constant, error), each with a colour token.
- **Syntax errors are marked while typing** from the tree's error nodes; the message a person reads comes from Lua itself when the script runs.
- **Indentation and brackets are pure Rust functions** that take the text and a position and answer with the change to make:
  - Enter keeps the line's indent and adds a level after a block opener (`function`, `do`, `then`, `else`, `repeat`, an open bracket).
  - Typing `end`, `else`, `elseif`, `until` or a closing bracket steps the line back a level.
  - Brackets and quotes close themselves, and typing the closing one moves past it.
  - The bracket matching the one beside the cursor is highlighted.
  - Tab and Shift+Tab indent the selected lines; one shortcut comments them in and out.
  - An indent level is two spaces.
- **Left out of the first version:** inserting `end`, completion, folding, find and replace.

### Fit with Opsis

- **In place:** the pattern of Rust splitting text and C++ drawing it: the variable-chip field (`VarText`, `var_text.cc`) has no tokenizer of its own and reads tokens from `text_tokens` (`crates/ui/src/project.rs`).
- **Missing:** `tree-sitter` and `tree-sitter-lua` in the workspace, the syntax module in `opsis-script`, the highlighter and the key handling in the panel.

### Findings

- **Tree-sitter is C behind a Rust crate.** `tree-sitter` 0.27.2 (MIT, Rust 1.90, `links = "tree-sitter"`) compiles `src/lib.c` with `cc`; its `lib.rs` holds the bindings and the safe wrapper, with 237 uses of `unsafe` inside it. A call such as `Parser::parse` goes straight into the C function, so the crate is one FFI hop, the same as a hand-written binding.
- **`tree-sitter-lua` 0.5.0 (MIT) covers Lua 5.5.** Its `grammar.js` has `global` declarations, `global function` and attributes. It compiles a generated `parser.c` (356,379 bytes of source) and `scanner.c` (4,559 bytes), and depends only on `tree-sitter-language`.
- **It ships four queries**: `highlights.scm`, `locals.scm`, `injections.scm` and `tags.scm`. There are no indent rules, in the grammar or in tree-sitter itself, so the indentation rules are this plan's own.
- **The crate can parse UTF-16 directly** (`Parser::parse_utf16_le`, `binding_rust/lib.rs`), which is how offsets can stay Qt's.
- **`QPlainTextEdit` has no indentation, bracket or comment help of its own.**
- **The alternatives, and why not:**
  - Tree-sitter's C compiled into the C++ view is one hop shorter and reads the Qt text directly, but the pin and the notice would be hand-managed and the editing rules would be C++, tested only through the `views` scenario.
  - A third Rust FFI crate of our own would rewrite the `tree-sitter` crate and be a third crate allowed `unsafe`.
  - Hand-written highlight rules in C++ put a tokenizer in the view, and Lua's long strings and comments (`[==[ … ]==]`) are awkward line by line.
  - KSyntaxHighlighting is MIT, but a KDE framework to build statically for one language, and some of its definition files are LGPL.
  - QScintilla is GPL v3, which fits, but it is a second C++ library to build on three platforms and it ignores the app's colour tokens.

### Work

1. Add `tree-sitter` at `"0.27"` and `tree-sitter-lua` at `"0.5"`.
2. `crates/script/src/syntax.rs`: text in, ranges out, with the capture names of `highlights.scm` mapped to the kinds, and a property test that every range lies inside the text and ranges come in order, over text with characters outside ASCII.
3. The indentation, bracket and comment functions, each with property tests (indenting then outdenting a selection returns it; commenting twice returns it; a typed closer never leaves the level below zero).
4. The highlighter and the key handling in `ScriptPanel`, the colour tokens in `look.rs` and `style.qss`.
5. Measure a parse of a 2,000-line script; when it takes longer than a frame, keep the tree per editor and reparse incrementally (`Tree::edit`).

### Decisions

- **Whether the text crosses as UTF-16 units or as UTF-8 with the ranges converted**, by which the `cxx` bridge carries more simply; both keep Qt's offsets.
- **Whether to reparse incrementally**, by the measurement in Work 5.

### Risks

- **Offsets off by a unit after a character outside the Basic Multilingual Plane** would colour the wrong text; Work 2's property test covers it.
- **A full reparse on every keystroke** may be slow for a very long script; Work 5 decides.

### Sources

- `tree-sitter` 0.27.2 (https://github.com/tree-sitter/tree-sitter): `Cargo.toml`, `binding_rust/build.rs`, `binding_rust/lib.rs`.
- `tree-sitter-lua` 0.5.0 (https://github.com/tree-sitter-grammars/tree-sitter-lua): `grammar.js`, `queries/`, `bindings/rust/build.rs`.
- KSyntaxHighlighting's licence (https://kate-editor.org/2018/10/21/mit-licensed-ksyntaxhighlighting-usage/), Qt Creator's acknowledgements (https://doc.qt.io/qtcreator/creator-acknowledgements.html), QScintilla (https://riverbankcomputing.com/software/qscintilla).
- [inspector.md](../agent-docs/inspector.md), Variable chips.

## Rules, docs and notices

### What it changes

- **`AGENTS.md`**: `opsis-script` in the crate table; under The app, the rule that a script reaches only the open project's files and never the network.
- **A new `agent-docs/scripting.md`** with the code map, the sandbox rules and their reasons, the batch, the `opsis` module's shape and the deliberate limits; its row in §8.
- **`api.md`, `document.md`, `editor.md` and `ui.md`** each gain what their part changed.
- **`THIRD_PARTY_NOTICES.md`** gains the code compiled into the program: Lua (MIT), tree-sitter and the Lua grammar (MIT), beside the crates' own entries where the file lists crates.
- **A user docs page** for scripting: what a script can reach, the `opsis` module, the panel, the endpoint.

### Fit with Opsis

- **In place:** the copy rule that allows naming Lua (`AGENTS.md`, User-facing copy; `CONTRIBUTING.md`); `cargo deny` checking licences on every `lint:fix`; `THIRD_PARTY_NOTICES.md` with a section per bundled work.
- **Missing:** everything in What it changes.

### Work

1. The `AGENTS.md` and `agent-docs` changes, in the same change as the code they describe.
2. The notices.
3. The user docs page, through the docs agents ([docs-site.md](docs-site.md)).
4. Delete this plan once it is built; its design lives in `agent-docs/scripting.md`.

### Decisions

- None.

### Risks

- **The comment check does not read `.lua` files** (`scripts/comments.ts` covers TS, C++, CSS, Rust, Zig and style sheets), so a comment in a test script passes it; either the check learns Lua or the test scripts are kept comment-free by hand.

### Sources

- [AGENTS.md](../AGENTS.md) §2, §6, §7, §8; [packaging.md](../agent-docs/packaging.md).

## Later: Lua as a pack language

Findings only. [node-packs.md](node-packs.md) decides that pack code is WebAssembly components loaded by wasmtime, and nothing here changes that.

- **Native Lua per node is the wrong home for drawing and executing.** The Lua probe links the computer's `libm`, so `math.sin` can differ between machines, which the packs plan guards against inside wasmtime; and native Lua has no sandbox of the kind an executor needs.
- **Fusion (DaVinci Resolve) does run nodes as Lua**: a Fuse is a Lua script that defines a node, runs on LuaJIT and reloads as it is edited, and its documentation says it may be slower than a compiled plugin. This comes from search summaries of Blackmagic's documents; the Fuse SDK pages returned 404.
- **Lua can instead be one more language for writing packs**: a stock Lua interpreter built as an `opsis:pack` component, with the author's `.lua` put into it by the pack checker. A pack stays one component, and every limit of the packs plan holds.
- **wasmtime 50.0.1 turns the exception-handling proposal on by default** under Cranelift (`Config::wasm_exceptions`, `crates/wasmtime/src/config.rs`); the option for the older encoding is hidden and deprecated. Lua raises errors with `setjmp` and `longjmp` (`LUAI_THROW`, `ldo.c`), which on WASI compile to exceptions, so the toolchain must emit the standard encoding.
- **The Bytecode Alliance's article on exceptions (6 November 2025)** reports a Lua interpreter using `setjmp` and `longjmp` working on wasmtime.
- **An older port, `lua-wasi`, stubs `setjmp` and `longjmp`**, so any Lua error there aborts the process; that is what a build without exception handling looks like.
- **`mlua` supports WebAssembly only on `wasm32-unknown-emscripten`** (its README), so the interpreter component would be C Lua with bindings generated for C, or `piccolo`, which is experimental.
- **None of this was built**: this machine has no WASI sysroot and no wasmtime program. The cost of an interpreter inside WebAssembly on every `draw` is unmeasured.
- **Zed and Zellij both chose WebAssembly for extensions**; Zed compiles Rust to `wasm32-wasip2`.

### Sources

- [node-packs.md](node-packs.md); `wasmtime` 50.0.1 `src/config.rs`; `lua-src` 551.0.2 `lua-5.5.1/ldo.c`; `mlua` 0.12.2 `README.md`.
- Exceptions in Cranelift and Wasmtime (https://bytecodealliance.org/articles/wasmtime-exceptions); lua-wasi (https://github.com/singlestore-labs/lua-wasi/blob/main/README.md).
- The Resolve scripting API (https://wiki.dvresolve.com/developer-docs/scripting-api), the Fusion Fuse SDK (https://documents.blackmagicdesign.com/UserManuals/Fusion_Fuse_SDK.pdf).
- Zed's extensions (https://zed.dev/docs/extensions/developing-extensions), Zellij's plugins (https://zellij.dev/documentation/plugins.html).

## Appendix: how the findings were measured

Both probes were small Cargo projects under `~/.cache`, built with `CARGO_BUILD_JOBS=8` and deleted afterwards.

- **Sizes.** One program that builds two nodes as serde values, hands them to Lua as a table, runs a short script and reads its result back. It was built once with no Lua and once per flavour (`mlua` features `lua54`, `lua55`, `luajit`, `luau`, each with `vendored`, `serde` and `async`), with the repository's release profile. The sizes are `stat` on each stripped program; the added size is the difference from the build with no Lua.
- **Reach and confinement.** One program on Lua 5.5.1 with three steps:
  - a state from `Lua::new()` running a script that tries each door (`io.open`, `loadfile`, `dofile`, `load` of bytecode, `os.getenv`, `os.execute`, `io.popen`, `package.searchpath`, `package.loaded.os.execute`);
  - a state with `coroutine`, `table`, `string`, `utf8`, `math`, `os` and `package`, then constrained (the `os` functions removed in place, `io` replaced by one `open` over `cap_std::fs::Dir`, `os.remove` and `os.rename` replaced, `dofile`, `loadfile` and `string.dump` removed, `load` forced to text), running the same script;
  - the constrained state running the attempts in Sandbox's table against a folder holding `notes/a.txt`, a link to `/etc`, a link to `notes`, a link to a file outside and a file beside the folder, then a check of what was on disk.
