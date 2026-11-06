# Local API

The HTTP API scripts and agents use to read, change and run the open project. Paths are relative to `crates/` unless they name another folder. The document it edits is in [document.md](document.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                | Where                                                                            |
| ---------------------- | -------------------------------------------------------------------------------- |
| Routes                 | `api/src/routes.rs` → `ApiHost`                                                  |
| Host implementation    | `session/src/api.rs`                                                             |
| Socket address         | `api/src/address.rs` → `api_address`                                             |
| OpenAPI document       | `api/src/openapi.rs`                                                             |
| Validation             | `check_workflow`, `workflow_problems`, `workflow_problems_with`, `file_problems` |
| Editor side of `/open` | `useApiHost` (`OpenRequested`)                                                   |

## Rules

- **The API is off by default; Settings → API switches it** (`settings.api.enabled`, global and persistent, shown as an API badge in every app bar).
- **`crates/api` serves plain HTTP with `axum` on `<data folder>/api.sock`, or the absolute path in `OPSIS_API_SOCKET`**, with mode 0600.
  - A stale socket from a crash is replaced, a live one from another copy of the app is left alone and reported, and anything that isn't a socket is never deleted.
- **Socket paths are capped at 100 bytes (`api_address`)**, below the OS limits (107 on Linux, 103 on macOS): a longer or relative path is refused before listening, and Settings → API says to set `OPSIS_API_SOCKET` to a shorter full path. There is no automatic fallback location.
- **On Windows the variable is ignored and the API uses the named pipe `\\.\pipe\opsis-api-<user>`, never a TCP port.**
- **Without an open project it answers `/openapi.json`, `/status`, `/nodes`, `/nodes/{type}`, `/validate` and `/open`.**
- **Under `/project`, every request works on the document in Rust through `ApiHost`**: an edit is one `WorkflowEdit`, so one undo step, autosaved and shown in every window, whether or not an editor window is open.
- **Rust checks bodies, node types, connections and settings before editing, and adds the problems to every answer**: the editor's own, from the run results and file checks (`ApiHost::problems`).
- **`/open` opens the project in Rust and asks the main window to show it (`OpenRequested`)**; runs start from the document like the editor's, and run events also stream on `/project/run/events`.
- **Every route is documented in `openapi.rs`** (from the `schemars` schemas of the bodies). A test keeps the document under 100 KB and checks every route is in it; node settings schemas come from `/nodes/{type}`.

## Deliberate limits

- **The Windows named pipe has no current-user-only ACL.**
