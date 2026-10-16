# App state database

One SQLite database for Demoflow's own state, instead of the JSON files in the app's data folder and the renderer's local storage. It holds what belongs to the app on this computer: settings, recent projects, window positions, the font library index, each project's view state, service keys and the assistant's request log. Projects stay plain folders.

Status: plan only. Nothing here is built yet.

## How to read this plan

- One feature, built in the order of **Work**; each step is checked before the next starts.
- Sections: **What it adds**, **Fit with Demoflow** (Already in place / Missing, every file and function named was checked in the repo), **Stack (decided)**, **Work**, **Licence**, **Risks**, **Sources**.
- Other plans store data here: the request log in `plans/ai-assist.md`, the service keys in `plans/voice-and-captions.md` and `plans/ai-assist.md`.

## Conventions

These restate rules from `AGENTS.md`; when the two disagree, `AGENTS.md` wins.

- **Main process only.** Only `src/main` opens the database. Windows read and write through IPC, like `getSettings` / `patchSettings` today. Nothing from `node:sqlite` or `drizzle-orm/node-sqlite` in `src/shared`, `src/video` or `src/renderer`; the schema itself (`drizzle-orm/sqlite-core` tables) may live in `src/main/db/`.
- **Projects stay files.** `project.demoflow.json`, `assets/`, `secrets/` and `.demoflow/` (cache, renders, traces, `last-run.json`, the stills index) stay in the project folder, so a project can be copied, put in git, opened on another computer, and edited by an agent or a text editor; the file watcher (`watchProject`, `src/main/projects.ts`) depends on that.
- **Tools run on Node, never on Bun's runtime.** Bun is the package manager only; no command runs with `--bun` (`bun --bun x …`). `drizzle-kit` runs through a `package.json` script on Node.
- **Checks.** `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`, the no-comments check from `AGENTS.md`, and a run of the app: start, change settings in the Settings window, open projects, pop out panels, restart, and see everything restored. Don't commit.

## Summary

| Part                    | What users get                                                                                           | Size |
| ----------------------- | -------------------------------------------------------------------------------------------------------- | ---- |
| Database and migrations | Nothing visible; state survives updates and a crash mid-write, and windows never disagree about settings | M    |
| Per-project view state  | Panel sizes, tabs, preview shape, minimap and Timeline settings shared by every window, not per window   | M    |
| Encrypted values        | Service keys stored once for the app, encrypted                                                          | S    |
| Assistant request log   | What was sent to a model and what came back, viewable later                                              | S    |

## Database and migrations

### What it adds

`<userData>/demoflow.db`, opened once by main at startup, in WAL mode so reads never wait on a write. A typed schema and versioned migrations, so a new app version can change the shape without losing anything.

### Fit with Demoflow

Already in place:

- App state in JSON files under `<userData>`: `settings.json` (`src/main/settings.ts`: `readSettings`, `writeSettings`, `patchSettings`, `onSettingsChange`), `recent-projects.json` (`src/main/projects.ts`: `listRecent`, `touchRecent`), `panel-windows.json` (`src/main/panel-windows.ts`: `loadBounds`, `saveBounds`), `settings-window.json` (`src/main/settings-window.ts`: `savedBounds`), and the font library index `library.json` (`manifestPath` in `src/main/fonts.ts`).
- Settings already go through one IPC path (`patchSettings`, broadcast as `settingsChanged`), so moving their storage changes no renderer code.
- Electron 44.7.2 (`node_modules/electron/package.json`), whose Node line includes the built-in `node:sqlite` (`DatabaseSync`).

Missing:

- `drizzle-orm`, `drizzle-kit`, a schema, migrations, and a place that opens the database.

### Stack (decided)

- **`drizzle-orm` and `drizzle-kit` at exactly `1.0.0-rc.4`.** The stable line (`latest`, 0.45.4 when checked) has no `node:sqlite` driver; the 1.0 release candidates have `drizzle-orm/node-sqlite`. This is an exception to the "latest release" rule, pinned exactly like Remotion and TypeScript, until 1.0 is stable: `bun add --exact drizzle-orm@1.0.0-rc.4` and `bun add -d --exact drizzle-kit@1.0.0-rc.4`. Add the exception to the dependency rules in `AGENTS.md`.
- **Driver:** `drizzle({ client: new DatabaseSync(file) })` from `drizzle-orm/node-sqlite`, with the synchronous query methods (`.all()`, `.get()`, `.run()`). No native module, so nothing to rebuild for Electron or unpack from the app archive.
- **Fallback** if `node:sqlite` turns out unusable in Electron 44.7.2: `better-sqlite3` with the same Drizzle version (a native module, rebuilt for Electron and staged in `forge.config.ts` `packageAfterCopy` like playwright-core). Decide only if step 1 of **Work** fails.

### Work

1. **Check the runtime.** A small check in the dev app's main process: `new DatabaseSync(":memory:")`, a table, an insert, a select, a transaction that rolls back. Record the Node version Electron reports. If it fails, stop and switch to the fallback.
2. **Dependencies** as in **Stack**.
3. **Schema.** `src/main/db/schema.ts` with `sqliteTable`s (below). Keys: integer primary keys with a unique text column where the natural key is text (a project folder, a setting section), not TEXT primary keys (see **Risks**, #6165).
4. **Config and migrations.** `drizzle.config.ts` at the repo root (`dialect: "sqlite"`, schema `src/main/db/schema.ts`, out `src/main/db/migrations`). A script `"db:generate": "drizzle-kit generate"`, run on Node. Generated migration files are checked in; add `src/main/db/migrations/**` to the generated-files table in `AGENTS.md` ("never hand-edit; regenerate with `bun run db:generate`").
5. **Opening.** `src/main/db/index.ts`: `openDatabase()` at `app.whenReady()` before anything reads settings; `PRAGMA journal_mode = WAL`, `PRAGMA foreign_keys = ON`, `PRAGMA busy_timeout`; before migrating, copy the file to `demoflow.db.bak-<schema version>`; run `migrate()` from `drizzle-orm/node-sqlite/migrator` with the folder Forge ships (stage `src/main/db/migrations` as an extra resource or import the SQL as text at build time); close on `will-quit`.
6. **Packaging.** Nothing native to stage. Check the packaged app finds its migrations and that the database lands in `<userData>`.
7. **Tests** (vitest, in-memory database): every migration applies to an empty database in order; a failed transaction leaves nothing behind (#1723); no migration file ends with an empty `--> statement-breakpoint` (#6207).

### Tables

| Table           | Columns (short)                                                                                         | Replaces                                                      |
| --------------- | ------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------- |
| `settings`      | `section` (unique), `value` (JSON text), `updatedAt`                                                    | `settings.json`                                               |
| `projects`      | `id`, `dir` (unique), `name`, `openedAt`                                                                | `recent-projects.json`                                        |
| `view_state`    | `projectId` (null = app-wide), `key`, `value` (JSON), unique (`projectId`, `key`)                       | `readPref` / `writePref` and `layoutStorage` in local storage |
| `window_bounds` | `window` (unique: `settings`, `panel:preview`…), `x`, `y`, `width`, `height`                            | `panel-windows.json`, `settings-window.json`                  |
| `fonts`         | `fontId` (unique), `def` (JSON `FontFamilyDef`)                                                         | `library.json` (font files stay in `<userData>/fonts`)        |
| `secrets`       | `scope` (`app`), `name` (unique per scope), `value` (`encryptedText`)                                   | new: service keys for voice and the assistant                 |
| `assist_log`    | `id`, `at`, `projectId`, `task`, `service`, `model`, `request` (JSON), `response` (JSON), `error`, `ms` | new                                                           |

## Per-project view state

### What it adds

Panel sizes, the active tab, Fast/Exact preview, preview shape, minimap, mixer folding and the Timeline's ruler and snapping are saved once per project for every window, instead of in each window's own local storage.

### Fit with Demoflow

Already in place:

- `src/renderer/store/view-prefs.ts` (`readPref`, `writePref`, `setViewProject`) is the single read/write path for the per-project preferences (`dockTab`, `previewQuality`, `previewShape`, `minimap`, `mixerCollapsed`, `rulerMode`, `snapping`), and `loadViewState` (`src/renderer/editor/view-state.ts`) loads them when a project opens.
- Panel layouts go through `layoutStorage` (`src/renderer/editor/panels.ts`) passed to `useDefaultLayout`.

Missing:

- An IPC pair (`getViewState(projectDir)`, `setViewState(projectDir, key, value)`) and a renderer cache, because `readPref` and `layoutStorage.getItem` are synchronous today.

### Work

1. `loadViewState` fetches the project's whole view state once (one IPC call) into a small store before the editor renders; `readPref` and `layoutStorage.getItem` read that store synchronously; writes update the store and send `setViewState`.
2. Main broadcasts changes so pop-out windows follow, like `settingsChanged`.
3. Window bounds use `window_bounds` instead of the two JSON files.

## Encrypted values

### What it adds

`encryptedText()`, a Drizzle column type that seals values before they reach the database, for the service keys the voice and assistant plans need.

### Fit with Demoflow

- The model is the email client's `encryptedText` (`../email-client/apps/web/src/db/crypto.ts`): a `customType<{ data: string; driverData: string }>` with `dataType: "text"`, AES-256-GCM, a 12-byte random IV, and the stored form `v1:<iv>:<tag>:<body>` in base64.
- Demoflow already encrypts project sign-ins with `safeStorage` (`src/main/secrets.ts`, files in the project's `secrets/`).

### Work

1. `src/main/db/crypto.ts`: the same `seal` / `open` and `encryptedText(name)`. The 32-byte key isn't an environment variable: it's made once with `crypto.randomBytes(32)`, encrypted with `safeStorage.encryptString` and kept in `<userData>/db-key.enc` (outside the database, so a copied database alone reveals nothing).
2. Refuse to create or use the key when `safeStorage.isEncryptionAvailable()` is false or, on Linux, `safeStorage.getSelectedStorageBackend()` is `basic_text`, and say why on the Settings page that asked for it.
3. A wrong key or a tampered value fails the GCM check: report "This key can't be read on this computer. Enter it again." and never crash.
4. Project sign-ins stay in the project's `secrets/` folder for now: they belong to the project, and moving them is a separate decision.
5. Tests: seal → open round trip, a tampered tag fails, an old prefix fails cleanly.

## Assistant request log

### What it adds

Each request the assistant sends (local or online) and its answer, stored in `assist_log` with the time, project, task, service and model, so the user can see what was sent. Keys and the request headers are never logged.

### Work

1. `logAssist(entry)` in `src/main/db/assist-log.ts`, called by the assistant's client after each request.
2. A list in Settings → Assistant (newest first, with what was sent and what came back), and **Clear log**.
3. Keep the last 1,000 entries or 30 days, whichever is smaller, pruned at startup.

## Licence

`drizzle-orm` and `drizzle-kit`: Apache-2.0; add them to `THIRD_PARTY_NOTICES.md`. SQLite (inside Node): public domain.

## Risks

- **Release candidate.** No stable date for Drizzle 1.0 when checked: `rc.4` came out on 2026-06-27, with only unlabelled `rc.5` test builds since, while 0.45.x kept getting patches. Breaking changes between RCs are possible; moving to the stable release is a separate step that re-runs the migration tests.
- **#6165:** drizzle-kit v1 leaves `NOT NULL` off SQLite TEXT primary keys (open against `rc.4`). Avoided by integer keys (step 3); review each generated migration anyway.
- **#6207:** the migrator sends blank statements when a migration ends with an empty breakpoint. Avoided by never hand-editing migrations and by the test in step 7.
- **#5515:** drizzle-kit can't find `node:sqlite` under Bun's runtime. Not a risk here: nothing runs under `--bun`.
- **#1723:** reports of transactions not rolling back (old, mostly other drivers). Covered by the rollback test.
- **Downgrades.** An older app version doesn't know the database. The pre-migration backup lets a user go back by hand.

## Sources

- [Drizzle: Node SQLite driver](https://orm.drizzle.team/docs/sqlite/connect-node-sqlite) and [v1 roadmap](https://orm.drizzle.team/roadmap) (checked 2026-10-17)
- npm metadata of `drizzle-orm`: dist-tags (`latest` 0.45.4, `rc` 1.0.0-rc.4), release dates, `exports` of 0.45.4 (no `./node-sqlite`) and 1.0.0-rc.4 (`./node-sqlite`, `./node-sqlite/driver`) (checked 2026-10-17)
- Drizzle issues [#2648](https://github.com/drizzle-team/drizzle-orm/issues/2648), [#6165](https://github.com/drizzle-team/drizzle-orm/issues/6165), [#6207](https://github.com/drizzle-team/drizzle-orm/issues/6207), [#5515](https://github.com/drizzle-team/drizzle-orm/issues/5515), [#1723](https://github.com/drizzle-team/drizzle-orm/issues/1723) and advisory [GHSA-gpj5-g38j-94v9](https://github.com/advisories/GHSA-gpj5-g38j-94v9) (fixed in 1.0.0-beta.20, so not in rc.4) (checked 2026-10-17)
- [Electron PR #47757](https://github.com/electron/electron/pull/47757) (missing SQLite built-in fixed) and [issue #45532](https://github.com/electron/electron/issues/45532) (checked 2026-10-17)
- `../email-client/apps/web/src/db/crypto.ts` (the encrypted column type this plan copies)
