# Toolchain

The Rust nightly and how to move it, the Zig build of `opsis-vt`'s wrapper, the Clippy workspace allows and the repository's own oxlint rules. Paths are relative to the repository root. The everyday commands and the lint exception format are in [AGENTS.md](../AGENTS.md) §3 and §4; Ghostty itself is in [terminal.md](terminal.md).

## Code map

| Concern                                | Where                                                                                                                 |
| -------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| Nightly pin                            | `rust-toolchain.toml` (with `rustfmt` and `clippy`)                                                                   |
| Minimum compiler                       | `rust-version` in `[workspace.package]`, `Cargo.toml`, inherited by every crate                                       |
| CI toolchain steps                     | every `dtolnay/rust-toolchain` step of `.github/workflows/build.yml` and `.github/workflows/release.yml`              |
| Workspace lints                        | `[workspace.lints.rust]`, `[workspace.lints.clippy]` in `Cargo.toml`; `[lints] workspace = true` in every crate       |
| Zig build of the wrapper               | `crates/vt/build.rs`, `crates/vt/zig/build.zig`, `crates/vt/zig/build.zig.zon`                                        |
| Zig in CI                              | `scripts/zig-install.ts` (`bun run zig:install`)                                                                      |
| Opsis oxlint rules                     | `scripts/oxlint-plugin-opsis.ts`                                                                                      |
| Shared oxlint, oxfmt, Vitest configs   | `scripts/lib/oxlint-base.ts`, `scripts/lib/oxfmt-base.ts`, `scripts/lib/vitest-base.ts`, `scripts/lib/tool-config.ts` |
| Comment check                          | `scripts/comments.ts`                                                                                                 |
| Qt and KDDockWidgets for `opsis-qt`    | `scripts/qt-install.ts` (`bun run qt:install`); `crates/qt/build.rs` → `QT_VERSION`, `KDDW_VERSION`, `qt_folder`      |
| The release build's static Qt          | `scripts/lib/static-qt.ts` (`bun run qt:static`), `scripts/lib/trixie.ts`                                             |
| C++ format and lint                    | `scripts/cpp.ts` → `STYLE`, `CHECKS`; `compile_commands.json` written by `crates/qt/build.rs`                         |
| Rust test runner (stages FFmpeg first) | `scripts/test-rust.ts`                                                                                                |

## Rules

- **The computer keeps one nightly, the one in `rust-toolchain.toml`.** To move it, in one change:
  - change the date in `rust-toolchain.toml` and in every `dtolnay/rust-toolchain` step of `.github/workflows/build.yml` and `.github/workflows/release.yml`;
  - raise `rust-version` to the new compiler's;
  - run `rustup default` with the new toolchain and uninstall the old one;
  - move `~/projects/agent-os/rust-toolchain.toml` to the same date and leave it uncommitted there.
- **Zig 0.17 builds the wrapper because Ghostty needs the same major and minor version.** `crates/vt/build.rs` runs `zig build` (the `ZIG` variable names another program) and refuses any other version.
- **`build.rs` always names the Zig target** (`x86_64-linux-gnu`, `aarch64-macos`, `x86_64-windows-msvc`…), because Zig's native target compiles for the building computer's processor.
- **CI installs Zig with `bun run zig:install` in every job and runs `zig build --fetch` before Cargo.** `scripts/zig-install.ts` pins each target's archive from ziglang.org by sha256 and size and unpacks it into `~/.cache/opsis-dev/zig`; the job puts it on `PATH` and caches that folder with Zig's global cache, so ziglang.org is asked only when the cache is empty. Moving Zig changes `ZIG_VERSION` and `ARCHIVES` there.
  - Zig unpacks the packages into `crates/vt/zig/zig-pkg` (ignored by git); the tarballs stay in Zig's global cache.
- **The Clippy workspace allows are permanent and no line is added to them.** Each exists because:
  - `missing_errors_doc`, `missing_panics_doc`: they ask for doc comments, which the no-comments rule bans.
  - `option_if_let_else`: it would turn every `if let … else` into `map_or_else`.
  - `too_many_lines`: it caps functions at 100 lines.
  - `must_use_candidate`: it asks for `#[must_use]` on every function whose result could be ignored.
  - `struct_excessive_bools`: it flags structs with more than three `bool` fields, mostly settings structs with their switches.
  - `significant_drop_tightening`: it guesses a lock's lifetime from its last use, so it flags guards something still borrows and locks held across an `await` on purpose. A lock that blocks others needlessly is still tightened by hand (`check_media_files`).
  - `module_name_repetitions`: it flags item names that repeat their module's name; it sits in `restriction`, and the allow keeps it off should Clippy move it into `pedantic`.
- **`opsis-qt` builds against Qt 6.12.0 and KDDockWidgets 2.5.1 from `~/.cache/opsis-dev/qt`**, which `bun run qt:install` fills: Qt through aqtinstall (`uvx`), KDDockWidgets from its release tarball, pinned by URL, sha256 and size. `build.rs` never downloads; it refuses a Qt older than 6.12 and names `qt:install` when either is missing.
  - `OPSIS_QT` names another Qt (CI points it at `jurplel/install-qt-action`'s install) and `OPSIS_KDDW_SOURCE` another KDDockWidgets source.
  - A static Qt (one with `libQt6Core.a`) is linked statically, with its platform plugins imported by name (`STATIC_PLUGINS`) and linked as one group; the shared Qt is linked with an rpath.
- **Release builds link the static Qt `bun run qt:static` builds (`scripts/lib/static-qt.ts`)**: Qt Base and Qt Svg from the pinned source tarballs, without ICU or OpenGL, into `~/.cache/opsis-dev/qt-static/<version>/<folder>`, marked done by an `.opsis-complete` file.
  - On Linux it builds in a plain `debian:trixie` container (`scripts/lib/trixie.ts`, which installs the X11, xcb, Wayland, fontconfig, FreeType, D-Bus and ALSA development packages, then runs as your user), because a static link needs development files a desktop doesn't have; with `CI=true` it builds on the machine itself. macOS and Windows build it in CI.
  - KDDockWidgets builds against Qt's private Widgets headers, so moving Qt means rebuilding and re-checking it.
- **Docked panels show no title, and a lone panel no tab** (`crates/qt/cpp/dock_style.cc` → `DockTitleBar::setVisible`, without `Flag_AlwaysShowTabs`): the docked title bar is zero pixels tall, never hidden, because KDDockWidgets refuses to float a panel through a hidden title bar (`TitleBar::makeWindow`). A floating panel keeps its window's title bar to drag and close it by.
  - The style sheet styles `QTabWidget::pane`, or Fusion draws a second sunken frame inside every dock.
- **Every animation goes through `crates/qt/cpp/motion.cc`** (`FadeIn`, `FadeOut`, `GrowWidth`, `MotionMs`), which skips it when the platform asks for reduced motion (`QAccessibilityHints::motionPreference`, Qt 6.12) and eases like the web editor (`[0.22, 1, 0.36, 1]`, 220 ms).
- **The focus ring shows only after Tab or Backtab** (`FocusRing` and `LookStyle` in `crates/qt/cpp/look_style.cc`): a `QFocusFrame` draws the 3 px `ring` outside the widget, which a style sheet can't, and Fusion's dotted focus rectangle is never drawn.
- **Drop-down lists are `Combo` (`crates/qt/cpp/widgets.h`), never a bare `QComboBox`**: the style sheet draws a combo without asking the style for its arrow, so `Combo` paints its own chevron.
- **Text with `{{variables}}` and pause marks is split and joined in Rust** (`text_tokens`, `join_tokens` → `opsis_compile::vars`), so the native field (`VarSurface`, `crates/qt/cpp/var_text.cc`) draws chips without a second tokenizer.
- **C++ is formatted to Google style with namespace comments off** (`{BasedOnStyle: Google, FixNamespaceComments: false}`), since closing-brace comments are comments. `clang-format` 23.1.5 and `clang-tidy` 23.1.4 run from their PyPI builds through `uvx`, pinned in `scripts/cpp.ts`.
- **`lint:cpp` runs `clang-tidy` with `bugprone-*`, `modernize-*` and `cppcoreguidelines-*` on `crates/qt/cpp` only**, minus the checks that fight Qt: owning memory (Qt's parents own widgets), casts in event handlers, varargs, member initializers (Qt builds child widgets in the constructor body, in layout order), magic numbers and swappable parameters.
- **The Opsis oxlint rules that fix code add their import from `common/`**, so run `lint:fix` until it reports nothing ([AGENTS.md](../AGENTS.md) §2).

## Deliberate limits

- **TypeScript 7 has no parser API**, so the comment check parses TS with `oxc-parser`; the native compiler only type-checks.
- **`lint:zig` builds the wrapper in Debug as a check** into `target/zig-check`; it is not the build Cargo links.
