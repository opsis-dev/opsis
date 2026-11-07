# Opsis — agent rules

> What every change follows. `README.md` says what the app does and `plans/` holds planned work. Each domain's detail lives in one file under [`agent-docs/`](agent-docs/) (index in §8): read the doc for the domain you touch before changing it.

## 1. The project

- **Opsis is a desktop app (Rust, with a native Qt Widgets editor): browser and terminal nodes record takes, and the other nodes turn them into finished videos.** GPL-3.0-or-later and unreleased.
- **The editor is Qt 6 Widgets with KDDockWidgets, its views written in C++ over Rust through a `cxx` bridge** ([ui.md](agent-docs/ui.md)). **Capture, the streamed views and every render run in the recording browser**, a pinned Chrome Headless Shell ([engine.md](agent-docs/engine.md)).
- **There is no live preview: a film is watched as an export.**
- **Rust owns the open project, the document, compiling, runs and every file; windows are views onto it** ([document.md](agent-docs/document.md)).

## 2. Hard rules

- **No comments. Any kind. Anywhere.** No `//`, `///`, `//!`, no `/* */`, no JSDoc, no file headers, no CSS comments, in source, tests, scripts and config, in TS, Rust, Zig and C++.
  - If something needs explaining, the name is wrong or the function is too big; a constraint that spans the codebase goes in this file, a domain's in its doc.
  - Allowed comment-shaped lines are functional directives only: `oxlint-disable…`, `/// <reference …>`, `@ts-…`. Rust attributes (`#[expect(…)]`, `#[cfg(…)]`) and a lint exception's `reason` are not comments.
  - `bun run lint:comments` (`scripts/comments.ts`, part of `lint` and `lint:fix`) checks every TS file through `oxc-parser`, C++ files line by line outside string literals, and CSS, Rust and Zig files line by line, everywhere but installed and generated code. TypeScript 7 has no parser API; its native compiler only type-checks.
- **TS file names are kebab-case** (`trace-panel.tsx`); the code inside keeps its usual names (`TracePanel`).
  - oxlint enforces it (`unicorn/filename-case`).
  - Rust module files are snake_case, because a module name must be an identifier.
  - Rename a TS file with the TypeScript rename edits (`getEditsForFileRename`, what VS Code calls on file move) so imports follow, then search for the old name in strings, configs and docs.
- **Components are named functions** (`export function Title({ … }: ElementSettings["title"])`), never `React.FC` constants (`opsis/use-function-components`, `scripts/oxlint-plugin-opsis.ts`).
- **Every function is a declaration: `function name(…) { … }`, never a function assigned to a `const`**, at module level or inside another function. Inline callbacks passed as arguments stay arrows; installed code keeps its upstream style.
  - `opsis/use-function-declarations` enforces it and `bun run lint:fix` rewrites what it safely can. It leaves to you a `const` with a type annotation, a named function expression and an arrow that uses `this` or `arguments`.
  - A nested declaration is hoisted, so it doesn't see a narrowing of the outer scope (`if (!gl) return`): pass the checked value in, or move the code after the check into a function that takes it.
  - `opsis/use-find-by-id`, `opsis/use-type-guards`, `opsis/use-function-components` and `opsis/use-percent-text` (only `Math.round(x * 100)%`) also rewrite code and add the import from `common/`. Two fixes that add imports to one file land on separate passes: run `lint:fix` again until it reports nothing.
- **Never hand-edit installed or generated code.** Regenerate it:

| Path                                                                                                                                        | Owner                                                                                                                                                                                                                                                                         | Regenerate with                                                                                      |
| ------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| `public/fonts/bundled/**`, `crates/compile/src/fonts/bundled.generated.json`                                                                | Fontsource packages (`@fontsource-variable/*`)                                                                                                                                                                                                                                | `bun run fonts:bundle` (list in `scripts/fonts-bundle.ts`)                                           |
| `brand/icons/*.png`                                                                                                                         | `brand/logo.svg` (the one source for the app icon)                                                                                                                                                                                                                            | `bun run brand:icons` (renders with the recording browser; ImageMagick draws the gradients wrong)    |
| `resources/**` (gitignored)                                                                                                                 | the packaging stage: the browser's licences                                                                                                                                                                                                                                   | `bun run stage`                                                                                      |
| `crates/compile/src/terminal/themes.generated.json`                                                                                         | VHS's `themes.json` (https://github.com/charmbracelet/vhs, MIT), at the commit and sha256 pinned in `scripts/terminal-themes.ts`                                                                                                                                              | `bun run terminal:themes`                                                                            |
| `crates/ui/glyphs.generated.json`                                                                                                           | the glyph sets' own packages (dev packages: `lucide`, `@tabler/icons`, `@lucide/lab`, `@hugeicons/core-free-icons`, `iconoir`, `@mynaui/icons`) for the glyphs the catalogue's node looks name, and `lucide` for the editor's own icons (`ICONS` in `scripts/node-glyphs.ts`) | `bun run glyphs:bundle`                                                                              |
| `packages/video/src/camera/lib/mac-cursors.generated.ts`                                                                                    | apple_cursor (https://github.com/ful1e5/apple_cursor, GPL-3.0, not on npm): its macOS theme's SVGs for the CSS cursor keywords, with their hotspots                                                                                                                           | `bun run cursors:bundle <path to an apple_cursor checkout>`                                          |
| `.sqlx/`                                                                                                                                    | the `sqlx` query macros of `opsis-store`, checked against `crates/store/schema.sql`                                                                                                                                                                                           | `bun run db:prepare` (a throwaway SQLite database made from the schema, then `cargo sqlx prepare`)   |
| `packages/video/src/generated/spec.ts`, `packages/video/src/registry/length-cases.generated.json`, `default-props.generated.json` beside it | the timeline spec types, the element length cases and every element's default props (`opsis-compile`)                                                                                                                                                                         | `bun run types:export`                                                                               |
| `crates/engine/src/downloads.generated.json`                                                                                                | the recording browser downloads, pinned by sha256 and size (`scripts/engine-downloads.ts`, for the browser version it's given or the one already pinned)                                                                                                                      | `bun run engine:downloads` (about 600 MB into `~/.cache/opsis-engine-downloads`)                     |
| `scripts/lib/ffmpeg.generated.json`                                                                                                         | the bundled FFmpeg: the ffmpeg-static release's program, its ffprobe and its licence for each target, pinned by sha256 and size (`scripts/ffmpeg-pin.ts`)                                                                                                                     | `bun run ffmpeg:pin` (about 130 MB into `~/.cache/opsis-ffmpeg`)                                     |
| `target/ffmpeg/**` (gitignored), `resources/LICENSE.ffmpeg.txt`                                                                             | the pinned FFmpeg for this computer as `opsis-ffmpeg-<target triple>`, and its licence (`stageFfmpeg` in `scripts/lib/ffmpeg.ts`, downloads cached in `~/.cache/opsis-ffmpeg`)                                                                                                | `bun run ffmpeg:stage` (`bun start`, `bun run test:rust`, the e2e runner and `bun run stage` run it) |
| `crates/capture/src/playwright/*.generated.*`                                                                                               | `playwright-core`: its injected page script, its fake clock, the US keyboard layout, the Chromium switches and fonts it launches with                                                                                                                                         | `bun run playwright:scripts`                                                                         |
| `film-bundle/**`                                                                                                                            | the film page (Vite, `apps/film`), with the repository's `public/`                                                                                                                                                                                                            | `bun run film:bundle` (`-- --mode release` for no source maps)                                       |
| `terminal-bundle/**`                                                                                                                        | the terminal page (Vite, `apps/terminal`), without `public/`                                                                                                                                                                                                                  | `bun run terminal:bundle` (`-- --mode release` for no source maps)                                   |

- **Golden files in `fixtures/**` are recorded, never edited by hand.** They hold what the code made for fixed inputs, stored with those inputs, compared through `opsis-golden` (`crates/golden`: numbers within one part in a billion, every difference listed by its path).
  - When a change is intended, record them with `OPSIS_WRITE_GOLDEN=1` (`cargo test -p <crate>`, plus `-- --ignored` for those that need the dev engine, the film page or the internet) and review the diff.
- **No old formats.** Nothing is released, so there is no migration or fallback code for older project files, settings, engine folders or caches.
  - When a format changes, change it everywhere, write `examples/` again (`bun run examples`, `crates/project/src/bin/write-examples.rs`, from `examples()` in `crates/project/src/templates.rs`) and delete local test projects.
  - A test opens every example and fails when one isn't what the templates write; the formatter leaves `examples/` alone so they stay exactly as the app saves a project.
- **The cache version constants (`PICTURE_VERSION`, `PREPARED_VERSION`, `ENGINE_VERSION`, `MIX_VERSION`) are not bumped before the first release.** To get fresh results while verifying, delete the test project's `.opsis/cache` or use a new project. What each one keys is in [runs.md](agent-docs/runs.md).
- **Do not commit or push.** Leave changes in the working tree unless asked.
- **Large files go to `~/.cache/opsis-*`** (browser binaries, clones, renders, release builds); `/tmp` is for small scripts and screenshots only. Long-running processes start with `run_in_background`.

## 3. Toolchain

- **bun is the package manager and runs every script** (`bun scripts/x.ts`, the e2e runner too); scripts import TypeScript without file extensions. Never run scripts through `tsx` or Node.
- **Every tool config is a `*.config.ts`, except the Rust ones**, which are TOML at the root (`rust-toolchain.toml`, `rustfmt.toml`, `deny.toml`). Shared TS tool configs are in `scripts/lib/` (`oxlint-base.ts`, `oxfmt-base.ts`, `vitest-base.ts`, `tool-config.ts`).
- **Rust builds on one pinned nightly, `rust-toolchain.toml`**, with `rust-version` in `[workspace.package]` refusing an older compiler. Moving it touches several files and another repository ([toolchain.md](agent-docs/toolchain.md)).
- **Zig 0.17 builds `opsis-vt`'s wrapper**, and `crates/vt/build.rs` refuses any other version ([toolchain.md](agent-docs/toolchain.md)).
- **Qt 6.12 and KDDockWidgets 2.5.1 build `opsis-qt`**, from `~/.cache/opsis-qt` after `bun run qt:install`; its C++ follows Google style ([toolchain.md](agent-docs/toolchain.md)).
- **Rust imports are grouped std, external crates, then the workspace's own, one `use` per module, and struct fields use the shorthand.** `rustfmt.toml` keeps the imports so (`group_imports = "StdExternalCrate"`, `imports_granularity = "Module"`, applied only by the nightly).
- **`bun start` (`scripts/start.ts`) stages FFmpeg when missing and runs the editor (`cargo run -p opsis-ui`).** Exports and terminal previews also need `bun run film:bundle`, live terminal pictures `bun run terminal:bundle`; make each again whenever `packages/video` or `packages/runtime` changes.
- **To see a change in the running app, use the `check-app` skill** (`.claude/skills/check-app`): it seeds `~/.cache/opsis-check` (short, so the API socket fits), starts `bun start` in a virtual display and drives it through the Local API.
- **Environment variables:** `OPSIS_USER_DATA` moves the data folder (default `<config dir>/Opsis`), for tests and side-by-side runs; `OPSIS_FFMPEG` (absolute, read only by builds with debug assertions) names the FFmpeg the app and tests start; `OPSIS_API_SOCKET` moves the Local API socket ([api.md](agent-docs/api.md)); `OPSIS_AUDIO_DEVICE` (a sound output's id such as `alsa:null`, read only by builds with debug assertions) replaces the default sound output; `OPSIS_WRITE_GOLDEN=1` records golden files.
  - `bun start`, `bun run test:rust` and the e2e runner set `OPSIS_FFMPEG` to the staged program; a test that starts FFmpeg reads it through `opsis_golden::ffmpeg()`.

## 4. Verifying

- **Verify with `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`.** The `:fix` variants fix and report the rest; do not run the non-fix variants first.
  - `lint:fix` runs oxlint, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo deny check advisories licenses` (`lint:deny`), the Zig wrapper build (`lint:zig`), `clang-tidy` on the C++ (`lint:cpp`) and the comment check.
  - `format:fix` runs oxfmt (TS, JSON and Markdown), `cargo fmt --all`, `zig fmt` (`format:zig`) and `clang-format` on the C++; `test` runs every package's Vitest and `cargo test --workspace`.
- **Clippy runs `pedantic` and `nursery` as warnings and `-D warnings` fails on any of them.** The permanent workspace allows and why each exists are in [toolchain.md](agent-docs/toolchain.md); never add a workspace line.
- **A lint exception is local, on the item or statement, and written `#[expect(lint, reason = "…")]`**, the reason saying why the flagged code is meant; `unfulfilled_lint_expectations` reports it once the lint stops firing.
  - An exception that holds in only some builds sits in a `cfg_attr` for those builds (`native_menu`, dead outside macOS).
  - `#[allow]` is only for code where an expectation can't hold in every build that compiles it (test helpers shared by several test binaries, `crates/capture/tests/support/mod.rs`), as an inner `#![allow(…, reason = "…")]`. Clippy's `allow_attributes` and `allow_attributes_without_reason` fail the rest.
  - An oxlint suppression is `// oxlint-disable-next-line <plugin>/<rule>`.
- **A pure function with an invariant (a round trip, a bound, an order, a count) is tested as a property over generated inputs**: `proptest` in Rust, `fast-check` in TS. A hand-written case only pins exact text people see; a test never restates a lookup table or checks a pure function against itself.
- **TS tests are `*.test.ts` on Node** (each package's `src`, the apps' `src`, `scripts`); a test that renders React components or hooks is `*.test.tsx` in happy-dom with Testing Library (the two `projects` in `scripts/lib/vitest-base.ts`).
- **Scripts that need the catalogue read it from Rust through the `editor-data` binary** (`crates/compile/src/bin/editor-data.rs`, built in release): `glyphs:bundle` and the profiler.
- **`cargo test -p opsis-ui` checks the native views on Qt's offscreen platform**, through the `views` scenario, so `bun run test` covers them without a display ([e2e.md](agent-docs/e2e.md)).
- **Tests that need the dev engine or a built page are `#[ignore]`d with the reason**; run them with `cargo test -p <crate> -- --ignored` (render tests also need `bun run film:bundle`).
- **End-to-end tests run only in a virtual display, never on the real screen, and never move the real pointer** ([e2e.md](agent-docs/e2e.md)).

## 5. Dependencies

- **Dependencies go through the package managers**: `bun add`, `bun add -d`, `bun remove` for TS (inside the folder that imports it), `cargo add` / `cargo remove` for Rust. Never edit dependency blocks by hand, except the root table below.
- **Every Rust dependency's version lives once in the root `Cargo.toml`'s `[workspace.dependencies]`.**
  - The workspace's own crates are path entries (`opsis-types = { path = "crates/types" }`); every other crate sits at its loose major (`"1"` from 1.0 on, `"0.<minor>"` before).
  - Crates take them with `dep.workspace = true` or `dep = { workspace = true, features = [...] }`, in `[dependencies]`, `[dev-dependencies]` and `[build-dependencies]`. `cargo add -p <crate> <dep>` reuses an entry already in the root table.
  - A new dependency needs its pin added to the root table by hand (the only dependency block edited by hand), then `cargo add -p`.
  - A root entry carries only the features every crate that uses it needs, since crates can add features but never remove them, and `default-features = false` wherever any crate turns them off; a crate that wants them says `default-features = true` (`image`).
- **`Cargo.lock` holds exact versions, and adding or updating a dependency takes the latest release**, except these pins:
  - `specta` `=2.0.0-rc.26` and `specta-typescript` and `specta-serde` `=0.0.13` in the root table: prereleases whose API changes between releases and whose output is `packages/video/src/generated/spec.ts`, so they move together and only with `bun run types:export`.
  - Ghostty's terminal engine is pinned in `crates/vt/zig/build.zig.zon` to the lib-vt source tarball of v1.4.0 (commit `83fb956`) by URL and hash, written with `zig fetch --save=ghostty <url>` in `crates/vt/zig`, never by hand. Ghostty builds only with the Zig version it names, so moving it may move Zig too.
- **Every dependency must be GPLv3-compatible** ([packaging.md](agent-docs/packaging.md) has the allowed licences and the notices).

## 6. Workspace and layout

- **A Bun workspace run by turbo (`turbo.json`) beside a Cargo workspace** (`Cargo.toml`, members `crates/*`, `Cargo.lock` at the root).
- **Apps build things and nothing imports them**: `apps/film` (`@opsis/film`, the film page the frame driver renders, mounting `@opsis/video/film`), `apps/terminal` (`@opsis/terminal`, the page that paints live terminal pictures, on `@opsis/video/terminal/view`).
- **`packages/video` (`@opsis/video`) is source the apps compile**: the film (`film.tsx`, `DemoTimeline`), the elements, and its plain helpers (`src/common`).
- **`packages/runtime` (`@opsis/runtime`) is the film's page runtime**: the clock (`useFrame`, `useFilm`), `Sequence`, `Freeze`, `Loop`, the render holds (`holdRender`, `failRender`, `useHeldLoad`), `AbsoluteFill`, `Img`, the render environment, the maths (`interpolate`, `Easing`, `spring`, `random`, our own, property-tested), the page protocol (`page.tsx`) and the test harness (`test-frame.tsx`).
- **Each member declares the dependencies it imports and has its own `tsconfig.json` (extending `tsconfig.base.json`), `vitest.config.ts` and `oxlint.config.ts`**; root scripts run each check everywhere plus the root's own (`typecheck:app`, `lint:app`, `test:app` cover `scripts`).
- **Installs are bun's isolated layout**: a folder imports only what it or the root declares, and scripts read a package's files from the package that owns it.
- **Imports:**
  - Members import each other only by package name, through their `exports` (`@opsis/video/common/*`, `@opsis/video/generated/spec`, `@opsis/video/film`…).
  - A member imports its own files through its `package.json` `imports` with its own prefix (`#video/…`, `#runtime/…`, `#film/…`), never `#/`. `video` and `runtime` mix `.ts` and `.tsx` and import with the extension (`#video/core/frame.tsx`).
  - Nothing imports by a relative path; oxlint rejects `@/` in `video`.
- **Use the shared helpers before writing a local copy**: `packages/video/src/common` (`math.ts`, `geometry.ts`…) for plain helpers, `packages/video/src/lib/<purpose>/` for the video's helpers, `packages/video/src/parts/` for parts several elements share.

```
crates/                Rust, one domain per crate (table below)
apps/film/src/         the film page: mounts the film for the frame driver
apps/terminal/src/     the terminal page: paints live terminal frames for the editor
packages/runtime/src/  the film's page runtime
packages/video/src/    the film: film.tsx, DemoTimeline, registry/, core/, camera/, three/, terminal/, common/, generated/, elements/, parts/, lib/
fixtures/              golden files
examples/              example projects, written by `bun run examples`
public/                the app's files (`app:` refs): fonts, sounds, music, placeholders, backdrops
scripts/               bundling, pinning, packaging, the oxlint plugin, shared tool configs
plans/                 planned work
agent-docs/            the domain docs (§8)
```

- **Rust crates live in `crates/<domain>`, named `opsis-<domain>`, one domain each; nothing Rust lives in `apps/`.** Only `opsis-qt` opens windows or dialogs; `opsis-session` asks its host through `Platform`.

| Crate             | Domain                                                                                                                                                                                                              |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `opsis-types`     | every shape that crosses a boundary, and `numbers`                                                                                                                                                                  |
| `opsis-compile`   | node and element definitions, settings, compile, cards, validation, the Inspector context, the timeline flattener, chapters, the mix plan, the catalogue, the edits (`edits/`), `editor-data`                       |
| `opsis-store`     | the app database (`schema.sql` and its queries), settings, view state, recent projects, window bounds, font rows, the keyring key and sealed sign-ins                                                               |
| `opsis-document`  | the open project's document: workflow, undo and redo, gestures, revisions, the saved revision and file version, the nodes each change edited                                                                        |
| `opsis-project`   | project folders: create from `templates.json`, open and save with versions, watching, asset and model import, file stamps, the last run, `examples()`                                                               |
| `opsis-fonts`     | the font library, the computer's fonts and online fonts                                                                                                                                                             |
| `opsis-engine`    | engine paths, the browser installer, the bundled FFmpeg, tool helpers (`EngineFiles`), licences, child temp folders and processes, the file server, file and hash helpers, `PICTURE_VERSION` and `PREPARED_VERSION` |
| `opsis-traces`    | the trace store                                                                                                                                                                                                     |
| `opsis-runner`    | the run: plan, cache keys, saved results, run events and progress                                                                                                                                                   |
| `opsis-audio`     | playback sound: the mix stem decoded through FFmpeg and played through cpal, its clock, the renderer                                                                                                                |
| `opsis-media`     | FFmpeg argument builders, runs and probes behind `MediaTools`, the media and sound executors, mix stems, loudness, waveform and filmstrip analysis                                                                  |
| `opsis-render`    | film renders (`frames.rs`), stills, saved pictures, the output executors, take previews                                                                                                                             |
| `opsis-prepare`   | prepared files: 3D typefaces, models, SVG logos, textures                                                                                                                                                           |
| `opsis-image`     | the image node executors                                                                                                                                                                                            |
| `opsis-footage`   | the film's footage frames, served beside the prepared files                                                                                                                                                         |
| `opsis-probe`     | reads a media file's length, container, picture and sound from its own bytes, for every probe the app makes                                                                                                         |
| `opsis-vt`        | Ghostty's terminal engine behind one safe `Terminal`; one of the two crates allowed `unsafe`                                                                                                                        |
| `opsis-terminal`  | terminal takes: shells, sessions, the driver, takes, casts, tapes, the live feed, prepared frames                                                                                                                   |
| `opsis-capture`   | the recording browser over CDP: sessions, selectors, actions, the screencast encoder, smooth takes, sign-in steps, stills, the film's `FrameDriver`, `TerminalPainter`                                              |
| `opsis-stream`    | the streamed views (sign-in, element picker)                                                                                                                                                                        |
| `opsis-api`       | the Local API                                                                                                                                                                                                       |
| `opsis-session`   | the open project and everything windows follow, with no window code: document, compiles, runs, shared state, settings, fonts, engine, streamed views, the exit decision, the Local API host                         |
| `opsis-qt`        | the native editor's view layer: Qt 6.12 Widgets and KDDockWidgets built from Cargo, the C++ views, the `cxx` bridge behind one safe `Host` trait; the other crate allowed `unsafe`                                  |
| `opsis-ui`        | the native editor's state and logic behind the views, implementing `Host` over `opsis-session`; the `opsis` binary                                                                                                  |
| `opsis-e2e-input` | dev-only: holds a virtual pointer and keyboard on a headless Wayland compositor and carries out the e2e's real input requests                                                                                       |
| `opsis-golden`    | dev-only: the golden-file helper and the readers and builders more than one crate's tests use (`repo`, `home`, `typed`, `text`, `number`, `cases`, `node`, `edge`)                                                  |
| `opsis-macros`    | procedural macros (`proc-macro2`, `syn`, `quote`; no `paste` or `pastey`, since `quote` and `format_ident!` build every name)                                                                                       |

- **Test helpers only one crate's tests share live in its `tests/support/mod.rs`**; those several crates share go in `opsis-golden`.

## 7. Rules across the codebase

### Boundaries and types

- **`packages/video` and `packages/runtime` are runtime-neutral**: the editor and the film page both compile them, so no `node:*` or `playwright-core` there (oxlint enforces it). Node-only code lives in `scripts`; everything else that touches the system is Rust.
- **Rust is the source of every type that crosses a boundary** (the editor's views and calls, the timeline spec, run events). The editor's are the `cxx` bridge's structs ([ui.md](agent-docs/ui.md)); the film's are exported with specta (`bun run types:export`) into `packages/video/src/generated/spec.ts`. Never write such a type by hand in TS; tests fail when an export is stale.
  - JSON Schema for the Inspector and the Local API comes from the same types (`schemars`).
- **Whole numbers are integers**: frame lengths, sizes, frame rates, counts and settings in milliseconds are `u32`, a sound's start (which can fall before its clip) `i32`, timestamps `i64` milliseconds since the epoch. Only values that can be fractional (positions in a remapped clip, action times, volumes, rates) are `f64`.
- **No number is converted with `as`.** Widening uses `f64::from`, integers `try_from`, everything else `opsis_types::numbers`.
  - `whole` and `whole_signed` round a float into a whole number, `truncate` cuts toward zero like a cast, `float`, `float_wide` and `narrow` go to `f64` and `f32`, `count` and `signed` narrow integers, `millis` reads a `Duration`.
  - `byte_size` writes a size in binary units ("1.5 KiB"), the one way sizes read in the app.
  - The `pedantic` cast lints fail a new cast.
- **Floats are compared exactly only through `exact_eq`** (a setting at its neutral value, whole numbers kept as `f64`, exact properties in tests). Clippy's `float_cmp` skips functions with `eq` in their name, so `exact_eq` needs no exception.
- **Kinds that form a fixed set are enums** (`BrowserAction`, `Sign`), and node types the code names are constants next to their definitions (`OPEN_BROWSER_TYPE`, `RENDER_TYPE`).
- **Each module has its own error enum (`thiserror`: `SettingsError`, `CompileError`, `StartupError`…)**, turned into text only where it reaches a person or a JSON field. A missing value falls back to a default only when the default is what it means.
- **Every public Rust type implements `Debug`** (`missing_debug_implementations`): it derives it, private types it holds derive it too, and where a field has none (a closure, a trait object) it writes `Debug` by hand, listing the fields that have it and ending with `finish_non_exhaustive`.
- **`unsafe_code` is denied in every crate but `opsis-vt` and `opsis-qt`** (`[workspace.lints.rust]`), whose `lib.rs` allow it with a reason. `opsis-vt` keeps its `unsafe` in `src/ffi.rs` and `src/terminal.rs`; `opsis-qt`'s is the code the `cxx` bridge expands to (`src/bridge.rs`).
- **Cached JSON is read into typed structs (`read_json`, `crates/engine/src/files.rs`), never trusted as-is.**

### The film

- **Every animation in `packages/video` is a pure function of `useFrame()`**, drawn through `@opsis/runtime` only. No Framer Motion, `motion`, GSAP, CSS transitions or Tailwind `animate-*` (oxlint: `no-restricted-imports`, `opsis/no-animate-class`).
  - Randomness uses `random(seed)`; async work holds the frame with `holdRender()` and calls the release it returns; colours between stops mix in OKLCH (`interpolateColors`, `lib/ui-kit/color.ts`); `app:` files reach the page through `assetUrl`.
  - The rest of the film page's contract is in [film.md](agent-docs/film.md).
- **Nodes never generate film code.** A node has ports, settings and an optional pure `compile`; compile produces a `TimelineSpec` and the film page's one `DemoTimeline` draws it ([nodes.md](agent-docs/nodes.md)).
- **Element metadata (`crates/compile/src/elements`) never names a component**; implementations are keyed separately (`ELEMENTS`, `packages/video/src/elements/index.ts`) ([elements.md](agent-docs/elements.md)).
- **Every value a user would want to change is a setting** in the Inspector; only timing the Timeline owns, animation tuning, functions and React nodes stay in code ([inspector.md](agent-docs/inspector.md)).
- **Never load fonts, images, scripts or sounds from a remote URL at render time.** Element defaults use neutral copy and `app:` placeholders (`public/placeholders`, `public/backdrops`).

### The app

- **Nothing uses the internet unless the user asks.** The app works offline; anything that needs the network (setting up the engine, getting fonts online) sits behind a button and asks first, and what it downloads is saved locally (`<data folder>/engine`, `<data folder>/fonts`).
- **Never hold a lock across a call that may take it again** (the settings lock and `announce`, which reads the settings): a database lock held that way hangs the close.
- **Nothing blocks on the async runtime.** The keyring is the one thing made in setup itself, and a run reaches it only through `off_runtime` ([app-state.md](agent-docs/app-state.md)), because the Linux Secret Service store blocks inside `zbus` and panics on a runtime thread.
- **Register long app work in `AppState.tasks`** (`begin(label)`/`end(label)`, `crates/session/src/state.rs`) so the exit dialog lists it.
- **Never write intermediates to the system temp folder from the app**; every child program gets its own temp folder ([engine.md](agent-docs/engine.md)).
- **Views that read a node by id handle it being gone**: a panel can refresh once more after the node is deleted.
- **The editor keeps no state of its own**; app state is the database in Rust ([app-state.md](agent-docs/app-state.md)).
- **Every Local API route is documented in `crates/api/src/openapi.rs`** ([api.md](agent-docs/api.md)).

### User-facing copy

- **No library, tool or format names in node titles, descriptions, field labels, badges, toasts or error messages** (Playwright, Chromium, FFmpeg, CRF, DPR, page.evaluate…), and never Tauri, Rust, Bun, CDP, a crate or the services behind a download. Settings → Licences and About are the exceptions, listing what Opsis includes by name. Technical settings go under `advanced`.
- **Sentence case; durations in seconds.**
- **Browser nodes are "actions", never "steps".**
- **Elements are described in words** (`element_name`); element descriptions are on their entries.
- **The node's name (`node.label`) is also its camera, chapter and caption label.**
- **Categories keep their internal ids**; display names come from `Category::label` (`crates/compile/src/node.rs`), sent with the catalogue.
- **`crates/compile/tests/unit/nodes_c.rs` enforces descriptions and the banned words.**

## 8. Domain docs

| Doc                                     | Domain                                                                                                                                        |
| --------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| [toolchain.md](agent-docs/toolchain.md) | moving the Rust nightly, Zig and the Ghostty build, Qt and KDDockWidgets, the C++ checks, the Clippy workspace allows, the Opsis oxlint rules |
| [nodes.md](agent-docs/nodes.md)         | node definitions, ports and resources, conversions, file needs, claims, executors, growing inputs, cards, looks and glyphs                    |
| [inspector.md](agent-docs/inspector.md) | `#[settings]` and `#[choice]`, schema keywords, widgets, `InspectorContext`, form behaviour, what is a setting                                |
| [elements.md](agent-docs/elements.md)   | element files in Rust and TS, themes, props, element sounds, lengths and the 30 fps clock, design size, stills, 3D elements                   |
| [film.md](agent-docs/film.md)           | the film page and `window.opsis`, the frame driver, paint traps, canvas transitions, transitions, other shapes, window frames                 |
| [capture.md](agent-docs/capture.md)     | browser sessions, takes, recording style, click and drag, the screencast, smooth takes, streamed views and sign-in, traces, screenshots       |
| [terminal.md](agent-docs/terminal.md)   | `opsis-vt`, terminal takes, tapes and casts, the live tape window, previews, live pictures, terminal grading, `TerminalView`                  |
| [footage.md](agent-docs/footage.md)     | footage uses, the colour grade list, Apply grade, Grade footage, Redact take, the Mac cursor                                                  |
| [assets.md](agent-docs/assets.md)       | asset refs and `hosted`, the file servers, prepared files, model import, followed project files                                               |
| [audio.md](agent-docs/audio.md)         | built-in sounds and music, the Mixer, stems and ducking, sound nodes and their filter traps, action and transition sounds                     |
| [images.md](agent-docs/images.md)       | the image nodes: loading, depth, areas, size planning, text drawing                                                                           |
| [timeline.md](agent-docs/timeline.md)   | the flattener, handles and timeline edits, keyframes, the Timeline clock, chapters in exports                                                 |
| [editor.md](agent-docs/editor.md)       | windows, docks and panels, the title bar and menus, canvas, look and motion, starting and closing, Settings, view state                       |
| [ui.md](agent-docs/ui.md)               | the native editor's bridge: Rust and C++ sides, views, events, frames, gestures, C++ rules                                                    |
| [document.md](agent-docs/document.md)   | the document in Rust, edits and gestures, compiles, shared state, live frames, autosave and disk changes                                      |
| [app-state.md](agent-docs/app-state.md) | the app database, settings, the keyring, projects folder                                                                                      |
| [api.md](agent-docs/api.md)             | the Local API: socket, routes, `ApiHost`, OpenAPI                                                                                             |
| [runs.md](agent-docs/runs.md)           | run state, results following keys, media file checks, cache keys and version constants                                                        |
| [engine.md](agent-docs/engine.md)       | the recording browser, the bundled FFmpeg, probing, child temp folders and processes                                                          |
| [fonts.md](agent-docs/fonts.md)         | bundled and terminal fonts, the library, project fonts and roles, `family_for`                                                                |
| [packaging.md](agent-docs/packaging.md) | `bun run make`, staging, the staged FFmpeg, icons, licences and notices; installers wait for the packaging step                               |
| [e2e.md](agent-docs/e2e.md)             | the e2e scenarios, their virtual display and headless Wayland, data folders, the offscreen view tests                                         |
| [profiling.md](agent-docs/profiling.md) | the profiling build, `bun run profile` in Docker, per-node CPU and memory, the size tools                                                     |

### Writing AGENTS.md and agent-docs

- **AGENTS.md holds what any session may need**: the hard rules, the toolchain, verifying, dependencies, the layout and the rules that cross domains. Everything else goes in the domain doc.
- **The docs hold only what the code can't say**: the why, the flows across files and crates, invariants, traps and deliberate limits, and dependencies across projects. Never a line-by-line description of the code.
- **Point to code by exact path and symbol** (`crates/compile/src/style.rs` → `restyle_with`). Never copy what the code already lists (node catalogues, enum variants, columns, tuning constants); name the file that holds it.
- **Each fact has one home doc**; other docs link to it.
- **Present tense, current state only.** No changelog language ("now", "no longer", "was", "used to", "renamed", "previously", "once" for "formerly"); history lives in git.
- **One rule per bullet, the rule first and in bold; mechanism goes in sub-bullets.**
- **Every doc has the same shape:** a title; a short scope paragraph naming what it covers, what its paths are relative to and the related docs; then `## Code map`, `## Rules`, `## How it works` and `## Deliberate limits`, leaving out any section it has nothing for.
- **A change that makes a doc wrong updates that doc in the same change**; a plan in `plans/` that is built is deleted, its design moving into the docs.
