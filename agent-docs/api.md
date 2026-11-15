# Local API

The HTTP API scripts and agents use to read, change and run the open project. Paths are relative to `crates/` unless they name another folder. The document it edits is in [document.md](document.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                | Where                                                                                                                                     |
| ---------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| Routes                 | `api/src/routes.rs` → `routes`, `ApiHost`, `HostError`, `MAX_BODY`, `edited_reply`                                                        |
| Request bodies         | `api/src/bodies.rs` → `parse_body` and the body types                                                                                     |
| Server and socket      | `api/src/server.rs` → `ApiServer::set_enabled`; `api/src/address.rs` → `api_address`                                                      |
| Host implementation    | `session/src/api.rs` → `Api`, `AppApi`, `follow_setting`, `document_failure`, `start_failure`                                             |
| Run event stream       | `session/src/api.rs` → `run_event`, `RUN_EVENTS_KEPT`                                                                                     |
| OpenAPI document       | `api/src/openapi.rs`; tests in `api/tests/routes.rs`                                                                                      |
| Validation             | `compile/src/check.rs` → `check_workflow`, `workflow_problems`, `workflow_problems_with`; `session/src/node_results.rs` → `file_problems` |
| Editor side of `/open` | `session/src/api.rs` emits `OpenRequested` to `MAIN_WINDOW`; `ui/src/ui.rs` shows the project                                             |
| Script routes          | `api/src/routes.rs` → `run_script`, `stop_script`, `script_status`; `session/src/api.rs` → `scripts_allowed`, `script_failure`            |

## Rules

- **The API is off by default; Settings → API switches it** (`settings.api.enabled`, global and persistent, shown as an API badge in every app bar).
- **`crates/api` serves plain HTTP with `axum` on `<data folder>/api.sock`, or the absolute path in `OPSIS_API_SOCKET`**, with mode 0600.
  - A stale socket from a crash is replaced, a live one from another copy of the app is left alone and reported, and anything that isn't a socket is never deleted.
- **Socket paths are capped at 100 bytes (`api_address`)**, below the OS limits (107 on Linux, 103 on macOS): a longer or relative path is refused before listening, and Settings → API says to set `OPSIS_API_SOCKET` to a shorter full path. There is no automatic fallback location.
- **On Windows the variable is ignored and the API uses the named pipe `\\.\pipe\opsis-api-<user>`, never a TCP port.**
- **Only the `/project` routes need an open project** (`routes`): without one they answer 409, except `/project/run/stop`, `/project/run/events` and `/project/script/stop`, which never look for a project.
- **A route that fails answers `{"error": …}`.**
  - The routes check what they can themselves: a body that isn't valid JSON or UTF-8 is 400, a node type `POST /project/nodes` doesn't know 400 and one `GET /nodes/{type}` doesn't know 404, a refused connection 400 or 404 (`connection_problem`).
  - The host chooses the status of its own failures (`HostError`, mapped by `document_failure` and `start_failure`).
  - Unknown routes answer 404 pointing to `/openapi.json`, and a body over `MAX_BODY` is refused before any route reads it.
- **Under `/project`, every request works on the document in Rust through `ApiHost`**: an edit is one `WorkflowEdit`, so one undo step, autosaved and shown in every window, whether or not an editor window is open.
- **Rust checks bodies, node types, connections and settings before editing.**
- **Every edit answer and `GET /project/problems` carry the editor's problems** (`ApiHost::problems`): the workflow's checks against the run results, plus the file checks.
  - The edit answers (`edited_reply`) are those of the node, edge, settings and variables routes, of an accepted `PUT /project`, and of undo, redo and save. `GET /project`, `/open` and the run routes carry none.
- **`/validate` checks a body without the open project** (`check_workflow`): it answers the workflow's checks with no run results and no file checks.
  - A `PUT /project` whose body isn't a project answers 400 with the same check's problem, which names the JSON path that failed.
- **`/open` opens the project in Rust and asks the main window to show it (`OpenRequested`).**
  - It takes an absolute folder (400 otherwise) that holds a project file (404 otherwise); a project file that can't be read answers 422 (`AppApi::open`).
  - The project opens even when telling the window fails; that failure is only logged.
- **Runs start from the document like the editor's.**
  - A run answers 409 while another runs and while the project's version is unchosen, as well as with no project (`StartError`, `session/src/run.rs`).
- **Run events also stream on `/project/run/events`** as server-sent events, one `data:` line of JSON per run event, sent only while a client follows them (`run_event`).
- **The script routes need a second switch, Settings → API → Run Lua scripts (`settings.api.scripts`, off by default)**, because with it the socket also reads and writes the project's files and drives runs; the switch is disabled while the API is off.
  - `POST /project/script` takes `{ "source": … }` or `{ "file": "scripts/name.lua" }`, runs it as the Script panel would ([scripting.md](scripting.md)) and answers when it ends with the outcome and the editor's problems; `POST /project/script/stop` stops it and `GET /project/script` says whether one runs.
  - They answer 403 while the switch is off, 409 with no project or while a script runs, 400 for a body that is neither form, and 200 with `ok: false` and the failure for a script that failed.
  - A client that goes away doesn't stop its script; the stop route does.
- **Every route is documented in `openapi.rs`** (from the `schemars` schemas of the bodies); node settings schemas come from `/nodes/{type}`.
  - `documents_every_route_it_serves` checks a list of routes written in the test, not the router, so a new route goes in that list too. Another test keeps the document under 100 KB.
