# App state

The app's one database, the settings held in memory, the keyring and the projects folder. Paths are relative to `crates/` unless they name another folder. Per-project view state is in [editor.md](editor.md), the font rows in [fonts.md](fonts.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                   | Where                                                                                                                                                            |
| ------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Schema                    | `store/schema.sql`                                                                                                                                               |
| Database file and opening | `session/src/paths.rs` → `AppPaths::database` (in `AppPaths::user_data`); `store/src/db.rs` → `Database::open` over `persistence/src/db.rs` → `open`, `META_SQL` |
| Projects and view state   | `store/src/projects.rs` → `list_recent`, `read_view_state`, `write_view_state`, `forget_view_state`                                                              |
| Queries                   | `store/src` (`sqlx::query!`, `query_as!`, `query_scalar!`); offline cache `.sqlx/` at the repository root                                                        |
| Offline build             | `.cargo/config.toml` at the repository root (`SQLX_OFFLINE=true`)                                                                                                |
| Settings in memory        | `session/src/state.rs` → `AppState::settings`, `AppState::patch_settings`; `store/src/settings.rs` → `merged_settings`, `read_settings`                          |
| Settings from the editor  | `session/src/settings.rs` → `patch_settings` (refuses `RUST_ONLY_SECTIONS`, `types/src/settings.rs`)                                                             |
| Start                     | `ui/src/main.rs` → `start`; `session/src/session.rs` → `Session::start`                                                                                          |
| Long work                 | `session/src/state.rs` → `Tasks` (`AppState.tasks`), read by `session/src/exit.rs`                                                                               |
| Keyring and sealing       | `persistence/src/keys.rs` (`keyring-core`) → `Keyring`, `clear_key`; `store/src/secrets.rs`; `session/src/settings.rs` → `ProjectSignIns`                        |
| Project folder suggestion | `project/src/paths.rs` → `project_locations`; `session/src/projects.rs` → `project_folder`, `create_project`                                                     |

## Rules

### The database

- **App state is one database, `<data folder>/opsis.db`**: SQLite through `sqlx` with the bundled SQLite, WAL, foreign keys on. It holds settings (one row per section), every project ever opened, each project's view state (the main window's place among it, [editor.md](editor.md)), the font library index and the assistant's event log ([assistant.md](assistant.md)).
- **Recent projects are read newest first, then Rust keeps the folders that hold a project file (`WORKFLOW_FILE`), up to `MAX_RECENT`** (`store/src/projects.rs` → `list_recent`).
- **View state is written only for a project the database knows**: `write_view_state` saves nothing for a folder never opened, and a project's view state is deleted with its row (`ON DELETE CASCADE`); `create_project` forgets any view state left for its folder before recording it.
- **Only Rust opens it (`AppState::db()`, a pool, so every query is async); windows reach it through the session over the bridge** ([ui.md](ui.md)).
- **The tables are plain SQL with snake_case columns**, embedded by the store and run on an empty database.
- **Every query is a checked macro against that schema.** After changing the schema or a query, run `bun run db:prepare` (it needs `sqlx-cli` built with SQLite).
  - Its throwaway database holds `meta`, the app's schema and `agent.db`'s schema together, since a build checks every crate's queries against one database; the two schemas share no table name.
- **There are no migrations: the `meta` table holds the sha256 of the schema SQL, and a database made for another schema is deleted and made again at startup** (`opsis_persistence::db::open`), its `-wal` and `-shm` files with it.
  - `opsis-persistence` makes `meta` itself (`persistence/meta.sql`) beside the schema it is given, so a schema file holds only its own tables; the scripts' seed (`scripts/lib/user-data.ts`) runs the same file.
- **`opsis-agent` keeps a second database, `<data folder>/agent.db`, through the same opener** (`agent/src/db.rs`, schema `agent/schema.sql`): the agent's settings and sessions stay out of the app's database, and the agent links only `opsis-persistence` ([assistant.md](assistant.md)).
- **Projects stay plain folders: nothing about a project's contents goes in the database.**

### Settings

- **Settings also live in memory in `AppState`**, read with `settings()` and written only through `AppState::patch_settings`, one patch at a time, so handlers that can't wait (the close request) read them without a query.
  - A patch merges field by field into each section it names (`merged_settings`) and must still parse and validate, or nothing changes; it is written to the database before the copy in memory is replaced.
  - At start `read_settings` reads every section; when the whole doesn't parse and validate, every setting falls back to its default, with a warning.
- **`settings.engine` is written only by Rust**: the editor's `patch_settings` (`session/src/settings.rs`) refuses `RUST_ONLY_SECTIONS` ([engine.md](engine.md)).
- **Nothing runs before the state exists**: `start` (`ui/src/main.rs`) makes the keyring and blocks on `Session::start`, which opens the database and builds every part of the session before the Qt event loop starts; a failed start logs why and exits with 1.
- **Long app work is counted in `AppState.tasks`, and the exit dialog lists its labels** (`session/src/exit.rs`).
  - `begin(label)` and `end(label)` nest: a label stays until every `begin` has its `end`.

### Secrets

- **Secrets use the OS keyring** (`keyring-core` with the Apple, Windows and Secret Service stores): sign-ins are sealed with AES-256-GCM under one key kept there, made on the first seal.
  - The sealed values are files in the project's `secrets` folder (`store/src/secrets.rs`), where `ensure_secrets_dir` writes a `.gitignore` of everything but itself when there is none.
  - With no secure store the app still starts: `AppState::keys` holds the error, and reading or saving a sign-in fails with it (`ProjectSignIns`).
- **The keyring is made before the session starts (`start`), and every call to it during a run goes through `off_runtime` (`capture/src/take.rs`) onto a blocking thread**, because the Linux Secret Service store blocks inside `zbus` and panics on a runtime thread.
  - `capture/tests/keyring_sign_in.rs` saves a sign-in through a real Secret Service from a run, so it is `#[ignore]`d and run by hand under its own `dbus-run-session` with `gnome-keyring-daemon --unlock --components=secrets`, and `XDG_RUNTIME_DIR` must be a short path: the daemon's control socket doesn't fit a long one and the store then waits two minutes before giving up.

### Projects folder

- **New projects are suggested in `settings.projects.lastDir` (the last place one was created, while that folder exists) or else the default folder** (`project_locations`): `settings.projects.defaultDir`, empty meaning `~/OpsisProjects` on every OS.
  - `create_project` writes `lastDir`; changing the default clears it (`merged_settings`).
- **Open project starts in the default folder** (`pick_project_dir`).
- **The New project dialog shows the folder it will create (`project_folder`)**; started from a template card, it shows only that template with Change.
