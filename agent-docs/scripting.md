# Scripting

Lua scripts that drive the open project: the graph, the timeline, the mixer, runs and exports, with file access held to the project folder. The Script panel writes and runs them, and the Local API runs them behind its own switch. Rust paths are relative to `crates/` unless they name another folder. The document batch a script edits through is in [document.md](document.md), the API routes in [api.md](api.md), the panel's dock in [editor.md](editor.md), the bridge it crosses in [ui.md](ui.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                                 | Where                                                                                                                                                                                                                                 |
| --------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| A run of one script                     | `script/src/runtime.rs` → `run`, `Script`, `Ending`, `failure_of`, `MEMORY_LIMIT`, `CHECK_EVERY`                                                                                                                                      |
| The standard library a script gets      | `script/src/sandbox.rs` → `install` (`print`, `warn`, `load`, `loadfile`, `dofile`, `require`, `os`, the `pcall` guard, `PLAIN`)                                                                                                      |
| Project-confined files                  | `script/src/paths.rs` → `inside_project`, `writable`; `script/src/folder.rs` → `Folder`; `script/src/io.rs` (the Rust `io` library)                                                                                                   |
| Output                                  | `script/src/output.rs` → `Streams`                                                                                                                                                                                                    |
| The `opsis` module                      | `script/src/module.rs` (the Rust natives, `located`), `script/src/opsis.lua` (the proxies and tables scripts see)                                                                                                                     |
| Values across the boundary              | `script/src/values.rs` → `to_lua`, `from_lua`                                                                                                                                                                                         |
| What a script asks of the app           | `script/src/host.rs` → `ScriptHost`, `RunEvents`; implemented by `session/src/script.rs` → `SessionHost`                                                                                                                              |
| Starting, stopping and the outcome      | `session/src/script.rs` → `run_script`, `stop_script`, `stop_and_wait`, `script_state`, `Scripts`, `SCRIPT_TASK`; events `ScriptChanged`, `ScriptLogged`                                                                              |
| File settings scripts may not point out | `compile/src/check.rs` → `absolute_file_settings`                                                                                                                                                                                     |
| Shared shapes                           | `types/src/script.rs` → `ScriptOutcome`, `ScriptFailure`, `ScriptLine`, `ScriptRequest`, `ScriptStatus`, `SCRIPTS_DIR`                                                                                                                |
| Highlighting and editing rules          | `script/src/syntax.rs` → `highlight`; `script/src/themes.rs` → `themes`, `theme`, `Theme::style`; `script/src/editing.rs` → `enter`, `typed`, `backspace`, `indent_lines`, `toggle_comment`, `matching_bracket`; `script/src/text.rs` |
| The panel                               | `qt/cpp/script_panel.cc` → `ScriptPanel`, `ScriptEditor`, `ScriptHighlighter`; `ui/src/script_panel.rs` → `ScriptEditing`, `view`, `key`                                                                                              |
| Tests                                   | `script/tests/sandbox.rs`, `script/tests/module.rs` (a fake `ScriptHost`), `session/tests/scripts.rs`, the `views` and `reopen` scenarios (`qt/cpp/e2e_views.cc`)                                                                     |

## Rules

### The language

- **Scripts are PUC-Rio Lua 5.5, compiled into the program through `mlua` (`lua55`, `vendored`, `serde`).** Nothing ships beside the program and the packager stages nothing.
  - Not LuaJIT: it is the Lua 5.1 dialect with rolling releases and no tarballs, and its hooks don't run inside compiled traces unless it is built with `LUAJIT_ENABLE_CHECKHOOK`, which would break Stop; its speed buys nothing when a script is mostly host calls.
  - Not Luau: another dialect, C++, and the largest build (1.88 MiB against Lua 5.5's 574 KiB in a release build).
  - Not `piccolo`: its README calls it experimental, with `io`, `os`, `package`, `string`, `table` and `utf8` missing or sparse.
- **`mlua` runs without `async` and `send`**: a script runs on its own thread and blocks there, and its state never leaves that thread.
- **Each run gets a fresh state**, made and dropped on the script's thread; nothing carries over between runs.
- **Copy names Lua where a person needs to know the language** ([AGENTS.md](../AGENTS.md), User-facing copy): the switch "Run Lua scripts", and nowhere else beyond the panel's own messages.

### What a script reaches

- **A script gets Lua's standard library and the `opsis` module, nothing else**: basic, `coroutine`, `table`, `string`, `utf8`, `math` and a changed `os`, plus an `io` written in Rust. `package` and `debug` are never loaded and `string.dump` is removed.
  - `mlua`'s safe state guards memory safety only: in a plain `Lua::new()` a script read `/etc/hostname` through `io.open`, `loadfile` and `dofile`, ran commands through `os.execute` and `io.popen`, read `HOME` through `os.getenv`, ran bytecode through `load(string.dump(f))` and found files through `package.searchpath`. Each of those doors is shut here.
  - `package` stays unloaded because, loaded, `package.loaded.os.execute` survives a replaced `os` global and `require` runs any `.lua` on `package.path`.
- **`os` is changed in place on the table Lua made**: `execute` (a shell), `getenv` (the environment, secrets included), `setlocale` (the whole app's C locale) and `tmpname` (the system temp folder) are removed; `remove` and `rename` are project-confined; `exit` ends the script, as success for `true`, `0` or nothing.
- **`load` only takes text, and `require "name"` reads `scripts/name.lua` then `scripts/name/init.lua` through the same project folder, once per run.**
- **Every path resolves beneath one `cap_std::fs::Dir` opened on the project folder**, never by comparing path text.
  - A relative path is relative to the project folder; an absolute one is accepted only inside it; a `project:` ref is the path it names (`inside_project`).
  - The project's own `is_inside` (`project/src/paths.rs`) is lexical, so a link pointing out of the project passes it: right for refs, wrong here.
- **Reading is allowed anywhere in the project; writing, removing and renaming are refused for the project file and for `.opsis/` outside `.opsis/out`**, checked on the resolved path (`Dir::canonicalize`), so a link into `.opsis/cache` doesn't get around it.
- **A script never reaches the network**: nothing in its library opens a socket, and the session calls it can make don't download.
- **An edit from a script may not set a file setting to an absolute path** (`absolute_file_settings`: fields with the `asset` widget or `x-accept`), and importing outside files is not bound, so `io` can never read outside the project through a copy.

### Running

- **One script runs at a time, only on an open project**, on a plain thread whose host calls block on the session's runtime (`SessionHost`); a second start answers "A script is already running."
- **A run is one document batch** ([document.md](document.md)): kept when the script ends normally, dropped when it fails, is stopped, passes its memory limit, or the project closes or another opens, so a failed script leaves the document as it was. Files it wrote and runs it started remain.
  - Undo and redo are refused while it runs, so a rollback always has its start.
- **Stop sets a flag that an instruction hook reads every `CHECK_EVERY` instructions, and every wait (`opsis.sleep`, a run's events) checks it.**
  - The hook is the global one (`set_global_hook`), which Lua copies to every coroutine a script makes; a per-thread hook would leave a coroutine's loop unstoppable.
  - `pcall`, `xpcall`, `coroutine.resume` and `coroutine.close` are wrapped in Lua so that once the script is stopped or has called `os.exit`, catching the error re-raises it; the wrappers are Lua functions so yields inside `pcall` still work.
  - 10,000 instructions stopped a tight loop 179 µs after the flag in a release build, where 10⁸ loop turns took 0.95 s.
- **The memory limit is 64 MiB.** The whole compile of the largest example (`data-story`: workflow, compile result, film and flat timeline) takes 129 KB in Lua, so the limit leaves room for any real script and stops a runaway one.
- **There is no time limit**: a script may wait for a render; Stop ends it.
- **Closing the project, opening another, adopting one and quitting stop the running script and wait for it** (`stop_and_wait`), before anything saves.
- **A run is listed in `AppState.tasks` ("Running a script")** so the exit dialog names it.
- **Output is kept as lines** (`print`, `warn`, `io.write`, `io.stdout`, `io.stderr`, the failure), capped at the run log's `MAX_LOG_LINES`, and sent to the windows as `ScriptLogged`.
- **The outcome is one value (`ScriptOutcome`)**: whether it ended well, the value it returned as JSON, the output, the failure with its file and line, whether it was stopped and whether the document was rolled back.

### The `opsis` module

- **Project data is read live and set directly, as in Blender's `bpy.data`**: `opsis.nodes.open.params.url = "…"` is one `WorkflowEdit::UpdateNode`; a handle to a node that is gone fails with "This node is gone."
- **The proxies make one edit per assignment**: a loop over a thousand nodes makes a thousand edits and compile requests; compiles merge (one running, one waiting) and the batch keeps undo at one step.
- **Everything the Local API can do has a Lua call, except opening a project, undo, redo and the script routes themselves**; `has_a_lua_call_for_every_local_api_route_but_the_ones_left_out` (`script/tests/module.rs`) reads the OpenAPI document and fails on a route with no call.
- **Left out on purpose**: opening, creating and closing projects, importing files, the app's settings, engine setup, font downloads and removals, updates, sign-in and the streamed views, the terminal window, licences, undo and redo.
- **Timeline reads and edits compile the current revision first** (`latest_compile`), because a timeline edit works against the film of a compile; a script never edits against a stale film.
- **An export is a run of an output node**, and `run:result(id).file` names its first artifact, which `io` can read.
- **Run events are an iterator with no subscriptions**: `run:events()` yields each `RunEvent` of that run until `run:done`; the subscription is made before the run starts, so none is missed at the start. One event stream instead of property callbacks, because Blender's `bpy.msgbus` misses changes made in the viewport.
- **Errors a script sees are plain strings with the script's file and line** (`located`, `PLAIN`): `mlua` raises Rust errors as userdata, which `message:find` can't read.
- **Rust values cross as Lua tables through serde** (`values.rs`): `None` becomes `nil`, JSON `null` becomes `opsis.null`, so a node's settings come back unchanged; an empty table goes back as a map, so a new empty list is `opsis.list()`, and the prelude sends lists it builds the same way.

### The panel

- **The Script tab sits in the bottom group after Run** (View → Script, Cmd/Ctrl+4) and pops out like the others.
  - KDDockWidgets floats a dock that a saved arrangement doesn't name (`floatUnknownWidgets`), so `BuildDocks` puts the Script tab back as a tab of the bottom group when the arrangement doesn't name it (`LayoutKnows`).
- **The editor and the output share one splitter, side by side while the panel is at least twice as wide as tall and stacked otherwise** (`kTurnRatio`), following the panel's shape rather than whether it is popped out: the docked bottom bar sits side by side, and a full-screen 16:9 window stacks.
- **A toolbar button that is disabled while it has focus hands focus to the editor** (`ScriptPanel::Enable`); otherwise Qt passes it to the next control that takes it, the theme box, whenever Run starts a script or Stop ends one.
- **The text saves itself 600 ms after typing stops, before every run, when another script opens and when the project closes**; there is no Save button. Each save names the project it belongs to (`script_save(dir, …)`), so text typed in one project never lands in the next.
  - A save still waiting when its script disappears from the list (renamed or deleted outside the app) or the project changes is dropped, so it never brings the old file back.
- **Rename renames the open script's file** (`script_rename`, `ui/src/script_panel.rs` → `renamed`, `rename`): a bare name lands in `scripts/` with `.lua` added, an existing script is never overwritten, and the open script and its last saved text follow the new name.
- **Delete moves the open script to the system trash after asking** (`script_delete`, `ui/src/script_panel.rs` → `discard`, the `trash` crate), and refuses while that script runs. The `views` test points the trash at its own folder through `XDG_DATA_HOME`.
- **With no scripts the list says so and the editor says to press New.**
- **A change on disk reloads the open script only while the editor's text is the text last saved or read** (`ScriptEditing::reload`), from a watch on `scripts/`.
  - The watch reacts only to new, written, renamed and removed files (`changes_files`): `notify` also reports a file or folder being opened, and the panel reads both on every rebuild, so reacting to them would rebuild it every 300 ms and throw the script list back while it is being chosen.
  - The list is refilled only when the files or the open script change, and never while its drop-down is open.
- **A failure marks its line in the editor when it is in the open script, and an output line that names the open script's line moves there when clicked.**
- **Highlighting is tree-sitter in Rust** (`tree-sitter` 0.27 and `tree-sitter-lua` 0.5, whose `grammar.js` has Lua 5.5's `global` declarations): C++ sends the text as UTF-8 and gets ranges in UTF-16 units, Qt's own, and `ScriptHighlighter` applies them block by block. The captures of the grammar's `highlights.scm` map to nine kinds, each a `syntax-*` colour token.
  - Not tree-sitter's C in the C++ view, which would make the pin and the notice hand-managed and the editing rules C++ tested only by scenarios; not a third FFI crate, which would be a third crate allowed `unsafe`; not KSyntaxHighlighting or QScintilla, a KDE framework or a second C++ library to build on three platforms for one language.
  - Later patterns in the query win over earlier ones, as in Helix, so a call to `print` is `function.call`, not `function.builtin`.
  - Syntax errors underline from the tree's error and missing nodes, cut at the end of their first line so an unclosed block doesn't underline the rest of the file.
  - A full parse and query of a 2,004-line script took 15.8 ms in a release build (6.5 ms parse, 7.6 ms query), within a frame, and the panel highlights 30 ms after typing stops, so there is no incremental reparse.
- **The code theme and size are app settings** (`settings.script.theme`, `settings.script.fontSize`, `SCRIPT_FONT_SIZE`), chosen in the Script toolbar and kept for every project; only the Script editor and its output change, which scroll, so no size or theme clips the rest of the app.
  - The themes are Helix's (`themes.rs`, from `bun run script:themes`), and Helix's own default theme is the default. A capture takes the theme's longest matching key (`function.call` falls back to `function`), as in Helix.
  - The editor and output take the theme's background, text, selection, bracket match (`ui.cursor.match`), error and warning colours; the generator resolves `inherits` and the palette, falls back to `ui.background`'s text colour where a theme sets no `ui.text`, and leaves out the themes that leave their background or text to the terminal.
  - Only dark themes are listed, by readable names the generator makes from the file names (`readable` in `scripts/script-themes.ts`), since the rest of the editor is dark.
- **Indentation, brackets and comments are pure Rust functions with property tests** (`editing.rs`): Enter keeps the indent and adds a level after a block opener; typing `end`, `else`, `elseif`, `until` or a closer steps the line back to where its block opened; brackets and quotes close themselves and typing the closer moves past it; Backspace between a pair removes both; Tab and Shift+Tab indent lines; Ctrl+/ comments them; an indent is two spaces. `QPlainTextEdit` has none of this, and tree-sitter has no indent rules.

## How it works

- **Starting:** `run_script` checks the project and that nothing runs, opens the document batch, registers the task and spawns the thread; the thread runs `opsis_script::run`, which makes the state, installs the sandbox and the `opsis` module (the natives, then `opsis.lua` over them) and runs the chunk named `@<file>` as text. At the end it closes the batch (`end_batch`, which announces the document and asks for a compile), stores the outcome, ends the task, sends `ScriptChanged` and answers whoever waits (the endpoint).
- **From the panel:** `ScriptPanel` asks Rust for `script_view` on every `Script` event and on project changes, reads and saves the open file through Rust, and runs it with `script_run`, which saves first and starts the run with the file; the outcome reaches it as the next view.
- **Keys:** `ScriptEditor` sends the text, the selection and the key to `script_key`, applies the change Rust answers as one edit block and puts the cursor where Rust says; a key Rust doesn't handle goes to `QPlainTextEdit`.

## Deliberate limits

- **Edits a person makes while a script runs belong to its batch**, and a failed script undoes them too; the panel's Running badge says so.
- **Autosave and a run's own save still write the project file during a batch**, so an app killed mid-script leaves the partial edits on disk with no undo step.
- **A link with an absolute target is refused even when it points inside the project**: `cap-std` resolves beneath the folder handle and treats an absolute target as an escape. A hard link to an outside file can't be told from an ordinary file; a script can't make one, a person can.
- **Between resolving a path and opening it a link can change**; both go through the same folder handle, so the worst case is a write landing in a read-only part of the project, never outside it.
- **A script that falls 256 run events behind loses some** (`RUN_EVENTS_KEPT`); `run:missed()` counts them and `run:wait()` reads the run's state at the end, so the outcome is never lost.
- **Playback calls do nothing without an editor window** and answer that no window is playing; playback sound lives in `opsis-ui`.
- **A table with both a list part and keys crosses as a list**, dropping the keys, as `mlua` reads it.
- **`os.date` uses the C locale and the computer's time zone**, and the maths library is the computer's `libm`, as in any Lua; that is why native Lua is wrong for drawing, which stays with node packs ([plans/node-packs.md](../plans/node-packs.md)).
- **There is no completion, folding, find and replace, or inserted `end`.**
