# Assistant

How Opsis works with coding agents: an Agent panel in the editor that runs Opsis's own lean agent or any agent the user has installed (Claude Code, Codex, goose, fx…), over the Agent Client Protocol (ACP); a separate `opsis-mcp` program that gives any agent Opsis's tools over MCP; a headless mode; and the logs. The suggestion buttons in the editor are in [ai-assist.md](ai-assist.md).

Status: plan only. Nothing here is built yet.

## How to read this plan

- Sections 1–7 are the design; section 8 is the build order, phase by phase; section 9 holds the tools that are deferred and not finalized.
- Each section has the same parts where they apply: **In place**, **Work** (numbered), **Decided**, **Risks**.
- Rules from `AGENTS.md` and `agent-docs/` hold throughout; when they disagree with this plan, they win.
- External facts name their source in **Sources** with the date they were checked.

## Summary

| Part                        | What users get                                                                                                  |
| --------------------------- | --------------------------------------------------------------------------------------------------------------- |
| 1. The Assistant switch     | One switch in Settings → Assistant, off by default, that lets agents reach the open project                     |
| 2. `opsis-mcp`              | Opsis's tools for any MCP client: Lua scripts, undo, redo, waiting on runs; every Local API route with `--full` |
| 3. Headless                 | `opsis --headless <project>`: the whole app with no window, for agents to drive                                 |
| 4. The Agent panel          | A chat with any installed agent, on the right beside the Inspector, tied to the open project                    |
| 5. `opsis-agent`            | The default agent, lean: files, search, shell, subagents and Opsis's tools, with the user's own model           |
| 6. The `opsis` module guide | One guide every agent receives, so it changes the project through Lua and nothing else                          |
| 7. Logs                     | A request log per agent connection, and every session's events in the app database                              |

## Conventions

- **Nothing in Opsis uses the internet unless the user asks.** The assistant is off until the user turns it on, and `opsis-agent` contacts only the model service the user chose. An outside agent's own network use (its web tools, its sign-in, its model) is the user's agent's business: Opsis neither blocks it nor offers sign-in for it.
- **Every new program is its own process**, like `opsis-renderer` and the FFmpeg sidecar, so its crash never ends the app. Each gets its own temp folder ([engine.md](../agent-docs/engine.md)).
- **User-facing copy** says "Assistant" and "Agent" and names what a person works with: the agents they install (Claude Code, Codex…) and the model services they give a key for. ACP, MCP, rig and rmcp never appear in the editor.
- **Dependencies, pinned in the root `Cargo.toml`:** `agent-client-protocol` (Apache-2.0), `rmcp` (Apache-2.0) and `rig-core` (MIT), all GPLv3-compatible. Each moves fast (section 10), so each is updated on purpose, never in passing.
- **Checks for every phase:** `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`, then the app itself (`check-app`).

## 1. The Assistant switch and the Local API

**In place.**

- The Local API (`crates/api`, [api.md](../agent-docs/api.md)): plain HTTP with `axum` on `<data folder>/api.sock` (or `OPSIS_API_SOCKET`), mode 0600, every route documented in `crates/api/src/openapi.rs`. Off by default behind Settings → API (`settings.api.enabled`); the script routes need a second switch, Run Lua scripts (`settings.api.scripts`).
- Lua scripts reach everything the Local API does except opening a project, undo, redo and the script routes ([scripting.md](../agent-docs/scripting.md)).

**Work.**

1. **Settings → API becomes Settings → Assistant**, one switch (`settings.assistant.enabled`), off by default, shown as an Assistant badge in every app bar. It serves the socket with every route, the script routes included; the second switch goes away.
2. **The socket becomes an internal path.** It stays HTTP on `api.sock` with mode 0600, and its routes stay as they are, but it is no longer documented for people: agents reach Opsis through `opsis-mcp`. [api.md](../agent-docs/api.md) says so.
3. **The project routes stay under their own path and can grow** with what agents need beyond Lua (running checks, adding fonts…). The Local API always serves every route; `opsis-mcp` decides which become tools.
4. **Settings → Assistant also holds** `opsis-agent`'s optional one-time setup (section 5), the agent list (section 4) and the request log (section 7).

## 2. `opsis-mcp`

A program of its own (`crates/mcp`, `opsis-mcp`), shipped beside the app, that serves Opsis's tools over MCP and reaches the running app through the Local API's socket.

**In place.**

- rmcp 3.11.0, the official Rust MCP SDK, implements the stable MCP `2026-07-28` revision, which has no protocol-level sessions; rmcp serves those clients statelessly by default, and `with_legacy_session_mode(false)` makes older clients stateless too.
- rmcp's HTTP server (`StreamableHttpServerConfig`, `transport/streamable_http_server/tower.rs`) checks `Host` by default (`allowed_hosts`: `localhost`, `127.0.0.1`, `::1`; anything else gets 403) and checks `Origin` once `enforce_origin_validation()` is called (a present `Origin` that isn't allowed gets 403; a request with none passes). It has no token check of its own.
- `api_address` (`crates/folders/src/address.rs`, with `this_computers_api_address` for programs outside the app) names the socket from the data folder or `OPSIS_API_SOCKET`.

**Work.**

1. **Modes.** `opsis-mcp stdio` (the default) and `opsis-mcp http`:
   - `--host` (default `127.0.0.1`); another host is added to `allowed_hosts`, and opens the server to the network with the token as its only guard.
   - `--port` (`0` picks a free port); the address is printed at start.
   - A token made at start, printed with the address, and required on every request as `Authorization: Bearer <token>` (an `axum` layer, since rmcp has none); a missing or wrong token gets 401.
   - `enforce_origin_validation()` on, so a browser page can never call it.
2. **It never starts Opsis.** It needs a running instance with the assistant on: it finds the socket through `api_address`, and with no socket or no answer it fails every call with "Opsis isn't running, or the assistant is off."
3. **Tools.** Without `--full`, the assistant set:
   - `run_lua(source)`: runs a script through the script route and returns its `ScriptOutcome` (`ok`, the returned value as JSON, the output lines, the failure, `stopped`, `rolled_back`).
   - `undo`, `redo`: the Local API routes; they can't be Lua calls, since undo and redo are refused while a script runs, so a script's rollback always has its start.
   - `wait_run(id, seconds)`: waits on a run's events for at most that long and returns its state, so an agent waits without holding the one-script slot.
4. **`--full`** adds every Local API route as a tool, built from `/openapi.json` at start, so a new route becomes a tool with no change to `opsis-mcp`.
5. **The guide.** The `opsis` module guide (section 6) is sent as the server's `instructions`, so every MCP client receives it and nothing is written into the project.
6. **Session tag.** When the app starts `opsis-mcp` for an Agent panel session it passes `--session <id>`, sent on every Local API request, so the app knows which session started a run (section 4, run-finished messages).
7. **Tests.** Against a fake Local API: each tool's request and answer; `--full` lists one tool per OpenAPI route; over HTTP, a wrong `Host` or `Origin` gets 403 and a missing or wrong token 401; with no socket every call fails with the plain message.

**Decided.**

- A separate program, never built into the app.
- `stdio` is the default; `http` is token-guarded, loopback unless `--host` says otherwise.
- `opsis-mcp` never starts Opsis.

## 3. Headless

**In place.**

- `opsis-session` has no window code; it asks its host for window things only through `Platform` (`crates/session/src/platform.rs`), which `QtPlatform` (`crates/ui/src/platform.rs`) implements for the editor and the script tests implement without Qt (`crates/session/tests/scripts.rs`).
- The app builds `QApplication` itself from a fixed argument list and sets a Qt variable from code before it (`qputenv("QT_QPA_PLATFORMTHEME", …)`, `crates/qt/cpp/app.cc`).
- Qt's `offscreen` platform is linked on Linux, macOS and Windows (`crates/qt/build.rs`), and `cargo test -p opsis-ui` already runs the editor on it ([e2e.md](../agent-docs/e2e.md)).
- A signal closes the app and saves the project (`close_on_signals`, `crates/session/src/exit.rs`).
- `main` (`crates/ui/src/main.rs`) reads no arguments.

**Work.**

1. **`opsis --headless <project folder>`.** `main` reads its arguments; `--headless` sets the `offscreen` platform from code before `QApplication` is made, opens the given project and serves the Local API's socket for that run. It runs until a signal ends it.
2. **The flag reaches the session**, so everything that would wait for a person answers by itself:
   - The disk-change question (`AskAboutDisk`, `resolve_disk_change`) answers **Load the file** (Undo brings the app's version back); when the file isn't a valid project yet, it answers **Wait**.
   - Exiting while work runs **stops the work and exits**.
   - File and folder pickers reply as cancelled.
   - Message boxes are written to the log and closed.
3. **Tests.** A headless start answers `run_lua` through `opsis-mcp`; each dialog's automatic answer; a signal mid-run stops the run and exits.

**Decided.**

- A flag on the app, not an environment variable or a second host.
- Disk change: load the file. Exit with running work: stop it and exit.

## 4. The Agent panel

The app as an ACP client: `agent-client-protocol` 5.4.0 (`Client.v1()`; `AcpAgent` from the `process` feature starts an agent program and talks to it over stdin and stdout).

**In place.**

- Docks ([editor.md](../agent-docs/editor.md)): the node list on the left, the Inspector on the right, and Timeline, Mixer, Run and Script as tabs of one bottom dock; any group with more than one tab gets the pop-out and hide corner buttons (`DockStack`), and the arrangement is saved per project. Only the bottom dock has tabs because of how `BuildDocks` builds them.
- Long app work is listed in the exit dialog through `AppState.tasks`.
- The editor keeps no state of its own; views are views onto Rust ([ui.md](../agent-docs/ui.md)).

**Work.**

1. **The panel.** An Agent dock tabbed with the Inspector on the right (`BuildDocks`), so the right group gets tabs and the corner buttons. Views: `crates/qt/cpp/agent_panel.cc` over `crates/ui/src/agent_panel.rs`; the session state in `opsis-session`.
2. **Agents.** Presets from the ACP registry's list, as a generated snapshot (a `bun run` script writes it, and it joins the generated-code table in `AGENTS.md`), so the app makes no network request. Opsis checks which presets are installed on this computer and their versions, installs nothing, and the user can add their own commands. `opsis-agent` is the default agent.
3. **A session.** One per open project, started in the project folder:
   - `session/new` passes `opsis-mcp stdio --session <id>` as the stdio tool server (ACP requires every agent to accept stdio tool servers), with the tool set the agent's preset names.
   - Sign-in is the agent's: when it answers `auth_required`, the panel shows the sign-in methods it lists and passes the choice back with `authenticate`.
   - `session/request_permission` shows the agent's request with the options it offers; the panel answers with the user's choice.
   - `session/set_mode` switches the agent's modes from the panel.
   - Closing the project ends the session; opening the project again brings it back with `session/load` when the agent offers it, and from the app database's projection (section 7) when it doesn't.
   - A turn in progress is listed in `AppState.tasks` ("Waiting for the agent").
4. **Run-finished messages.** When a run started through `opsis-mcp` with this session's tag ends, the panel sends the agent a new message with the result ("Export finished: `.opsis/out/film.mp4`", or the error); when a turn is in progress it waits for the next one, since ACP runs one turn at a time.
5. **A crashed agent** ends only its own program: the panel shows the error, starts it again and reloads the session.
6. **Tests.** Against a fake agent on the crate's in-memory transport: permission answers, sign-in, set mode, load, the run-finished message queued behind a running turn; the dock arrangement with the Agent tab saved and restored (`reopen` scenario).

**Decided.**

- Right side, a tab with the Inspector.
- Registry presets plus own commands, detection only, no installation.
- Outside agents' own tools, web tools included, aren't blocked.

## 5. `opsis-agent`

The default agent, and the lean one: it takes the place fx would have had, since fx doesn't run on Windows, and serves people who use a local or OpenAI-compatible model and don't want to install an agent. It is a program of its own (`crates/agent`, `opsis-agent`), shipped beside the app, speaking ACP as the agent through the same `agent-client-protocol` crate, with `rig-core` 0.46 for model requests. It links `opsis-persistence` for the keyring and its own database, and nothing else of the app's.

**In place.**

- `rig-core` (MIT) handles streaming and tool calls for each model service; its providers are modules created directly, so the ones Opsis never creates are left out of the release binary (`lto = true`, `codegen-units = 1`) as long as its all-provider registry (`providers/registry.rs`) isn't used.
- Keys: `Keyring::set_key` and `key` (`crates/store/src/keys.rs`) keep named secrets in the system keyring under the service `Opsis` as `key:<name>`; the Linux keyring blocks, so it is read off the async runtime as `off_runtime` does (`crates/capture/src/take.rs`).
- The app database: `Database` (`crates/store/src/db.rs`) opens `opsis.db` (`DB_FILE`) in the data folder with the app's schema (`crates/store/schema.sql`), stores the schema's hash in a `meta` table and replaces a database made for another schema; queries are checked by `sqlx` through `bun run db:prepare`.
- `Keyring` uses nothing else of `opsis-store`, and only `StoreError`'s `Settings` variant ties `db.rs` to the app; the rest of `opsis-store` (settings, fonts, projects, secrets) pulls in `opsis-types`, `specta` and `garde`.

**Work.**

1. **`opsis-persistence`.** A new crate (`crates/persistence`) takes `Keyring` and `KeyError` from `opsis-store`, gains `Keyring::clear_key` (shared with `plans/voice-and-captions.md`), and takes the generic half of `db.rs` as a database that opens a file with the schema it is given: the connection settings, the `meta` table it creates itself with the schema's hash, and replacing a database made for another schema. It has its own error enum. `opsis-store` keeps the app's schema, settings, fonts, projects and secrets over it, and `opsis-capture`, `opsis-session`, `opsis-ui` and `opsis-fonts` take `Keyring` from it.
2. **Model service.** `opsis-agent set` takes a server on this computer (any OpenAI-compatible address, such as Ollama, llama.cpp or LM Studio), Anthropic, OpenAI, OpenRouter, or any OpenAI-compatible address, with a model name and a key; `opsis-agent set --from-env` reads the same from its environment. The agent stores them itself: the key in the keyring as `assistant:<service>`, the rest in its database.
   - Settings → Assistant offers it as an optional one-time setup: the app runs `opsis-agent set --from-env` with the values in that one child's environment. Skipping it is fine; `opsis-agent set` in a terminal does the same.
3. **Its database.** `agent.db`, a second SQLite database beside `opsis.db` in the data folder, with its own schema (`crates/agent/schema.sql`), opened through `opsis-persistence`, holds the agent's settings and sessions. `bun run db:prepare` checks the queries of both databases. No JSONL session files.
4. **Tools.**
   - `read`: text files, and images as pictures for models that take them.
   - `write`, `edit`: whole-file writes and exact replacements.
   - `glob`, `grep`: on ripgrep's crates (`ignore`, `grep`).
   - `shell`: runs a command, in the background when asked, and sends the result back when it ends.
   - Subagents: another session of the same agent.
   - Opsis's tools come through `opsis-mcp`, like any agent's.
   - No web tools.
5. **Modes.** **Ask** (the default): `read`, `glob`, `grep`, `wait_run` and shell commands that only read (`ls`, `cat`, `rg`, `git status`…, with no redirection or chaining into anything else) run without asking; everything else asks. **Full**: nothing asks. Each request offers allow once, allow always and reject.
6. **The loop.** Prompt, stream the answer as `session/update`, ask permission, run the tool, repeat; `session/cancel` stops the turn and any running script; sessions are saved in `agent.db` so `session/load` replays them.
7. **Tests.** Against a model server mocked with `wiremock`: a turn with a tool call, cancel mid-turn, load; `set --from-env` puts the key in the keyring and never in `agent.db`; `agent.db` and `opsis.db` open side by side and stay separate; the read-only command check as a property test over generated commands (one that writes or chains is never read-only).

**Decided.**

- The default agent, in place of fx; not a special preset.
- Its own program, so a crash ends only the agent.
- It sets its own model service and key (`opsis-agent set`); the app's setup is optional and one time.
- It links `opsis-persistence` only: the keyring code moved out of `opsis-store`, and a second database `agent.db` beside `opsis.db`.
- `rig-core` for model requests.
- Ask and Full modes; read-only shell commands never ask.
- No web tools.

## 6. The `opsis` module guide

One guide (`crates/mcp/guide.md`, included in `opsis-mcp`) for every agent, sent as the MCP server's `instructions` (section 2) and in `opsis-agent`'s system prompt. `opsis.lua` has no comments (the no-comments rule), so the guide is the module's only documentation.

**Work.**

1. **What it says.**
   - Change the project only through Lua (`run_lua`), never by writing the project file: Rust watches that file, and a change it didn't make asks the disk-change question, skips the undo batch and skips the checks.
   - Every other file in the project folder (scripts in `scripts/`, assets, notes) can be read and written directly.
   - Each script run is one undo step, and a failed script undoes itself.
   - Start runs and return (`opsis.run.start`), then wait with `wait_run`; runs outlive the script that started them.
   - Look up one node type at a time (`opsis.catalog[type]`); scripts have a 64 MiB memory limit (`MEMORY_LIMIT`).
   - Exports land in `.opsis/out`; read them as files.
2. **Test.** Fails when a function of the `opsis` module isn't in the guide, like the Local API's coverage test (`script/tests/module.rs`).

## 7. Logs

Like T3 Code: a raw log per process beside an event log with projections in the database.

**Work.**

1. **Request log.** Every agent connection writes its ACP traffic, its tool calls and, for `opsis-agent`, its model requests and answers as newline-delimited JSON in `<data folder>/logs/assistant/`, rotated by size, keeping a fixed number of files. Keys and authorization headers are never written.
2. **Event log in the app database** (`crates/store/schema.sql`, queries through `sqlx`, `bun run db:prepare`):
   - `assistant_events`: append-only, with a sequence, the session, the project, the agent, the event type, the time and the payload as JSON.
   - Projections built from it: `assistant_sessions`, `assistant_messages` (with their text for search), `assistant_turns`, and `assistant_requests` (permission and sign-in requests, with their answer and when).
3. **Settings → Assistant** lists sessions and their requests newest first, opens a session's log file, and has **Clear log**.
4. **Tests.** Events project into the same sessions and messages after a restart; nothing secret reaches either log.

## 8. Build order

1. **`opsis-mcp`, the Assistant switch and headless.** From here on any agent run by the user (Claude Code in a terminal) drives Opsis through `opsis-mcp`.
2. **`opsis-persistence`, then `opsis-agent`**, with `agent.db` and the one-time setup. It needs nothing from the Agent panel: `opsis-mcp` and the guide come from phase 1, and its tests drive it as an ACP client would.
3. **The Agent panel**: the event log, agent presets and detection, sessions, permissions, the panel, run-finished messages and session restore.

When each part is built, its design moves into a new `agent-docs/assistant.md` (listed in `AGENTS.md` §8); `api.md` says the socket is internal; `app-state.md` names `agent.db`; `AGENTS.md`'s crate table gains `opsis-persistence`, `opsis-mcp` and `opsis-agent`, and its generated-code table the registry snapshot; `packaging.md` lists the two new programs beside `opsis-renderer`; `editor.md` describes the right-hand tab group. This plan is deleted once everything in sections 1–7 is built.

## 9. Deferred, not finalized

Tools that would let an agent read the user's site the way Opsis records it. They are not built yet, and their design isn't settled.

- **Page outline.** Opsis opens the site in its own recording browser with the project's Open browser settings and saved sign-in, and returns its accessibility tree (`Accessibility.getFullAXTree`) cut to landmarks and named controls, each with a ref. `opsis-agent` has no web tools, and an outside agent's own browser doesn't have the project's sign-in.
- **Try the actions.** Plays a chain as a check take with no recording and reports targets that didn't resolve, with what was visible.
- **Walk the site.** One action at a time from a fresh outline, keeping only the actions that worked; it clicks the real site, so it can create accounts or data.

## 10. Risks

- **The crates move fast.** `agent-client-protocol` has had several major versions in months (projects pin 2.2, 3.6 and 5.4 today), and `rig-core` breaks its API every two to three weeks (0.30 on 3 Feb 2026 to 0.46 on 31 Oct 2026). Both ends of every Opsis session use the same pinned ACP crate, so `opsis-agent` and the app never disagree.
- **A dialog headless mode doesn't answer hangs it.** Every modal dialog needs its headless answer, and a test that opens each one headless.
- **Agents that refuse stdio tool servers get no Opsis tools.** ACP requires stdio, so this is an agent's bug.
- **The read-only shell check is a list.** A command it doesn't know asks, which is the safe side.

## Sources

- `agent-client-protocol` 5.4.0 on docs.rs (Apache-2.0; `Client`, `AcpAgent`, `process` and `stdio` features), checked 2026-11-20.
- ACP session setup (`session/new` with `mcpServers`): https://agentclientprotocol.com/protocol/v1/session-setup, checked 2026-11-20.
- rmcp: https://github.com/modelcontextprotocol/rust-sdk, source read at tag `rmcp-v3.11.0` (commit `165f8fd5`), `crates/rmcp/src/transport/streamable_http_server/tower.rs`, checked 2026-11-21.
- MCP `2026-07-28` Streamable HTTP (no protocol sessions; servers must validate `Origin`, should bind to localhost and should authenticate): https://modelcontextprotocol.io/specification/2026-07-28/basic/transports, checked 2026-11-21.
- `rig-core` 0.46.0 on crates.io (MIT; features and release dates), and https://github.com/0xPlaygrounds/rig, checked 2026-11-20.
- ACP registry: https://github.com/agentclientprotocol/registry, checked 2026-11-20.
- T3 Code's own data folder (`statev2.sqlite`: an `orchestration_v2_events` table with projection tables; rotated `*.trace.ndjson` files in `logs/`), read 2026-11-21.
