# Opsis — agent rules

> What every change follows. `README.md` says what the app does and `plans/` holds planned work. Each domain's detail lives in one file under [`agent-docs/`](agent-docs/) (index in §8): read the doc for the domain you touch before changing it.

## 1. The project

- **Opsis is a desktop app (Rust, with a native Qt Widgets editor): browser and terminal nodes record takes, and the other nodes turn them into finished videos.**
- **Opsis is GPL-3.0-or-later and unreleased.**
- **The editor is Qt 6 Widgets with KDDockWidgets, its views written in C++ over Rust through a `cxx` bridge** ([ui.md](agent-docs/ui.md)).
- **Capture and the streamed views run in the recording browser**, a pinned Chrome Headless Shell ([engine.md](agent-docs/engine.md)).
- **Films are drawn natively**: elements return scene nodes in the film's CSS and SVG subset, Taffy and Parley lay them out, Skia paints them on the CPU, and Filament runs the shader effects and 3D in the renderer program ([scene.md](agent-docs/scene.md), [film.md](agent-docs/film.md)).
- **There is no live preview: a film is watched as an export.**
- **Rust owns the open project, the document, compiling, runs and every file; windows are views onto it** ([document.md](agent-docs/document.md)).

## 2. Hard rules

- **No comments. Any kind. Anywhere.** No `//`, `///`, `//!`, no `/* */`, no JSDoc, no file headers, no CSS comments, in source, tests, scripts and config, in TS, Rust, Zig, C++, GLSL materials and Lua.
  - If something needs explaining, the name is wrong or the function is too big; a constraint that spans the codebase goes in this file, a domain's in its doc.
  - Allowed comment-shaped lines are functional directives only: `oxlint-disable…`, `/// <reference …>`, `@ts-…`. Rust attributes (`#[expect(…)]`, `#[cfg(…)]`) and a lint exception's `reason` are not comments.
  - `bun run lint:comments` (`scripts/comments.ts`, part of `lint` and `lint:fix`) checks every TS file through `oxc-parser`, C++, Filament material (`.mat`, `.glsl`) and Lua files line by line outside string literals, and CSS, Rust, Zig and Qt style sheet files line by line, in `crates`, `scripts` and the root `*.config.ts`, skipping git-ignored files and `generated/` folders. TypeScript 7 has no parser API; its native compiler only type-checks.
- **TS file names are kebab-case** (`glyph-sets.ts`); the code inside keeps its usual names (`GLYPH_SETS`).
  - oxlint enforces it (`unicorn/filename-case`).
  - Rust module files are snake_case, because a module name must be an identifier.
  - Rename a TS file with the TypeScript rename edits (`getEditsForFileRename`, what VS Code calls on file move) so imports follow, then search for the old name in strings, configs and docs.
- **Every function is a declaration: `function name(…) { … }`, never a function assigned to a `const`**, at module level or inside another function. Inline callbacks passed as arguments stay arrows; installed code keeps its upstream style.
  - `opsis/use-function-declarations` enforces it and `bun run lint:fix` rewrites what it safely can. It leaves to you a `const` with a type annotation, a named function expression and an arrow that uses `this` or `arguments`.
  - A nested declaration is hoisted, so it doesn't see a narrowing of the outer scope (`if (!gl) return`): pass the checked value in, or move the code after the check into a function that takes it.
  - `opsis/use-find-by-id` and `opsis/use-type-guards` also rewrite code and add the import from `scripts/lib/common/`. Two fixes that add imports to one file land on separate passes: run `lint:fix` again until it reports nothing. Every Opsis rule and where it applies is in [toolchain.md](agent-docs/toolchain.md).
- **Never hand-edit installed or generated code.** Regenerate it:

| Path                                                                                    | Owner                                                                                                                                                                                                                           | Regenerate with                                                                                    |
| --------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| `public/fonts/bundled/**`, `crates/compile/src/fonts/bundled.generated.json`            | Fontsource packages (`@fontsource-variable/*`)                                                                                                                                                                                  | `bun run fonts:bundle` (list in `scripts/fonts-bundle.ts`)                                         |
| `brand/icons/*.png`                                                                     | `brand/logo.svg` (the one source for the app icon)                                                                                                                                                                              | `bun run brand:icons` (renders with the recording browser; ImageMagick draws the gradients wrong)  |
| `resources/**` (gitignored)                                                             | the packaging stage: the browser's licences                                                                                                                                                                                     | `bun run stage`                                                                                    |
| `crates/compile/src/terminal/themes.generated.json`                                     | VHS's `themes.json` (https://github.com/charmbracelet/vhs, MIT), at the commit and sha256 pinned in `scripts/terminal-themes.ts`                                                                                                | `bun run terminal:themes`                                                                          |
| `crates/script/src/themes.generated.json`, `crates/script/src/highlights.generated.scm` | Helix's code themes and its Lua highlight query (https://github.com/helix-editor/helix, MPL-2.0), at the release and sha256 pinned in `scripts/script-themes.ts`; dark themes only, and the GFDL Modus Vivendi themes left out  | `bun run script:themes`                                                                            |
| `crates/ui/glyphs.generated.json`                                                       | the glyph sets' own dev packages (`GLYPH_SETS` in `scripts/lib/glyph-sets.ts`) for the glyphs the catalogue's node looks name, and `lucide` for the editor's own icons (`ICONS` in `scripts/node-glyphs.ts`)                    | `bun run glyphs:bundle`                                                                            |
| `crates/compile/src/elements/camera/mac-cursors.generated.json`                         | apple_cursor (https://github.com/ful1e5/apple_cursor, GPL-3.0, not on npm): its macOS theme's SVGs for the CSS cursor keywords, with their hotspots                                                                             | `bun run cursors:bundle <path to an apple_cursor checkout>`                                        |
| `crates/effects/textures/paper-noise.png`                                               | Paper Shaders' noise texture, from the root's `@paper-design/shaders` dev package                                                                                                                                               | `bun run paper:noise`                                                                              |
| `public/environments/room/**`                                                           | three.js's `RoomEnvironment` (MIT), ported into `scripts/room-environment.ts`, ray-cast to an equirectangular picture and prefiltered by Filament's `cmgen`                                                                     | `bun run room:environment`                                                                         |
| `.sqlx/`                                                                                | the `sqlx` query macros of `opsis-store`, checked against `crates/store/schema.sql`                                                                                                                                             | `bun run db:prepare` (a throwaway SQLite database made from the schema, then `cargo sqlx prepare`) |
| `scripts/lib/generated/spec.ts`                                                         | the font types `scripts/fonts-bundle.ts` writes (`opsis-compile`)                                                                                                                                                               | `bun run types:export`                                                                             |
| `crates/engine/src/downloads.generated.json`                                            | the recording browser downloads, pinned by sha256 and size (`scripts/engine-downloads.ts`, for the browser version it's given or the one already pinned)                                                                        | `bun run engine:downloads` (about 600 MB into `~/.cache/opsis-dev/engine-downloads`)               |
| `QT_NOTICES.generated.md`                                                               | the static Qt's software bill of materials (`sbom/*.spdx`) for the modules and plugins `crates/qt/build.rs` links, and the licence texts from Qt's sources                                                                      | `bun run qt:notices` (after `bun run setup`)                                                       |
| `scripts/lib/setup/pins.generated.json`                                                 | every download `setup` fetches, pinned by URL, sha256 and size: Zig, the Qt and KDDockWidgets sources, the Skia commit, Filament, the bundled FFmpeg (program, ffprobe and licence per target) and the font and shader packages | `bun run setup pin <item> <version>`                                                               |
| `.setup/**` (gitignored), `~/.cache/opsis-dev/<item>/<hash>`                            | what `setup` fetched and built for this checkout: `paths.json` naming each item's folder, `bin/zig`                                                                                                                             | `bun run setup`                                                                                    |
| `crates/capture/src/playwright/*.generated.*`                                           | `playwright-core`: its injected page script, its fake clock, the US keyboard layout, the Chromium switches and fonts it launches with                                                                                           | `bun run playwright:scripts`                                                                       |

- **Golden files in `fixtures/**` are recorded, never edited by hand.** They hold what the code made for fixed inputs, stored with those inputs, compared through `opsis-golden` (`crates/golden`: numbers within one part in a billion, every difference listed by its path).
  - When a change is intended, record them with `OPSIS_WRITE_GOLDEN=1` (`cargo test -p <crate>`, plus `-- --ignored` for those that need the dev engine's SwiftShader, the renderer program or the internet) and review the diff.
- **No old formats.** Nothing is released, so there is no migration or fallback code for older project files, settings, engine folders or caches.
  - When a format changes, change it everywhere, write `examples/` again (`bun run examples`, `crates/project/src/bin/write-examples.rs`, from `examples()` in `crates/project/src/templates.rs`) and delete local test projects.
  - A test opens every example and fails when one isn't what the templates write; the formatter leaves `examples/` alone so they stay exactly as the app saves a project.
- **The cache version constants are not bumped before the first release.** To get fresh results while verifying, delete the test project's `.opsis/cache` or use a new project. Each constant and what it keys is in [runs.md](agent-docs/runs.md).
- **Do not commit or push.** Leave changes in the working tree unless asked.
- **Large files go under `~/.cache`; `/tmp` is for small scripts and screenshots only.**
  - What the dev process keeps (toolchains, pinned downloads, release builds, e2e and test outputs) lives in `~/.cache/opsis-dev/<name>`, named through `devCache` (`scripts/lib/tool-config.ts`) or `opsis_golden::dev_cache`.
  - What one task needs only for a while (a clone, a download to inspect, a run's builds) goes in `~/.cache/opsis-<name>` and is deleted once the work is done.
- **Long-running processes start with `run_in_background`.**

## 3. Toolchain

- **bun is the package manager and runs every script** (`bun scripts/x.ts`, the e2e runner too); scripts import TypeScript without file extensions. Never run scripts through `tsx` or Node.
- **Every tool config is a `*.config.ts`, except the Rust ones and the files a tool reads only in its own format**: the Rust ones are TOML at the root (`rust-toolchain.toml`, `rustfmt.toml`, `deny.toml`, `.cargo/config.toml`), and TypeScript reads `tsconfig.base.json` and the root `tsconfig.json`. Shared TS tool configs are in `scripts/lib/` (`oxlint-base.ts`, `oxfmt-base.ts`, `vitest-base.ts`, `tool-config.ts`).
- **Rust builds on one pinned nightly, `rust-toolchain.toml`**, with `rust-version` in `[workspace.package]` refusing an older compiler. Moving it touches several files and another repository ([toolchain.md](agent-docs/toolchain.md)).
- **`bun run setup` makes a development machine once** (`scripts/setup.ts`): every pinned download in `scripts/lib/setup/pins.generated.json` and every one-time build, each in `~/.cache/opsis-dev/<item>/<recipe hash>` and redone only when its pins or build recipe change, plus the recording browser (`--no-engine` skips it). It writes `.setup/paths.json` for this checkout, which the build scripts and tests read ([toolchain.md](agent-docs/toolchain.md)).
- **On Linux all C++ is compiled by clang against libc++, which Rust links statically** (`.cargo/config.toml`; the static Qt and Skia are built with clang and libc++ too). Zig 0.17 builds `opsis-vt`'s wrapper, and `crates/vt/build.rs` refuses any other version ([toolchain.md](agent-docs/toolchain.md)).
- **Qt 6.12 and KDDockWidgets 2.5.1 build `opsis-qt`, one static Qt without ICU for development and release**, built by `setup` on the machine; on a Linux development machine `bun run setup --container` builds it and Skia in a `debian:trixie` container instead, and CI never does. Its C++ follows Google style ([toolchain.md](agent-docs/toolchain.md)).
- **Skia `chrome/m157` builds `opsis-skia`**, as static libraries `setup` builds (`scripts/lib/skia.ts`: CPU only, its own FreeType and image codecs, no system fonts, no PartitionAlloc, and Chrome's Linux text gamma and legacy drawing defines so pictures match the browser's); the build writes `consumer.json`, which `crates/skia/build.rs` reads.
- **Filament v1.80.1 builds `opsis-filament`** from its pinned release archive, which `setup` unpacks: `crates/filament/build.rs` compiles `crates/filament/cpp` and links Filament's archives directly, since they use the same libc++.
- **Rust imports are grouped std, external crates, then the workspace's own, one `use` per module, and struct fields use the shorthand.** `rustfmt.toml` keeps the imports so (`group_imports = "StdExternalCrate"`, `imports_granularity = "Module"`, applied only by the nightly).
- **`bun start` (`scripts/start.ts`) builds the renderer program (`opsis-renderer`) and runs the editor (`cargo run -p opsis-ui`)** with `setup`'s FFmpeg.
- **To see a change in the running app, use the `check-app` skill** (`.claude/skills/check-app`): it seeds `~/.cache/opsis-dev/check` (short, so the API socket fits), starts `bun start` in a virtual display and drives it through the Local API.
- **The environment variables any session may need are these; the ones a single domain's tests or scripts read are in its doc.**
  - `OPSIS_USER_DATA` moves the data folder (default `<config dir>/Opsis`), for tests and side-by-side runs.
  - `OPSIS_FFMPEG` (absolute, read only by builds with debug assertions) names the FFmpeg the app and tests start. `bun start`, `bun run test:rust` and the e2e runner set it to `setup`'s FFmpeg; a test that starts FFmpeg reads it through `opsis_golden::ffmpeg()`, which falls back to `setup`'s.
  - `OPSIS_API_SOCKET` moves the Local API socket ([api.md](agent-docs/api.md)).
  - `OPSIS_AUDIO_DEVICE` (a sound output's id such as `alsa:null`, read only by builds with debug assertions) replaces the default sound output.
  - `OPSIS_WRITE_GOLDEN=1` records golden files.
  - `OPSIS_UPDATE_KEY` and `OPSIS_UPDATE_KEY_PASSWORD` (the ignored `.env` locally, repository secrets in CI) sign update files; `OPSIS_COMMIT` (set by `bun run make`) puts the commit in a release build's About page ([updates.md](agent-docs/updates.md)).
  - `OPSIS_QT`, `OPSIS_KDDW_SOURCE`, `OPSIS_SKIA` and `OPSIS_FILAMENT` name another Qt, KDDockWidgets source, Skia build and Filament than `setup`'s ([toolchain.md](agent-docs/toolchain.md)).
  - The domain ones: `OPSIS_QT_*` in [e2e.md](agent-docs/e2e.md); `OPSIS_ENGINE_DIR` and `OPSIS_TAKE_DIR` in [capture.md](agent-docs/capture.md); `OPSIS_STILLS_OUT` in [elements.md](agent-docs/elements.md); `OPSIS_HEADLESS_SHELL`, `OPSIS_APPIMAGETOOL`, `OPSIS_APPIMAGE_RUNTIME` and `OPSIS_CHECK_*` in [packaging.md](agent-docs/packaging.md); `OPSIS_TRIXIE_SCRIPT` in [toolchain.md](agent-docs/toolchain.md); `OPSIS_QT_NO_ITEM_CACHE` in [profiling.md](agent-docs/profiling.md).

## 4. Verifying

- **Verify with `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`.** The `:fix` variants fix and report the rest; do not run the non-fix variants first.
  - `lint:fix` runs oxlint, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo deny check advisories licenses` (`lint:deny`), the Zig wrapper build (`lint:zig`), `clang-tidy` on the C++ (`lint:cpp`) and the comment check.
  - `format:fix` runs oxfmt (TS, JSON and Markdown), `cargo fmt --all`, `zig fmt` (`format:zig`) and `clang-format` on the C++; `test` runs the scripts' Vitest (`test:ts`) and `cargo test --workspace` (`test:rust`).
- **Clippy runs `pedantic` and `nursery` as warnings and `-D warnings` fails on any of them.** The permanent workspace allows and why each exists are in [toolchain.md](agent-docs/toolchain.md); never add a workspace line.
- **A lint exception is local, on the item or statement, and written `#[expect(lint, reason = "…")]`**, the reason saying why the flagged code is meant; `unfulfilled_lint_expectations` reports it once the lint stops firing.
  - An exception that holds in only some builds sits in a `cfg_attr` for those builds.
  - `#[allow]` is only for code where an expectation can't hold in every build that compiles it (test helpers shared by several test binaries, `crates/capture/tests/support/mod.rs`), as an inner `#![allow(…, reason = "…")]`. Clippy's `allow_attributes` and `allow_attributes_without_reason` fail the rest.
  - An oxlint suppression is `// oxlint-disable-next-line <plugin>/<rule>`.
- **A pure function with an invariant (a round trip, a bound, an order, a count) is tested as a property over generated inputs**: `proptest` in Rust, `fast-check` in TS. A hand-written case only pins exact text people see; a test never restates a lookup table or checks a pure function against itself.
- **TS tests are `*.test.ts` on Node, in `scripts`.**
- **Scripts that need the catalogue read it from Rust through the `editor-data` binary** (`crates/compile/src/bin/editor-data.rs`, built in release): `glyphs:bundle`, the profiler, and the root's Vitest run, whose global setup builds it (`scripts/lib/build-editor-data.ts`).
- **`cargo test -p opsis-ui` checks the native views on Qt's offscreen platform**, through the `views` scenario, so `bun run test` covers them without a display ([e2e.md](agent-docs/e2e.md)).
- **Tests that need the dev engine are `#[ignore]`d with the reason**; run them with `cargo test -p <crate> -- --ignored`. The element goldens and 3D stills are among them, since they draw with the engine's SwiftShader ([film.md](agent-docs/film.md), Goldens).
- **End-to-end tests run only in a virtual display, never on the real screen, and never move the real pointer** ([e2e.md](agent-docs/e2e.md)).

## 5. Dependencies

- **Dependencies go through the package managers**: `bun add`, `bun add -d`, `bun remove` for TS (inside the folder that imports it), `cargo add` / `cargo remove` for Rust. Never edit dependency blocks by hand, except the root table below.
- **Every Rust dependency's version lives once in the root `Cargo.toml`'s `[workspace.dependencies]`.**
  - The workspace's own crates are path entries (`opsis-types = { path = "crates/types" }`); every other crate sits at its loose major (`"1"` from 1.0 on, `"0.<minor>"` before).
  - Crates take them with `dep.workspace = true` or `dep = { workspace = true, features = [...] }`, in `[dependencies]`, `[dev-dependencies]` and `[build-dependencies]`. `cargo add -p <crate> <dep>` reuses an entry already in the root table.
  - A new dependency needs its pin added to the root table by hand (the only dependency block edited by hand), then `cargo add -p`.
  - A root entry carries only the features every crate that uses it needs, since crates can add features but never remove them, and `default-features = false` wherever any crate turns them off; a crate that wants them says `default-features = true` (`image`).
- **`Cargo.lock` holds exact versions, and adding or updating a dependency takes the latest release**, except these pins:
  - `specta` `=2.0.0-rc.28` and `specta-typescript` and `specta-serde` `=0.0.15` in the root table: prereleases whose API changes between releases and whose output is `scripts/lib/generated/spec.ts`, so they move together and only with `bun run types:export`.
  - Skia's commit and Filament's release are pinned in `scripts/lib/setup/pins.generated.json`; each moves only with `bun run setup pin` and the goldens recorded again.
  - Ghostty's terminal engine is pinned in `crates/vt/zig/build.zig.zon` to the source tarball of libghostty-vt 1.0.0 (tag `vt-1.0.0`, commit `0fa3d57`) by URL and hash, written with `zig fetch --save=ghostty <url>` in `crates/vt/zig`, never by hand. Ghostty's build accepts only the major and minor Zig version its `minimum_zig_version` names (`requireZig`), so a Ghostty pin that names a new minor moves Zig too.
- **Every dependency must be GPLv3-compatible** ([packaging.md](agent-docs/packaging.md) has the allowed licences and the notices).

## 6. Workspace and layout

- **A Cargo workspace (`Cargo.toml`, members `crates/*`, `Cargo.lock` at the root) beside one Bun package for the scripts** (`package.json`, `bun.lock`).
- **TypeScript is only the scripts**: `scripts/` and the skills' scripts, one `tsconfig.json`, `oxlint.config.ts` and `vitest.config.ts` at the root. Scripts import by relative path; plain helpers live in `scripts/lib/common/` and the generated font types in `scripts/lib/generated/`.
- **Installs are bun's isolated layout**: a script imports only what the root declares.

```
crates/                Rust, one domain per crate (table below)
fixtures/              golden files
examples/              example projects, written by `bun run examples`
public/                the app's files (`app:` refs): fonts, sound effects (`sfx`), music, placeholders, backdrops, environments
scripts/               bundling, pinning, packaging, the oxlint plugin, shared tool configs
plans/                 planned work
agent-docs/            the domain docs (§8)
```

- **Rust crates live in `crates/<domain>`, named `opsis-<domain>`, one domain each.**
- **Only `opsis-qt` opens windows or dialogs**; `opsis-session` asks its host through `Platform`.

| Crate             | Domain                                                                                                                                                                                                                                                                       |
| ----------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `opsis-types`     | every shape that crosses a boundary, and `numbers`                                                                                                                                                                                                                           |
| `opsis-compile`   | node and element definitions, settings, compile, cards, validation, the Inspector context, the timeline flattener, chapters, the mix plan, the catalogue, the edits (`edits/`), `editor-data`                                                                                |
| `opsis-store`     | the app database (`schema.sql` and its queries), settings, view state, recent projects, font rows, the keyring key and sealed sign-ins                                                                                                                                       |
| `opsis-document`  | the open project's document: workflow, undo and redo, gestures, revisions, the saved revision and file version, the nodes each change edited                                                                                                                                 |
| `opsis-project`   | project folders: create from `templates.json`, open and save with versions, watching, asset and model import, file stamps, the last run, `examples()`                                                                                                                        |
| `opsis-fonts`     | the font library, the computer's fonts and online fonts                                                                                                                                                                                                                      |
| `opsis-engine`    | engine paths, the browser installer, the bundled FFmpeg, tool helpers (`EngineFiles`), licences, child temp folders and processes, file and hash helpers, `PICTURE_VERSION` and `PREPARED_VERSION`                                                                           |
| `opsis-traces`    | the trace store                                                                                                                                                                                                                                                              |
| `opsis-runner`    | the run: plan, cache keys, saved results, run events and progress                                                                                                                                                                                                            |
| `opsis-audio`     | playback sound: the mix stem decoded through FFmpeg and played through cpal, its clock, the renderer                                                                                                                                                                         |
| `opsis-media`     | FFmpeg argument builders, runs and probes behind `MediaTools`, the media and sound executors, mix stems, loudness, waveform and filmstrip analysis                                                                                                                           |
| `opsis-render`    | film renders and stills on the native renderer (`native.rs`, `chunks.rs`), saved pictures, the output executors, take previews, `render-elements`                                                                                                                            |
| `opsis-prepare`   | prepared files: 3D typefaces, models, SVG logos, textures                                                                                                                                                                                                                    |
| `opsis-image`     | the image node executors                                                                                                                                                                                                                                                     |
| `opsis-footage`   | the film's footage: one decoder per source, its grade and redactions, frames handed over decoded, terminal frames                                                                                                                                                            |
| `opsis-probe`     | reads a media file's length, container, picture and sound from its own bytes, for every probe the app makes                                                                                                                                                                  |
| `opsis-vt`        | Ghostty's terminal engine behind one safe `Terminal`; one of the crates allowed `unsafe`                                                                                                                                                                                     |
| `opsis-terminal`  | terminal takes: shells, sessions, the driver, takes, casts, tapes, the live feed, prepared frames                                                                                                                                                                            |
| `opsis-capture`   | the recording browser over CDP: sessions, selectors, actions, the screencast encoder, smooth takes, sign-in steps, stills                                                                                                                                                    |
| `opsis-stream`    | the streamed views (sign-in, element picker)                                                                                                                                                                                                                                 |
| `opsis-api`       | the Local API                                                                                                                                                                                                                                                                |
| `opsis-script`    | Lua scripts: the Lua 5.5 runtime, the sandbox that holds a script to the open project's files, the `opsis` module over a `ScriptHost`, and the Script panel's highlighting and editing rules                                                                                 |
| `opsis-session`   | the open project and everything windows follow, with no window code: document, compiles, runs, scripts, shared state, settings, fonts, engine, streamed views, the exit decision, the Local API host                                                                         |
| `opsis-qt`        | the native editor's view layer: Qt 6.12 Widgets and KDDockWidgets built from Cargo, the C++ views, the `cxx` bridge behind one safe `Host` trait; one of the crates allowed `unsafe`                                                                                         |
| `opsis-scene`     | what every element returns: nodes in the film's CSS and SVG subset, the clock and its maps, the animation maths, content hashes, the timeline walker, motion blur                                                                                                            |
| `opsis-layout`    | layout: scene nodes into positioned boxes on Taffy (block, flex, grid, positioning) and Parley (text), with the browser's line metrics and transforms                                                                                                                        |
| `opsis-paint`     | painting: laid-out scene nodes drawn through `opsis-skia` in CSS stacking order, colours and gradients, borders, shadows, text from the layout's glyph runs, pictures, effects, clip paths, masks and inline SVG through `usvg`, the frame caches                            |
| `opsis-effects`   | the shader materials and the renderer program `opsis-renderer`, which runs the passes and 3D stages on Filament                                                                                                                                                              |
| `opsis-renderer`  | the client the film starts the renderer program with: the driver choice, the program's lookup, the pipe protocol                                                                                                                                                             |
| `opsis-geometry`  | three.js's geometry ported: primitives, shapes and extrusions with `earcut`, tubes                                                                                                                                                                                           |
| `opsis-stage3d`   | 3D stages on `opsis-filament`: materials, lights, the room's reflections, models fitted from their bounds                                                                                                                                                                    |
| `opsis-film`      | one film frame by frame: walks the spec into a scene, lays it out, paints it and starts the renderer program for effects and 3D; the live terminal painter                                                                                                                   |
| `opsis-skia`      | Skia (`chrome/m157`, linked statically from `~/.cache/opsis-dev/skia`) behind a safe Rust API, through the `cxx` bridge to its C++ wrapper; one of the crates allowed `unsafe`                                                                                               |
| `opsis-filament`  | Filament (shader passes and 3D) behind a safe Rust API over the plain C functions of its C++ wrapper; materials compiled by `matc` at build time; one of the crates allowed `unsafe`                                                                                         |
| `opsis-update`    | updates: which installer made this copy, the release feed, the signed download and the install step                                                                                                                                                                          |
| `opsis-ui`        | the native editor's state and logic behind the views, implementing `Host` over `opsis-session`; the `opsis` binary                                                                                                                                                           |
| `opsis-e2e-input` | dev-only: holds a virtual pointer and keyboard on a headless Wayland compositor and carries out the e2e's real input requests                                                                                                                                                |
| `opsis-golden`    | dev-only: the golden-file helper and picture goldens (`pictures.rs`), `dev_cache`, `ffmpeg()`, and the readers and builders more than one crate's tests use (`crates/golden/src/lib.rs`)                                                                                     |
| `opsis-cases`     | dev-only: the CSS and SVG property cases as scene nodes, which the layout and paint goldens draw                                                                                                                                                                             |
| `opsis-macros`    | procedural macros (`proc-macro2`, `syn`, `quote`; no `paste` or `pastey`, since `quote` and `format_ident!` build every name)                                                                                                                                                |
| `opsis-packager`  | dev-only: the installers `bun run make` builds with each format's own tool (deb, rpm, pacman, AppImage, NSIS, MSI, the macOS app and dmg), each program marked with its installer, and the signed update files, after checking the program loads no Qt, ICU or KDDockWidgets |

- **Test helpers only one crate's tests share live in its `tests/support/mod.rs`**; those several crates share go in `opsis-golden`.

## 7. Rules across the codebase

### Boundaries and types

- **Node-only code lives in `scripts`; everything the app does is Rust.**
- **Rust is the source of every type that crosses a boundary** (the editor's views and calls, the timeline spec, run events). The editor's are the `cxx` bridge's structs ([ui.md](agent-docs/ui.md)); the font types the scripts write are exported with specta (`bun run types:export`) into `scripts/lib/generated/spec.ts`. Never write such a type by hand in TS; a test fails when the export is stale.
  - JSON Schema for the Inspector and the Local API comes from the same types (`schemars`).
- **Whole numbers are integers**: frame lengths, sizes, frame rates, counts and settings in milliseconds are `u32`, a sound's start (which can fall before its clip) `i32`, timestamps `i64` milliseconds since the epoch. Only values that can be fractional (positions in a remapped clip, action times, volumes, rates) are `f64`.
- **No number is converted with `as`.** Widening uses `f64::from`, integers `try_from`, everything else the functions in `opsis_types::numbers` (`crates/types/src/numbers.rs`).
  - Sizes read in the app are written in binary units through `numbers::byte_size` ("1.5 KiB").
  - The `pedantic` cast lints fail a new cast.
- **Floats are compared exactly only through `exact_eq`** (a setting at its neutral value, whole numbers kept as `f64`, exact properties in tests). Clippy's `float_cmp` skips functions with `eq` in their name, so `exact_eq` needs no exception.
- **Kinds that form a fixed set are enums** (`BrowserAction`, `Sign`), and node types the code names are constants next to their definitions (`OPEN_BROWSER_TYPE`, `RENDER_TYPE`).
- **Each module has its own error enum (`thiserror`: `SettingsError`, `CompileError`, `StartupError`…)**, turned into text only where it reaches a person or a JSON field. A missing value falls back to a default only when the default is what it means.
- **Every public Rust type implements `Debug`** (`missing_debug_implementations`): it derives it, private types it holds derive it too, and where a field has none (a closure, a trait object) it writes `Debug` by hand, listing the fields that have it and ending with `finish_non_exhaustive`.
- **`unsafe_code` is denied in every crate but `opsis-vt`, `opsis-qt`, `opsis-skia` and `opsis-filament`** (`[workspace.lints.rust]`), whose `lib.rs` allow it with a reason. `opsis-vt` keeps its `unsafe` in `src/ffi.rs` and `src/terminal.rs`; `opsis-qt`'s and `opsis-skia`'s is the code the `cxx` bridge expands to (`src/bridge.rs`); `opsis-filament`'s is the calls into its C functions (`src/lib.rs` over `src/ffi.rs`).
- **Cached JSON is read into typed structs (`read_json`, `crates/engine/src/files.rs`), never trusted as-is.**

### The film

- **Every frame is a pure function of its frame number** ([scene.md](agent-docs/scene.md)): an element is a function of its settings, its clock and its stage, and never reads a wall clock, a random source, a file or the network.
  - Randomness uses `random(seed)`; time comes in on the `Clock`, never a global; colours between stops mix in OKLCH; files reach an element only through `Measure` and the resolved refs.
  - The renderer's contract is in [film.md](agent-docs/film.md).
- **Nodes never generate film code.** A node has ports, settings and an optional pure `compile`; compile produces a `TimelineSpec` and one walker, `draw_film`, turns it into scenes ([nodes.md](agent-docs/nodes.md)).
- **A node does one thing on one kind of input, and variants of that thing are one node with a Kind**: every new or changed node follows the rules in [nodes.md](agent-docs/nodes.md), What a node is.
- **An element is one Rust file: its settings, its `definition()` and its drawing**, reached by its key through `draw.rs` ([elements.md](agent-docs/elements.md)).
- **Every value a user would want to change is a setting** in the Inspector; only timing the Timeline owns, animation tuning and functions stay in code ([inspector.md](agent-docs/inspector.md)).
- **Never load fonts, images, scripts or sounds from a remote URL at render time.** Element defaults use neutral copy and `app:` placeholders (`public/placeholders`, `public/backdrops`).
- **A change that moves pictures records the goldens again and is reviewed**; picture goldens are exact on x86 Linux with AVX2 ([film.md](agent-docs/film.md), Goldens).

### The app

- **Nothing uses the internet unless the user asks.** The app works offline; anything that needs the network (setting up the engine, getting fonts online) sits behind a button and asks first, and what it downloads is saved locally (`<data folder>/engine`, `<data folder>/fonts`). Updates are checked only on their button, or at start when the user turns that on ([updates.md](agent-docs/updates.md)).
- **Never hold a lock across a call that may take it again** (the settings lock and `announce`, which reads the settings): a database lock held that way hangs the close.
- **Nothing blocks on the async runtime.** The keyring is the one thing made before the session starts (`start`, `crates/ui/src/main.rs`), and a run reaches it only through `off_runtime` (`crates/capture/src/take.rs`, [app-state.md](agent-docs/app-state.md)), because the Linux Secret Service store blocks inside `zbus` and panics on a runtime thread.
- **Register long app work in `AppState.tasks`** (`begin(label)`/`end(label)`, `crates/session/src/state.rs`) so the exit dialog lists it.
- **Never write intermediates to the system temp folder from the app**; every child program gets its own temp folder ([engine.md](agent-docs/engine.md)).
- **Views that read a node by id handle it being gone**: a panel can refresh once more after the node is deleted.
- **The editor keeps no state of its own**; app state is the database in Rust ([app-state.md](agent-docs/app-state.md)).
- **Every Local API route is documented in `crates/api/src/openapi.rs`** ([api.md](agent-docs/api.md)).
- **A script reaches only the open project's files and never the network** ([scripting.md](agent-docs/scripting.md)): its `io`, `os`, `require` and `dofile` resolve every path beneath the project folder, and the `opsis` module binds no call that downloads, opens another project or imports outside files.

### User-facing copy

- **No names of the libraries, tools or formats a person never works with, in node titles, descriptions, field labels, badges, toasts or error messages** (Playwright, Chromium, FFmpeg, CRF, DPR, page.evaluate…), and never Qt, Rust, Bun, CDP, a crate or the services behind a download.
  - A name a person has to work with is shown by name: the language they write scripts in (Lua), the kind of service they choose and give a key for (OpenAI-compatible).
  - Settings → Licences and About list what Opsis includes by name.
  - Technical settings go under `advanced`.
- **Sentence case; durations in seconds.**
- **Browser nodes are "actions", never "steps".**
- **Elements are described in words** (`element_name`); element descriptions are on their entries.
- **The node's name (`node.label`) is also its camera, chapter and caption label.**
- **Categories keep their internal ids**; display names come from `Category::label` (`crates/compile/src/node.rs`), sent with the catalogue.
- **`crates/compile/tests/unit/nodes_c.rs` enforces descriptions and bans the names in its list from node titles, descriptions and field labels**; the other copy (badges, toasts, errors) is kept to these rules by hand.

## 8. Domain docs

| Doc                                     | Domain                                                                                                                                                           |
| --------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [toolchain.md](agent-docs/toolchain.md) | moving the Rust nightly, Zig and the Ghostty build, Qt and KDDockWidgets, Skia and Filament, the C++ checks, the Clippy workspace allows, the Opsis oxlint rules |
| [nodes.md](agent-docs/nodes.md)         | what a node may be, node definitions, kind groups, ports and resources, conversions, file needs, claims, executors, growing inputs, cards, looks and glyphs      |
| [inspector.md](agent-docs/inspector.md) | `#[settings]` and `#[choice]`, schema keywords, widgets, `InspectorContext`, form behaviour, what is a setting                                                   |
| [elements.md](agent-docs/elements.md)   | element files, which node draws an element, themes, props, lengths and the 30 fps clock, design size, stills and goldens, 3D elements                            |
| [scene.md](agent-docs/scene.md)         | scene nodes, the clock and its maps, the timeline walker, measuring, content hashes and the caches                                                               |
| [film.md](agent-docs/film.md)           | the renderer: layout, paint, effects and 3D, the drawing choice, output, goldens, transitions, other shapes, window frames                                       |
| [capture.md](agent-docs/capture.md)     | browser sessions, takes, recording style, click and drag, the screencast, smooth takes, streamed views and sign-in, traces, screenshots                          |
| [terminal.md](agent-docs/terminal.md)   | `opsis-vt`, terminal takes, tapes and casts, the live tape window, previews, live pictures, terminal grading, `terminal_view`                                    |
| [footage.md](agent-docs/footage.md)     | footage uses, the colour grade list, Apply grade, Grade footage, Redact take, the Mac cursor                                                                     |
| [assets.md](agent-docs/assets.md)       | asset refs and `hosted`, the files the renderer reads, prepared files, model import, followed project files                                                      |
| [audio.md](agent-docs/audio.md)         | built-in sounds and music, the Mixer, stems and ducking, sound nodes and their filter traps, action and transition sounds                                        |
| [images.md](agent-docs/images.md)       | the image nodes: loading, depth, areas, size planning, text drawing                                                                                              |
| [timeline.md](agent-docs/timeline.md)   | the flattener, handles and timeline edits, keyframes, the Timeline clock, chapters in exports                                                                    |
| [editor.md](agent-docs/editor.md)       | windows, docks and panels, the title bar and menus, canvas, look and motion, starting and closing, Settings, view state                                          |
| [ui.md](agent-docs/ui.md)               | the native editor's bridge: Rust and C++ sides, views, events, frames, gestures, C++ rules                                                                       |
| [document.md](agent-docs/document.md)   | the document in Rust, edits and gestures, compiles, shared state, live frames, autosave and disk changes                                                         |
| [app-state.md](agent-docs/app-state.md) | the app database, settings, the keyring, projects folder                                                                                                         |
| [api.md](agent-docs/api.md)             | the Local API: socket, routes, `ApiHost`, OpenAPI                                                                                                                |
| [scripting.md](agent-docs/scripting.md) | Lua scripts: the runtime, the sandbox, the `opsis` module, running and stopping, the Script panel, highlighting and editing rules                                |
| [runs.md](agent-docs/runs.md)           | run state, results following keys, media file checks, cache keys and version constants                                                                           |
| [engine.md](agent-docs/engine.md)       | the recording browser, the bundled FFmpeg, probing, child temp folders and processes                                                                             |
| [fonts.md](agent-docs/fonts.md)         | bundled and terminal fonts, the library, project fonts and roles, `family_for`                                                                                   |
| [packaging.md](agent-docs/packaging.md) | `bun run make`, staging, the static Qt, `opsis-packager` and the installers, update signing and the feed, icons, licences and notices                            |
| [updates.md](agent-docs/updates.md)     | the updater: installer marker, feed, signed downloads, install at exit, the About page                                                                           |
| [e2e.md](agent-docs/e2e.md)             | the e2e scenarios, their virtual display and headless Wayland, data folders, the offscreen view tests                                                            |
| [profiling.md](agent-docs/profiling.md) | the profiling build, `bun run profile` in Docker, per-node CPU and memory, the size tools                                                                        |

### Writing AGENTS.md and agent-docs

- **AGENTS.md holds what any session may need**: the hard rules, the toolchain, verifying, dependencies, the layout and the rules that cross domains. Everything else goes in the domain doc.
- **The docs hold only what the code can't say**: the why, the flows across files and crates, invariants, traps and deliberate limits, and dependencies across projects. Never a line-by-line description of the code.
- **Point to code by exact path and symbol** (`crates/compile/src/style.rs` → `restyle_with`). Never copy what the code already lists (node catalogues, enum variants, columns, tuning constants); name the file that holds it.
- **Each fact has one home doc**; other docs link to it.
- **Present tense, current state only.** No changelog language ("now", "no longer", "was", "used to", "renamed", "previously", "once" for "formerly"); history lives in git.
- **One rule per bullet, the rule first and in bold; mechanism goes in sub-bullets.**
- **Every doc has the same shape:** a title; a short scope paragraph naming what it covers, what its paths are relative to and the related docs; then `## Code map`, `## Rules`, `## How it works` and `## Deliberate limits`, leaving out any section it has nothing for.
- **A change that makes a doc wrong updates that doc in the same change**; a plan in `plans/` that is built is deleted, its design moving into the docs.
