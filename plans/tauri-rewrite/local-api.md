# Local API in Rust

The Local API keeps its routes, addresses and rules, served by Rust. Everything that needs node definitions or compile (`/nodes`, `/validate`) is answered in-process; requests under `/project` still go to the editor window, which owns the store and undo.

Crate: `opsis-api` (phase 6, after compile and the runner exist in Rust, so no route is forwarded to TS).

## What it adds

- No Node in the API path, and `/validate` without a call to anything else.
- The Windows pipe can be created for the current user only, which Node couldn't do (`plans/shipping.md`, section 1, Work step 7).

## Fit with Opsis

Already in place:

- `src/main/api.ts`, `api-routes.ts`, `api-editor.ts`, `api-address.ts`: plain HTTP on `<userData>/api.sock` or `OPSIS_API_SOCKET` (mode 0600; a stale socket is replaced, a live one from another copy is left alone and reported, anything that isn't a socket is never deleted; paths capped at 100 bytes), on Windows the named pipe `\\.\pipe\opsis-api-<user>`.
- 23 routes: `GET /openapi.json`, `GET /status`, `GET /nodes`, `GET /nodes/:type`, `POST /validate`, `POST /open`, `GET|PUT /project`, `POST /project/nodes`, `PATCH|DELETE /project/nodes/:id`, `POST /project/edges`, `DELETE /project/edges/:id`, `PATCH /project/settings`, `PUT /project/variables`, `GET /project/problems`, `POST /project/undo`, `POST /project/redo`, `POST /project/save`, `GET|POST /project/run`, `POST /project/run/stop`, and the stream `GET /project/run/events`.
- Requests under `/project` go to the editor window (`apiRequest`/`apiReply`, `src/renderer/editor/api-bridge.ts`), each one store update; main waits up to 20 s for the editor's listener (`apiListening`); `/open` waits until the new editor registers.
- Bodies are zod schemas (`src/shared/api/bodies.ts`); `src/shared/api/openapi.ts` builds the OpenAPI document (about 70 KB).

Missing:

- A Rust HTTP server on a Unix socket and on a named pipe.

## Work

1. **Server.** `axum` 0.8.9: `axum::serve` takes a tokio `UnixListener` on Unix (its `Listener` trait is implemented for `TcpListener` and, on Unix, `UnixListener`). On Windows, accept connections from tokio's named pipe server and serve each with `hyper` 1.12 (`hyper-util`'s connection builder), since axum has no pipe listener; `interprocess` 2.4.7 (local sockets: Unix sockets on Unix, named pipes on Windows, optional tokio support) is the alternative if one code path is preferred.
2. **Same address rules.** Port `apiAddress` (the 100-byte cap and the refusal of relative or long paths), the stale-socket check, mode 0600 and the "never delete what isn't a socket" rule.
3. **The pipe for the current user only.** Create it with `ServerOptions::create_with_security_attributes_raw` and a security descriptor that grants only the current user's SID; keep `reject_remote_clients` (the default) and set `first_pipe_instance` for the first instance. This settles the open question in `plans/shipping.md` section 1 without a token file; verify with a second local account in the Windows CI job as that plan describes.
4. **Routes answered in Rust.** `/status`, `/open`, `/nodes` and `/nodes/:type` (the catalogue and its `schemars` JSON Schema, `compile-in-rust.md`), `/validate` (`check_workflow`), `GET /project` and `GET /project/problems` from the latest workflow the editor sent for compiling (it sends every change), `GET /project/run` and the run events stream from the runner's own state.
5. **Routes sent to the editor.** Everything that edits the project (`PUT /project`, nodes, edges, settings, variables, undo, redo, save, `POST /project/run` so the run starts from the editor's current state) goes to the editor window as today, through a Rust event and a reply command, with the 20 s wait and the reload rule.
6. **OpenAPI document.** Built in Rust from the route table and the `schemars` schemas of the bodies; a test keeps it under about 70 KB and checks every route is documented (today's rule: add an endpoint to the document whenever a route is added).
7. **Tests.** Port the API tests; one test per route through a real socket in a temporary folder.

## Phase 6 as built

- **`opsis-api`.** `axum` on a Unix socket (`ApiServer`, `server.rs`) with today's address rules (`address.rs`: the 100-byte cap, relative paths refused, stale sockets replaced, live ones and non-sockets left alone, mode 0600), and on Windows a tokio named pipe served with `hyper-util`, `first_pipe_instance` and the default remote-client refusal.
- **Checks in Rust.** The routes check bodies (`bodies.rs`), node types, connections (`connection_problem`) and project settings before asking the editor; the editor answers edits with the new workflow and Rust adds the problems (`workflow_problems`), so the editor bridge holds no validation. Requests to the editor are one typed enum, `EditorRequest`, exported to the editor's bindings.
- **OpenAPI.** Built with `schemars` (OpenAPI 3 settings, definitions under `#/components/schemas/`); the TS document's `$ref` prefix `#core/components/schemas/` was wrong and is fixed.
- **Not done here.** The pipe gets the default security descriptor; the current-user-only ACL of step 3 and its check from a second account need the Windows CI job, so they wait for packaging (phase 8).
- **Tests.** Address rules (7) and routes through a real socket (6), including the editor-bound checks.

## Risks

- **Two code paths for transport** (Unix socket, named pipe) need tests on Linux, macOS and Windows CI.
- **Unsafe Windows code.** Building the security descriptor uses the Windows API through `unsafe`; keep it in one small module with a test that reads the pipe's ACL back.

## Sources

- `src/main/api*.ts`, `src/renderer/editor/api-bridge.ts` (read 2026-10-25).
- [axum `serve::Listener`](https://docs.rs/axum/latest/axum/serve/trait.Listener.html), [tokio `named_pipe::ServerOptions`](https://docs.rs/tokio/latest/tokio/net/windows/named_pipe/struct.ServerOptions.html), [interprocess on docs.rs](https://docs.rs/crate/interprocess/latest); crates.io API for `axum`, `hyper`, `interprocess` (2026-10-25).
