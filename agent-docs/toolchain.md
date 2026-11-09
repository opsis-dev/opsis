# Toolchain

The Rust nightly and how to move it, the Zig build of `opsis-vt`'s wrapper, Qt and KDDockWidgets for `opsis-qt` and the release build's static Qt, the C++ checks, the Clippy workspace allows and the repository's own oxlint rules. Paths are relative to the repository root. The everyday commands and the lint exception format are in [AGENTS.md](../AGENTS.md) §3 and §4; Ghostty itself is in [terminal.md](terminal.md), the editor's look and docks in [editor.md](editor.md), the packaging container in [packaging.md](packaging.md).

## Code map

| Concern                                | Where                                                                                                                                 |
| -------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| Nightly pin                            | `rust-toolchain.toml` (with `rustfmt` and `clippy`)                                                                                   |
| Minimum compiler                       | `rust-version` in `[workspace.package]`, `Cargo.toml`, inherited by every crate                                                       |
| CI toolchain steps                     | every `dtolnay/rust-toolchain` step of `.github/workflows/build.yml` and `.github/workflows/release.yml`                              |
| Workspace lints                        | `[workspace.lints.rust]`, `[workspace.lints.clippy]` in `Cargo.toml`; `[lints] workspace = true` in every crate                       |
| Zig build of the wrapper               | `crates/vt/build.rs` → `zig_target`; `crates/vt/zig/build.zig`, `crates/vt/zig/build.zig.zon`                                         |
| Zig in CI                              | `scripts/zig-install.ts` (`bun run zig:install`) → `ZIG_VERSION`, `ARCHIVES`                                                          |
| Opsis oxlint rules                     | `scripts/oxlint-plugin-opsis.ts`                                                                                                      |
| Shared oxlint, oxfmt, Vitest configs   | `scripts/lib/oxlint-base.ts`, `scripts/lib/oxfmt-base.ts`, `scripts/lib/vitest-base.ts`, `scripts/lib/tool-config.ts`                 |
| Comment check                          | `scripts/comments.ts`                                                                                                                 |
| Qt and KDDockWidgets for `opsis-qt`    | `scripts/qt-install.ts` (`bun run qt:install`); `crates/qt/build.rs` → `QT_VERSION`, `QT_MIN_MINOR`, `KDDW_VERSION`, `STATIC_PLUGINS` |
| The release build's static Qt          | `bun run qt:static` → `scripts/qt-static.ts`; `scripts/lib/static-qt.ts` → `ensureStaticQt`; `scripts/lib/trixie.ts` → `runInTrixie`  |
| Qt in CI                               | `jurplel/install-qt-action` in `.github/workflows/build.yml`; `bun run qt:static` in `.github/workflows/release.yml`                  |
| C++ format and lint                    | `scripts/cpp.ts` → `STYLE`, `CHECKS`; `compile_commands.json` written by `crates/qt/build.rs`                                         |
| Rust test runner (stages FFmpeg first) | `scripts/test-rust.ts`                                                                                                                |

## Rules

### Rust

- **The computer keeps one nightly, the one in `rust-toolchain.toml`.** To move it, in one change:
  - change the date in `rust-toolchain.toml` and in every `dtolnay/rust-toolchain` step of `.github/workflows/build.yml` and `.github/workflows/release.yml`;
  - raise `rust-version` to the new compiler's;
  - run `rustup default` with the new toolchain and uninstall the old one;
  - move `~/projects/agent-os/rust-toolchain.toml` to the same date and leave it uncommitted there.
- **The Clippy workspace allows are permanent and no line is added to them.** Each exists because:
  - `missing_errors_doc`, `missing_panics_doc`: they ask for doc comments, which the no-comments rule bans.
  - `option_if_let_else`: it would turn every `if let … else` into `map_or_else`.
  - `too_many_lines`: it caps the length of a function.
  - `must_use_candidate`: it asks for `#[must_use]` on every function whose result could be ignored.
  - `struct_excessive_bools`: it flags structs with several `bool` fields, mostly settings structs with their switches.
  - `significant_drop_tightening`: it guesses a lock's lifetime from its last use, so it flags guards something still borrows and locks held across an `await` on purpose. A lock that blocks others needlessly is still tightened by hand (`crates/session/src/node_results.rs` → `check_media_files`).
  - `module_name_repetitions`: it flags item names that repeat their module's name; it sits in `restriction`, and the allow keeps it off should Clippy move it into `pedantic`.

### Zig

- **Zig 0.17 builds the wrapper because Ghostty needs the same major and minor version.** `crates/vt/build.rs` runs `zig build` (the `ZIG` variable names another program) and refuses any other version.
- **`build.rs` always names the Zig target (`zig_target`)**, because Zig's native target compiles for the building computer's processor.
- **CI installs Zig with `bun run zig:install` in every job that builds Rust and runs `zig build --fetch` before Cargo**: both jobs of `build.yml` and the package job of `release.yml`.
  - `scripts/zig-install.ts` pins each target's archive from ziglang.org by sha256 and size and unpacks it into `~/.cache/opsis-dev/zig`; the job puts it on `PATH` and caches that folder with Zig's global cache, so ziglang.org is asked only when the cache is empty. Moving Zig changes `ZIG_VERSION` and `ARCHIVES` there, `ZIG_MAJOR_MINOR` in `crates/vt/build.rs` and `minimum_zig_version` in `build.zig.zon`, and needs a Ghostty pin that builds with it.
  - Zig unpacks the packages into `crates/vt/zig/zig-pkg` (ignored by git); the tarballs stay in Zig's global cache.

### Qt and KDDockWidgets

- **`opsis-qt` builds against the Qt and KDDockWidgets that `QT_VERSION` and `KDDW_VERSION` name, from `~/.cache/opsis-dev/qt`**, which `bun run qt:install` fills: Qt through aqtinstall (`uvx`), KDDockWidgets from its release tarball, pinned by URL, sha256 and size. `build.rs` never downloads; it refuses any Qt but Qt 6 from `QT_MIN_MINOR` up and names `qt:install` when either is missing.
  - `OPSIS_QT` names another Qt and `OPSIS_KDDW_SOURCE` another KDDockWidgets source.
  - A static Qt (one with `libQt6Core.a`) is linked statically, with its platform plugins imported by name (`STATIC_PLUGINS`) and linked as one group; the shared Qt is linked with an rpath.
  - KDDockWidgets builds against Qt's private Widgets headers, so moving Qt means rebuilding and re-checking it.
  - The Qt version is written in `crates/qt/build.rs`, `scripts/qt-install.ts`, `scripts/lib/static-qt.ts` (with its pinned source tarballs) and both `install-qt-action` steps of `build.yml`, and moves in all of them at once; KDDockWidgets' in `build.rs` and `qt-install.ts`.
- **CI's Qt depends on the workflow**: `build.yml` points `OPSIS_QT` at `jurplel/install-qt-action`'s shared Qt; `release.yml` builds the static Qt with `bun run qt:static`, caches it, and sets `OPSIS_QT` from the folder it reports. Both then run `bun run qt:install`, which uses the Qt in `OPSIS_QT` when that folder has a `lib` folder (and downloads one otherwise) and fetches KDDockWidgets.
- **Release builds link the static Qt `bun run qt:static` builds (`ensureStaticQt`)**: Qt Base and Qt Svg from the pinned source tarballs, without ICU or OpenGL, into `~/.cache/opsis-dev/qt-static/<version>/<folder>`, marked done by an `.opsis-complete` file.
  - It builds only on Linux, locally and in CI, inside the `debian:trixie` packaging container (`runInTrixie`, so Docker is needed), because a static link needs X11 and xcb development files a desktop doesn't have. On any other platform `ensureStaticQt` refuses unless a finished build is already there ([plans/release.md](../plans/release.md) section 2).

### C++

- **C++ is formatted to Google style with namespace comments off (`STYLE`)**, since closing-brace comments are comments. `clang-format` and `clang-tidy` run from their PyPI builds through `uvx`, pinned in `scripts/cpp.ts`.
- **`lint:cpp` runs `clang-tidy` with the `bugprone`, `modernize` and `cppcoreguidelines` checks on the `.cc` files of `crates/qt/cpp` only, minus the ones in `CHECKS` that fight Qt or the code's style**: owning memory (Qt's parents own widgets) and member initializers (Qt builds child widgets in the constructor body, in layout order) among them.
  - It first runs `cargo check -p opsis-qt`, which writes the `compile_commands.json` it reads, so it needs Qt installed.

### TypeScript

- **The Opsis oxlint rules that fix code add their import from `common/`**, so run `lint:fix` until it reports nothing ([AGENTS.md](../AGENTS.md) §2).

## Deliberate limits

- **TypeScript 7 has no parser API**, so the comment check parses TS with `oxc-parser`; the native compiler only type-checks.
- **`lint:zig` builds the wrapper in Debug as a check** into `target/zig-check`; it is not the build Cargo links.
- **The static Qt, and so the release packages, isn't built on macOS or Windows**: the release workflow's jobs there fail at `qt:static` unless their cache already holds a finished build.
