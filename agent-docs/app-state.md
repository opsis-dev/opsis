# App state

The app's one database, the settings held in memory, the keyring and the projects folder. Paths are relative to `crates/` unless they name another folder. Per-project view state is in [editor.md](editor.md), the font rows in [fonts.md](fonts.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                   | Where                                                                                           |
| ------------------------- | ----------------------------------------------------------------------------------------------- |
| Schema                    | `store/schema.sql`                                                                              |
| Queries                   | `store` (`sqlx::query!`, `query_as!`, `query_scalar!`); offline cache `.sqlx/`                  |
| Offline build             | `.cargo/config.toml` (`SQLX_OFFLINE=true`)                                                      |
| Settings in memory        | `desktop/src/state.rs` → `AppState::settings`, `AppState::patch_settings`, `RUST_ONLY_SECTIONS` |
| Start                     | `desktop/src/lib.rs` → `open`                                                                   |
| Keyring and sealing       | `store/src/keys.rs` (`keyring-core`)                                                            |
| Project folder suggestion | `project_folder` command                                                                        |

## Rules

### The database

- **App state is one database, `<data folder>/opsis.db`**: SQLite through `sqlx` with the bundled SQLite, WAL, foreign keys on. It holds settings (one row per section), every project ever opened, each project's view state, window bounds and the font library index.
- **Recent projects are a query**: newest first, folders that still hold a project, first 20.
- **Only Rust opens it (`AppState::db()`, a pool, so every query is async); windows go through commands.**
- **The tables are plain SQL with snake_case columns**, embedded by the store and run on an empty database.
- **Every query is a checked macro against that schema.** After changing the schema or a query, run `bun run db:prepare` (it needs `sqlx-cli` built with SQLite).
- **There are no migrations: the `meta` table holds the sha256 of the schema SQL, and a database made for another schema is deleted and made again at startup.**
- **Projects stay plain folders: nothing about a project's contents goes in the database.**

### Settings

- **Settings also live in memory in `AppState`**, read with `settings()` and written only through `patch_settings`, one patch at a time, so handlers that can't wait (the close request) read them without a query.
- **`settings.engine` is written only by Rust** (`RUST_ONLY_SECTIONS`, refused by `patch_settings`) ([engine.md](engine.md)).
- **Setup spawns the start (`open`)**: it opens the database, registers every piece of state at once and then opens the main window, so no command runs before the state exists; a failed start logs why and exits.

### Secrets

- **Secrets use the OS keyring** (`keyring-core` with the Apple, Windows and Secret Service stores): sign-ins are sealed with AES-256-GCM under one key kept there, and saving refuses when the platform has no secure store.
- **The keyring is made in setup itself**, because the Linux Secret Service store blocks inside `zbus` and panics on a runtime thread.

### Projects folder

- **New projects are suggested in `settings.projects.lastDir` (the last place one was created) or else the default folder**: `settings.projects.defaultDir`, empty meaning `~/OpsisProjects` on every OS. Changing the default clears `lastDir`.
- **Open project starts in the default folder.**
- **The New project dialog shows the folder it will create (`project_folder`)**; started from a template card, it shows only that template with Change.

## Deliberate limits

- **There are no migrations before the first release**; a schema change wipes the app database.
