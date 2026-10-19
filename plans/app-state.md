# App state database

One SQLite database for Opsis's own state, instead of the JSON files in the app's data folder and the renderer's local storage. It holds what belongs to the app on this computer: settings, the projects opened, window positions, the font library index, each project's view state, service keys and the assistant's request log. Projects stay plain folders, `.opsis/` included.

Status: **Database and schema** and **Per-project view state** are built (not committed). **Encrypted values** has its column type built and tested (`src/main/db/crypto.ts`), unused and marked `// TODO: To be used later`; the `secrets` table comes with the first plan that stores a key. **Assistant request log** is not started; it adds its table with `plans/ai-assist.md`.

## How to read this plan

- One feature, built in the order of **Work**; each step is checked before the next starts.
- Sections: **What it adds**, **Fit with Opsis** (Already in place / Missing, every file and function named was checked in the repo), **Stack (decided)**, **Work**, **Licence**, **Risks**, **Sources**.
- Other plans store data here: the request log in `plans/ai-assist.md`, the service keys in `plans/voice-and-captions.md` and `plans/ai-assist.md`.

## Conventions

These restate rules from `AGENTS.md`; when the two disagree, `AGENTS.md` wins.

- **Main process only.** Only `src/main` opens the database. Windows read and write through IPC, like `getSettings` / `patchSettings` today. Nothing from `node:sqlite` or `drizzle-orm/node-sqlite` in `src/shared`, `src/video` or `src/renderer`; the schema (`drizzle-orm/sqlite-core` tables) lives in `src/main/db/`.
- **Projects stay files.** `project.opsis.json`, `assets/`, `secrets/` and `.opsis/` (cache, exports, traces, `last-run.json`, the stills index) stay in the project folder, so a project can be copied, put in git, opened on another computer, and edited by an agent or a text editor; the file watcher (`watchProject`, `src/main/projects.ts`) depends on that.
- **No old formats.** Nothing has been released: no migrations, no import of the old JSON files or local storage, no backups. A schema change replaces the dev database (see **Schema without migrations**).
- **Tools run on Node, never on Bun's runtime.** Bun is the package manager only; no command runs with `--bun`. `drizzle-kit` runs through a `package.json` script on Node.
- **Checks.** `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`, the no-comments check from `AGENTS.md`, and a run of the app: start, change settings in the Settings window, open projects, pop out panels, restart, and see everything restored. Don't commit.

## Summary

| Part                   | What users get                                                                                       | Size |
| ---------------------- | ---------------------------------------------------------------------------------------------------- | ---- |
| Database and schema    | Nothing visible; one file for the app's state instead of five JSON files and the browser's storage   | M    |
| Per-project view state | Panel sizes, tabs, preview shape, minimap and Timeline settings kept by main, not in browser storage | M    |
| Encrypted values       | Service keys stored once for the app, encrypted                                                      | S    |
| Assistant request log  | What was sent to a model and what came back, viewable later (built with `plans/ai-assist.md`)        | S    |

## Database and schema

### What it adds

`<userData>/opsis.db`, opened once by main at startup, in WAL mode so reads never wait on a write, with a typed schema.

### Fit with Opsis

Already in place:

- App state in JSON files under `<userData>`: `settings.json` (`src/main/settings.ts`: `readSettings`, `patchSettings`, `onSettingsChange`), `recent-projects.json` (`src/main/projects.ts`: `listRecent`, `touchRecent`, capped at `MAX_RECENT` = 20 and filtered to folders that still hold a project file), `panel-windows.json` (`src/main/panel-windows.ts`) and `settings-window.json` (`src/main/settings-window.ts`), both through `boundsStore` in `src/main/window-bounds.ts`, and the font library index `library.json` (`manifestPath` in `src/main/fonts.ts`, which keeps entries it can't read).
- Every JSON file is already written atomically (`writeJson` → `atomicWrite`) and broken files are moved aside (`readJsonKeepingBroken`); settings changes already reach every window (`patchSettings`, broadcast as `settingsChanged`). The database doesn't fix a bug here: it replaces five file formats with one and lets main answer queries (recent projects, per-project view state) without reading whole files.
- Settings already go through one IPC path, so moving their storage changes no renderer code.
- Electron 44.9.0 (`node_modules/electron/package.json`), whose Node line includes the built-in `node:sqlite` (`DatabaseSync`).

Missing:

- `drizzle-orm`, `drizzle-kit`, a schema, and a place that opens the database.

### Stack (decided)

- **`drizzle-orm` and `drizzle-kit` at exactly `1.0.0-rc.4`.** The stable line (`latest`: `drizzle-orm` 0.45.4, `drizzle-kit` 0.31.11 when checked) has no `node:sqlite` driver; the 1.0 release candidates have `drizzle-orm/node-sqlite`. This is an exception to the "latest release" rule, pinned exactly like Remotion and TypeScript, until 1.0 is stable: `bun add --exact drizzle-orm@1.0.0-rc.4` and `bun add -d --exact drizzle-kit@1.0.0-rc.4`. Add the exception to the dependency rules in `AGENTS.md`.
- **Driver:** `drizzle({ client: new DatabaseSync(file) })` from `drizzle-orm/node-sqlite`, with the synchronous query methods (`.all()`, `.get()`, `.run()`). No native module, so nothing to rebuild for Electron or unpack from the app archive.
- **Fallback** if `node:sqlite` turns out unusable in Electron 44.9.0: `better-sqlite3` with the same Drizzle version (a native module, rebuilt for Electron and staged in `forge.config.ts` `packageAfterCopy` like playwright-core). Decide only if step 1 of **Work** fails.

### Schema without migrations

- `drizzle-kit export` prints the schema as plain DDL. `bun run db:schema` writes it to `src/main/db/schema.generated.sql` (checked in, in the generated-files table of `AGENTS.md`: never hand-edited).
- Main imports it as text (`schema.generated.sql?raw`), so Vite bundles it into `main.cjs`; nothing is staged as a resource.
- A one-row `meta` table holds the sha256 of that DDL. At startup: no file, or no `meta` row → create the tables and write the hash. Another hash → close, delete the database and its `-wal` / `-shm` files, and create it again. Only development databases exist, so nothing is lost that matters.
- A test fails when `schema.generated.sql` differs from what `drizzle-kit export` prints now, so the file can't go stale.
- Real migrations (`drizzle-kit generate` and `migrate()`) come with the first release, as a separate step.

### Work

1. **Check the runtime.** Under Electron's Node (`ELECTRON_RUN_AS_NODE=1`): `new DatabaseSync(":memory:")`, a table, an insert, a select, a transaction that rolls back. Done: Electron 44.9.0 runs Node 24.22.0 with SQLite 3.53.4, and the rollback held, so no fallback.
2. **Dependencies** as in **Stack**.
3. **Schema.** `src/main/db/schema.ts` with `sqliteTable`s (below). Integer primary keys with a unique text column where the natural key is text (a project folder, a setting section), not TEXT primary keys (see **Risks**, #6165).
4. **Config and DDL.** `drizzle.config.ts` at the repo root (`dialect: "sqlite"`, schema `src/main/db/schema.ts`). The `db:schema` script and the staleness test from **Schema without migrations**.
5. **Opening.** `src/main/db/index.ts`: `openDatabase()` at `app.whenReady()` before anything reads settings; `PRAGMA journal_mode = WAL`, `PRAGMA foreign_keys = ON`, `PRAGMA busy_timeout`; the hash check; close on `will-quit`. Tests open it with `":memory:"`.
6. **Move each store,** deleting its JSON code: settings, projects (recents become a query), window bounds, the font library index.
7. **Packaging.** Nothing native to stage. Check the packaged app finds the DDL and that the database lands in `<userData>`. Not done yet: checked only in development (`bun start`).
8. **Tests** (vitest, in-memory database): the DDL creates every table; a changed hash recreates the database; a failed transaction leaves nothing behind (#1723); each moved store round-trips.

### Tables

| Table           | Columns (short)                                                                                            | Replaces                                                      |
| --------------- | ---------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------- |
| `meta`          | `id` (always 1), `schemaHash`                                                                              | new                                                           |
| `settings`      | `section` (unique), `value` (JSON text)                                                                    | `settings.json`                                               |
| `projects`      | `id`, `dir` (unique), `name`, `openedAt`                                                                   | `recent-projects.json`                                        |
| `view_state`    | `projectId` (not null, references `projects`, cascade), `key`, `value` (JSON), unique (`projectId`, `key`) | `readPref` / `writePref` and `layoutStorage` in local storage |
| `window_bounds` | `window` (unique: `settings`, `panel:preview`…), `x`, `y`, `width`, `height`                               | `panel-windows.json`, `settings-window.json`                  |
| `fonts`         | `fontId` (unique), `def` (JSON `FontFamilyDef`, kept as text when it no longer parses)                     | `library.json` (font files stay in `<userData>/fonts`)        |
| `secrets`       | `scope` (`app`), `name` (unique per scope), `value` (`encryptedText`)                                      | new: service keys for voice and the assistant                 |
| `assist_log`    | `id`, `at`, `projectId`, `task`, `service`, `model`, `request` (JSON), `response` (JSON), `error`, `ms`    | new                                                           |

- **`projects` keeps every project opened**, never just the last 20: `view_state` and `assist_log` hang off it, and pruning it would drop their rows. Recent projects are a query: newest `openedAt` first, skipping folders without a project file, first 20 (`MAX_RECENT`).
- No app-wide view rows: `view_state.projectId` is never null (SQLite treats every NULL as distinct, so `unique(projectId, key)` couldn't stop app-wide rows repeating). The one app-wide value in local storage, `opsis:setup-skipped`, became the setting `app.setupSkipped`, and view prefs changed while no project is open are kept in memory only.
- `secrets` and `assist_log` are added with the plans that use them, not before.

## Per-project view state

### What it adds

Panel sizes, the active tab, Fast/Exact preview, preview shape, minimap, mixer folding and the Timeline's ruler and snapping move from the renderer's local storage into the database, so the app's state is in one place, main can read it, and clearing the browser storage loses nothing. Windows already share it today (one origin, and `useViewOptions` is mirrored over `opsis-sync` in `src/renderer/app/sync.ts`); that stays as it is.

### Fit with Opsis

Already in place:

- `src/renderer/store/view-prefs.ts` (`readPref`, `writePref`, `setViewProject`) is the single read/write path for the per-project preferences (`dockTab`, `previewQuality`, `previewShape`, `minimap`, `mixerCollapsed`, `rulerMode`, `snapping`), and `loadViewState` (`src/renderer/editor/view-state.ts`) loads them when a project opens (`open-project.ts`, and `applyState` in `sync.ts` for panel windows).
- Panel layouts go through `layoutStorage` (`src/renderer/editor/panels.ts`) passed to `useDefaultLayout`, with ids that already name the project folder (`opsis:editor-columns:<dir>`, `opsis:editor-rows:<dir>`).
- `src/renderer/app/startup-flags.ts` keeps `opsis:setup-skipped` in local storage (app-wide).

Missing:

- A way to get a project's view state to the renderer before the stores read it, because `readPref` and `layoutStorage.getItem` are synchronous, and a way to save and share changes.

### Work

1. `openProject` returns the project's view state (`OpenProject.view`), so it arrives with the workflow and needs no second call; `loadViewState(dir, view)` puts it in the renderer's cache (`view-prefs.ts`) before the stores read it, and the editor renders after that as before. `readPref` and `layoutStorage.getItem` read the cache synchronously.
2. `writePref` updates the cache and sends `setViewPref(dir, key, value)`; main saves it and broadcasts `viewPrefChanged`, so every window's cache follows (`watchViewPrefs` in `main.tsx`). Panel windows get the editor's cache in the sync `state` message.
3. Layout ids stop carrying the folder (`opsis:editor-columns`, `opsis:editor-rows`), since the row already belongs to the project; the editor remounts per project (`key={dir}`).
4. A project created in a folder that held one before starts with no saved view (`forgetViewState`).
5. `opsis:setup-skipped` becomes the setting `app.setupSkipped`; the renderer keeps nothing in local storage (`session-storage.ts` keeps the once-per-launch flag).
6. Window bounds use `window_bounds`, keyed by the window's key in `openRememberedWindow` (`settings`, `panel:<name>`), instead of the two JSON files.
7. `AGENTS.md` names the database instead of local storage and the JSON files, and has an **App state is one database** section.

## Encrypted values

### What it adds

`encryptedText()`, a Drizzle column type that seals values before they reach the database, for the service keys the voice and assistant plans need.

### Fit with Opsis

- Opsis already encrypts project sign-ins with `safeStorage` (`src/main/secrets.ts`, files in the project's `secrets/`), and refuses to save them when `safeStorage.isEncryptionAvailable()` is false.

### Work

1. `src/main/db/crypto.ts`: `encryptedText(name)` is a `customType<{ data: string; driverData: string }>` whose `toDriver` is `safeStorage.encryptString(value)` in base64 and whose `fromDriver` decrypts it. No key of our own: `safeStorage` already keeps its key in the OS keychain, so a copied database alone reveals nothing, and a second layer would add a key file without adding protection.
2. Refuse to store a value when `safeStorage.isEncryptionAvailable()` is false or, on Linux, `safeStorage.getSelectedStorageBackend()` is `basic_text`, and say why on the Settings page that asked for it.
3. A value that can't be decrypted on this computer reports "This key can't be read on this computer. Enter it again." and never crashes.
4. Project sign-ins stay in the project's `secrets/` folder: they belong to the project.
5. Tests (with `safeStorage` mocked): round trip, a value that fails to decrypt reports cleanly, `basic_text` is refused.

## Assistant request log

Built together with `plans/ai-assist.md`, which calls it.

### What it adds

Each request the assistant sends (local or online) and its answer, stored in `assist_log` with the time, project, task, service and model, so the user can see what was sent. Keys and the request headers are never logged.

### Work

1. `logAssist(entry)` in `src/main/db/assist-log.ts`, called by the assistant's client after each request.
2. A list in Settings → Assistant (newest first, with what was sent and what came back), and **Clear log**.
3. Keep the last 1,000 entries or 30 days, whichever is smaller, pruned at startup.

## Licence

`drizzle-orm` and `drizzle-kit`: Apache-2.0; add them to `THIRD_PARTY_NOTICES.md`. SQLite (inside Node): public domain.

## Risks

- **Release candidate.** No stable date for Drizzle 1.0 when checked: `rc.4` came out on 2026-06-27, while 0.45.x kept getting patches (0.45.4 on 2026-10-08). Breaking changes between RCs are possible; moving to the stable release is a separate step that re-runs the tests.
- **#6165:** drizzle-kit v1 leaves `NOT NULL` off SQLite TEXT primary keys (open against `rc.4`). Avoided by integer keys (step 3); review the exported DDL anyway.
- **#5515:** drizzle-kit can't find `node:sqlite` under Bun's runtime. Not a risk here: nothing runs under `--bun`.
- **#1723:** reports of transactions not rolling back (old, mostly other drivers). Covered by the rollback test.
- **`node:sqlite` in the main bundle.** Forge's Vite plugin keeps Node's built-ins external using the build machine's `builtinModules`; it lists `node:sqlite` on Node 26. Check CI's Node does too (step 7).
- **Schema changes wipe dev state.** Until real migrations exist, changing the schema resets settings and view state on every development machine.

## Sources

- [Drizzle: Node SQLite driver](https://orm.drizzle.team/docs/sqlite/connect-node-sqlite), [drizzle-kit export](https://orm.drizzle.team/docs/drizzle-kit-export) and [v1 roadmap](https://orm.drizzle.team/roadmap) (checked 2026-10-17)
- npm metadata of `drizzle-orm` and `drizzle-kit`: dist-tags (`latest` 0.45.4 / 0.31.11, `rc` 1.0.0-rc.4), release dates, `exports` of 0.45.4 (no `./node-sqlite`) and 1.0.0-rc.4 (`./node-sqlite`, `./node-sqlite/driver`) (checked 2026-10-19)
- Drizzle issues [#2648](https://github.com/drizzle-team/drizzle-orm/issues/2648), [#6165](https://github.com/drizzle-team/drizzle-orm/issues/6165), [#5515](https://github.com/drizzle-team/drizzle-orm/issues/5515), [#1723](https://github.com/drizzle-team/drizzle-orm/issues/1723) and advisory [GHSA-gpj5-g38j-94v9](https://github.com/advisories/GHSA-gpj5-g38j-94v9) (fixed in 1.0.0-beta.20, released before rc.1, so not in rc.4) (checked 2026-10-17)
- [Electron PR #47757](https://github.com/electron/electron/pull/47757) (missing SQLite built-in fixed) and [issue #45532](https://github.com/electron/electron/issues/45532) (checked 2026-10-17)
- [Electron `safeStorage`](https://www.electronjs.org/docs/latest/api/safe-storage) (`getSelectedStorageBackend` on Linux)
