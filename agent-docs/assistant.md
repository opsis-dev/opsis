# Assistant

How Opsis works with coding agents: the Assistant switch that serves the Local API, the `opsis-mcp` program that gives any agent Opsis's tools, and the headless mode agents drive. Rust paths are relative to `crates/` unless they name another folder. The Local API's routes are in [api.md](api.md), the scripts agents run in [scripting.md](scripting.md), starting and closing the editor in [editor.md](editor.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern            | Where                                                                                                                                                                                                                                                                                                                                                |
| ------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| The switch         | `types/src/settings.rs` → `AssistantSection`; `session/src/api.rs` → `follow_setting`; `qt/cpp/settings_window.cc` → `AssistantPage`; `qt/cpp/window_chrome.cc` → `AppBar::SyncAssistantBadge`                                                                                                                                                       |
| Headless arguments | `ui/src/main.rs` → `headless_folder`, `USAGE`                                                                                                                                                                                                                                                                                                        |
| Headless start     | `qt/cpp/app.cc` → `App::Run`; `session/src/headless.rs` → `open_project`; `session/src/api.rs` → `open_and_show`                                                                                                                                                                                                                                     |
| Headless answers   | `session/src/platform.rs` → `Platform::headless`; `ui/src/platform.rs` → `QtPlatform`; `session/src/document.rs` → `ask_about_disk`; `session/src/exit.rs` → `confirm`; `App::ShowError`                                                                                                                                                             |
| `opsis-mcp`        | `mcp/src/main.rs`; `mcp/src/cli.rs` → `parse`; `mcp/src/full.rs` → `route_tools`, `call_route`; `mcp/src/http.rs` → `router`, `authorized`; `mcp/src/server.rs` → `OpsisTools`, `GUIDE`; `mcp/src/tools.rs` → `assistant_tools`, `call`; `mcp/src/local_api.rs` → `LocalApi`, `EventStream`; `folders/src/address.rs` → `this_computers_api_address` |
| The guide          | `mcp/guide.md`; its coverage test `mcp/tests/guide.rs`                                                                                                                                                                                                                                                                                               |
| Tests              | `session/tests/headless.rs`; `ui/tests/views.rs` → `a_headless_start_serves_the_project_and_ends_on_a_signal`; `ui/src/platform.rs` and `ui/src/main.rs` tests; `mcp/tests/tools.rs` (a fake Local API on a socket)                                                                                                                                  |

## Rules

- **One switch, Settings → Assistant (`settings.assistant.enabled`, off by default), serves the Local API with every route** ([api.md](api.md)); agents change the project through Lua, so there is no separate switch for scripts.
- **`opsis-mcp` is a program of its own, never part of the app**, so an agent's tool server can't take the editor down, and it never starts Opsis: it finds the socket the way the app names it (`this_computers_api_address`), and with no socket or no answer every call fails with "Opsis isn't running, or the assistant is off."
  - `opsis-mcp stdio` (the default) serves MCP on standard input and output; logs go to standard error.
- **`opsis-mcp http` serves the same tools over Streamable HTTP at `/mcp`, guarded three ways** (`mcp/src/http.rs` → `router`):
  - A token made at start (`new_token`, 32 random bytes) is printed with the address and required on every request as `Authorization: Bearer <token>`, compared in constant time (`authorized`); rmcp checks no token itself, so an `axum` layer answers 401.
  - rmcp's `Host` check allows only loopback names, plus the `--host` given (`allowed_hosts`); another host opens the server to the network with the token as its only guard, and a warning says so.
  - `enforce_origin_validation` answers 403 to any request that carries an `Origin`, so no web page can call it; command-line clients send none.
  - It binds `127.0.0.1` unless `--host` says otherwise, `--port 0` picks a free port, and it serves statelessly (`with_legacy_session_mode(false)`): the MCP `2026-07-28` revision has no sessions, and older clients get none either.
  - Its tools are `run_lua` (the script route, answering with the outcome and the editor's problems; a failed script is a tool error), `undo` and `redo` (the Local API routes, since undo and redo are refused while a script runs) and `wait_run`.
  - `wait_run` follows the run events before it reads the run state, so a run that ends in between is still seen; a run that isn't going answers so, and the wait is held to 1–600 seconds.
- **`--full` adds every Local API route as a tool, built from `/openapi.json` on each listing and call** (`mcp/src/full.rs` → `route_tools`), so a new route becomes a tool with no change to `opsis-mcp`.
  - A tool is named from its method and path (`tool_name`: `post_project_nodes`); its path parameters become required strings, holding only letters, digits, `-`, `_`, `.` and `:` so a value can't change the path, and its request body becomes `body`.
  - Each tool's input schema carries the component schemas it uses as `$defs`, so it stands alone.
  - Only routes whose success answer is JSON become tools: the run event stream never ends, and `/openapi.json` is the document itself.
- **`--session <id>` sends `x-opsis-session` on every Local API request**, so the app can tell which Agent panel session started a run.
- **The `opsis` module guide (`mcp/guide.md`) is the module's only documentation** (`opsis.lua` has no comments), sent as the MCP server's instructions so every agent receives it and nothing is written into the project. It says to change the project only through Lua: Rust watches the project file, and a change it didn't make asks the disk-change question, skips the undo batch and skips the checks.
  - `the_guide_names_every_part_of_the_opsis_module` fails when a function, table or run method of `opsis.lua` isn't named in it.
- **`opsis --headless <project folder>` is the whole app with no window, for agents to drive**: `App::Run` sets Qt's `offscreen` platform from code before `QApplication` is made, `MainWindow::StartAsChosen` skips the start choice, and the session opens the given folder once it begins (`open_project`). A folder that can't be opened ends the app with code 1; any other argument list prints the usage and exits with code 2.
  - Headless always serves the Local API, whatever the switch says (`follow_setting`), since an agent is its only user.
  - It runs until a signal ends it; the signal saves the project and stops a run (`close_on_signals`).
- **The flag reaches the session as `Platform::headless`, so everything that would wait for a person answers by itself**:
  - The disk-change question answers **Load the file** (Undo brings the app's version back), or **Wait** when the file isn't a valid project yet (`ask_about_disk`).
  - Closing while work runs stops the work and exits, saving first (`confirm`).
  - File and folder pickers reply as cancelled, and notices are logged (`QtPlatform`).
  - Error message boxes are written to the log (`App::ShowError`).

## Deliberate limits

- **Headless on Windows ends only by ending the process**: `close_on_signals` listens on Unix only, so nothing saves the project first.
- **Dialogs a person opens with a click (Quick add, New project, renaming a script) have no headless answer**: with no window to click, nothing opens them.
