# Toolchain

The Rust nightly and how to move it, the Zig build of `opsis-vt`'s wrapper, Qt and KDDockWidgets for `opsis-qt` and the release build's static Qt, why the editor uses Qt Widgets, KDDockWidgets and `cxx`, Skia and Filament for the renderer, the C++ checks, the Clippy workspace allows and the repository's own oxlint rules. Paths are relative to the repository root. The everyday commands and the lint exception format are in [AGENTS.md](../AGENTS.md) §3 and §4; Ghostty itself is in [terminal.md](terminal.md), the editor's look and docks in [editor.md](editor.md), the packaging container in [packaging.md](packaging.md).

## Code map

| Concern                                | Where                                                                                                                                                                                                                                                                                                                            |
| -------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Nightly pin                            | `rust-toolchain.toml` (with `rustfmt` and `clippy`)                                                                                                                                                                                                                                                                              |
| Minimum compiler                       | `rust-version` in `[workspace.package]`, `Cargo.toml`, inherited by every crate                                                                                                                                                                                                                                                  |
| CI toolchain steps                     | every `dtolnay/rust-toolchain` step of `.github/workflows/build.yml` and `.github/workflows/release.yml`                                                                                                                                                                                                                         |
| Workspace lints                        | `[workspace.lints.rust]`, `[workspace.lints.clippy]` in `Cargo.toml`; `[lints] workspace = true` in every crate                                                                                                                                                                                                                  |
| Zig build of the wrapper               | `crates/vt/build.rs` → `zig_target`; `crates/vt/zig/build.zig`, `crates/vt/zig/build.zig.zon`                                                                                                                                                                                                                                    |
| Zig in CI                              | `scripts/zig-install.ts` (`bun run zig:install`) → `ZIG_VERSION`, `ARCHIVES`                                                                                                                                                                                                                                                     |
| Opsis oxlint rules                     | `scripts/oxlint-plugin-opsis.ts`; switched on in `scripts/lib/oxlint-base.ts` and the root `oxlint.config.ts`                                                                                                                                                                                                                    |
| Shared oxlint, oxfmt, Vitest configs   | `scripts/lib/oxlint-base.ts`, `scripts/lib/oxfmt-base.ts`, `scripts/lib/vitest-base.ts`, `scripts/lib/tool-config.ts`                                                                                                                                                                                                            |
| Comment check                          | `scripts/comments.ts`                                                                                                                                                                                                                                                                                                            |
| Qt and KDDockWidgets for `opsis-qt`    | `scripts/qt-install.ts` (`bun run qt:install`); `crates/qt/build.rs` → `QT_VERSION`, `QT_MIN_MINOR`, `KDDW_VERSION`, `STATIC_PLUGINS`                                                                                                                                                                                            |
| The release build's static Qt          | `bun run qt:static` → `scripts/qt-static.ts`; `scripts/lib/static-qt.ts` → `ensureStaticQt`, `SOURCES`, `configureArgs`, `buildSteps`; `scripts/lib/trixie.ts` → `runInTrixie`                                                                                                                                                   |
| Qt in CI                               | `jurplel/install-qt-action` in `.github/workflows/build.yml`; `bun run qt:static` in `.github/workflows/release.yml`                                                                                                                                                                                                             |
| C++ format and lint                    | `scripts/cpp.ts` → `STYLE`, `CHECKS`; `compile_commands.json` written by `crates/qt/build.rs`                                                                                                                                                                                                                                    |
| Rust test runner (stages FFmpeg first) | `scripts/test-rust.ts`                                                                                                                                                                                                                                                                                                           |
| Skia                                   | `bun run skia:build [--docker]` → `scripts/skia-build.ts` (`SKIA_COMMIT`, `ARGS`, `CHROME_LINUX_DEFINES`), into `~/.cache/opsis-dev/skia/<commit>/out/<platform>`; `crates/skia/build.rs` reads its `consumer.json`                                                                                                              |
| Filament                               | `bun scripts/filament-pin.ts` → `crates/filament/downloads.generated.json`; `bun run filament:build [--docker]` → `scripts/filament-build.ts` (`LIBRARIES`, `PERSONALITY_REF`), into `~/.cache/opsis-dev/filament/<version>/<platform>/build`; `crates/filament/build.rs` (`matc` over `materials/`, the archive's `build.json`) |

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

- **Why Qt 6.12 LTS Widgets**: five years of fixes; widgets paint with QPainter's raster engine, so no GPU, driver, virtual machine or Xvfb question; native Wayland with text-input-v3 input methods, drag and drop including files from the file manager, and fractional scaling; accessibility through UIA, NSAccessibility and AT-SPI.
- **Why not another toolkit**: Avalonia, the runner-up, runs on XWayland by default with an experimental Wayland backend; Qt Quick with CXX-Qt draws on the GPU and its API breaks twice a year; Slint, GTK 4, egui, Iced, Flutter, Compose, JavaFX, Godot and others each lack a stable release, more than one active maintainer, drawing without a GPU, native Wayland, docking or tests.
- **Why KDDockWidgets and `cxx`**: KDDockWidgets nests, tabs and floats native windows that dock back, built for Widgets first; `cxx` is 1.0 and stable, with `cxx-build` compiling the C++ from `build.rs` and `qt-build-utils` finding Qt and running `moc` and `rcc`.
- **`opsis-qt` builds against the Qt and KDDockWidgets that `QT_VERSION` and `KDDW_VERSION` name, from `~/.cache/opsis-dev/qt`**, which `bun run qt:install` fills: Qt through aqtinstall (`uvx`), KDDockWidgets from its release tarball, pinned by URL, sha256 and size. `build.rs` never downloads; it refuses any Qt but Qt 6 from `QT_MIN_MINOR` up and names `qt:install` when either is missing.
  - `OPSIS_QT` names another Qt and `OPSIS_KDDW_SOURCE` another KDDockWidgets source.
  - A static Qt (one with `libQt6Core.a`, or on MSVC `Qt6Core.lib` with no `Qt6Core.dll`) is linked statically, with the Qt plugins its target needs imported by name (`Toolchain::plugins`: `LINUX_PLUGINS`, `MACOS_PLUGINS`, `WINDOWS_PLUGINS`) and KDDockWidgets built static too; `build.rs` stops when a listed plugin's archive is missing. The shared Qt is linked with an rpath (none on Windows).
  - The link line follows the target's linker (`Toolchain`): GNU ld gets the archives as one `--start-group` group, while Apple's linker and MSVC's resolve archives in any order and get them plain. System libraries come from Qt's `.prl` files; a `-framework` name stays paired with its flag, and on MSVC `-lname` becomes `name.lib` and `-Ldir` `/LIBPATH:dir`.
  - No style plugin is linked: the editor draws with Fusion (`app.cc`), which is built into Qt Widgets.
  - **On Linux the editor exports nothing from its static archives** (`-Wl,--exclude-libs,ALL`, `crates/ui/build.rs`): GNU ld otherwise exports any symbol a shared library asks for, so the FreeType inside the static Skia (`opsis-skia`, pulled in through `opsis-render`) would stand in for the system FreeType the shared Qt and fontconfig load, and the editor's fonts would come out wrong.
  - On MSVC every C++ file is compiled with `/Zc:__cplusplus /permissive- /utf-8` and Qt's Windows defines (`Toolchain::qt_flags`), which Qt 6's headers need.
  - KDDockWidgets builds against Qt's private Widgets headers, so moving Qt means rebuilding and re-checking it.
  - The Qt version is written in `crates/qt/build.rs` (`QT_VERSION`, and `QT_MIN_MINOR` when the minor moves), `scripts/qt-install.ts` (beside the aqtinstall pin, `AQT`), `scripts/lib/static-qt.ts` (with its pinned source tarballs) and both `install-qt-action` steps of `build.yml`, and moves in all of them at once; KDDockWidgets' in `build.rs`, `qt-install.ts` and the About page check in `crates/qt/cpp/e2e_settings.cc`.
  - After a move, run `bun run qt:notices` again, and update the versions written in prose: `build.rs`'s messages, AGENTS.md (§3 and the `opsis-qt` row), README.md, the reasons above and the Qt and KDDockWidgets headings of `THIRD_PARTY_NOTICES.md`.
- **CI's Qt depends on the workflow**: `build.yml` points `OPSIS_QT` at `jurplel/install-qt-action`'s shared Qt; `release.yml` builds the static Qt with `bun run qt:static`, caches it, and sets `OPSIS_QT` from the folder it reports. Both then run `bun run qt:install`, which uses the Qt in `OPSIS_QT` when that folder has a `lib` folder (and downloads one otherwise) and fetches KDDockWidgets.
- **Release builds link the static Qt `bun run qt:static` builds (`ensureStaticQt`)**: the modules in `SOURCES` from the pinned source tarballs, configured by `CONFIGURE`, into `~/.cache/opsis-dev/qt-static/<version>/<folder>`, marked done by an `.opsis-complete` file. Why a release links it statically is in [packaging.md](packaging.md).
  - The build is a list of steps (`buildSteps`: unpack with `cmake -E tar`, Qt's `configure`, build, install, then the same for Qt SVG), configured per platform (`configureArgs`): only Linux asks for xcb, fontconfig and the system FreeType, macOS adds `-no-framework` and the 14.4 deployment target, and no platform asks for a static C runtime, since Rust links the shared one.
  - On Linux the steps run inside the `debian:trixie` packaging container (`runInTrixie`, so Docker is needed, `shellScript`), because a static link needs X11 and xcb development files a desktop doesn't have. On macOS and Windows they run natively (`buildsNatively`, `runNatively`), each step's output in a log beside it whose last lines are printed when it fails.
  - On Windows Qt's own scripts are batch files, started through `cmd.exe`, and the build needs Visual Studio's x64 C++ tools: the script loads their environment through `vswhere` and `vcvars64.bat` (`msvcEnv`) unless it already runs in one, and builds under `$RUNNER_TEMP/qtb` in CI to keep paths short.

### C++

- **C++ is formatted to Google style with namespace comments off (`STYLE`)**, since closing-brace comments are comments. `clang-format` and `clang-tidy` run from their PyPI builds through `uvx`, pinned in `scripts/cpp.ts`.
- **`lint:cpp` runs `clang-tidy` with the `bugprone`, `modernize` and `cppcoreguidelines` checks on the `.cc` files of `crates/qt/cpp` only, minus the ones in `CHECKS` that fight Qt or the code's style**: owning memory (Qt's parents own widgets) and member initializers (Qt builds child widgets in the constructor body, in layout order) among them.
  - It first runs `cargo check -p opsis-qt`, which writes the `compile_commands.json` it reads, so it needs Qt installed.

### Skia and Filament

- **Skia is built from one pinned commit (`SKIA_COMMIT`, branch `chrome/m157`) with GN, CPU only**: its own FreeType, libpng, libjpeg-turbo, libwebp and zlib, Skottie and the primitive shaper; no fontconfig, HarfBuzz, ICU, PartitionAlloc or Perfetto, because PartitionAlloc as `malloc` would replace the whole app's allocator. `--docker` builds it in an `ubuntu:26.04` container.
  - It takes Chrome's Linux defines (`CHROME_LINUX_DEFINES`: text gamma and contrast, and the legacy blur, rounded-rect and sampling flags), so its blur, gradient and edge maths stay closest to the browser's.
  - The build writes `consumer.json` (defines, include folders, libraries); `crates/skia/build.rs` compiles the wrapper's C++ and links with exactly that. `OPSIS_SKIA` moves the folder.
- **Filament comes from Google's release archives, pinned by sha256 and size per platform** (`filament-pin.ts`), unpacked into `~/.cache/opsis-dev/filament/<version>/<platform>`; `OPSIS_FILAMENT` moves the folder.
- **Filament's Linux build is clang and libc++, the rest of the app's C++ GCC and libstdc++, so `crates/filament/cpp` is sealed**: `filament:build` compiles it with clang and libc++, links it with Filament's archives (`LIBRARIES`) and the static `libc++.a` and `libc++abi.a` (`ld -r`), keeps only the `opsis_*` symbols global (`objcopy`) and writes `libopsis_filament.a` and `build.json`.
  - No C++ type crosses that boundary; Rust reaches it through the C functions in `opsis_filament.h`.
  - `objcopy` also renames `DW.ref.__gxx_personality_v0` (`PERSONALITY_REF`): it sits in a COMDAT group, and when Skia's libstdc++ objects bring a group of the same name the linker keeps theirs and the sealed object's exception tables point into a discarded section.
- **`crates/filament/build.rs` refuses a sealed archive built from other sources or another Filament version** (`build.json`'s hash of `crates/filament/cpp` and its version), so a change there needs `bun run filament:build` again before Cargo builds; it compiles `materials/*.mat` with the archive's `matc`.
- **Only `opsis-renderer` links Filament**, so the app itself carries no Filament or Vulkan loader ([film.md](film.md)).

### TypeScript

- **TypeScript is only `scripts/`** (and the skills' scripts): no package or app members, one root `tsconfig.json`, `oxlint.config.ts` and `vitest.config.ts`.
- **Opsis's own oxlint rules live in one plugin (`scripts/oxlint-plugin-opsis.ts`)**, switched on in `scripts/lib/oxlint-base.ts`:
  - `use-find-by-id`: `list.find((x) => x.id === id)` becomes `findById` (`scripts/lib/common/lists.ts`);
  - `use-type-guards`: `(x) => typeof x === "string"` (or `number`, `boolean`) becomes `isString`, `isNumber` or `isBoolean` (`scripts/lib/common/guards.ts`);
  - `use-parse-json-with`: a schema's `safeParse` that takes `JSON.parse(…)` or an `ok ? … : …` asks for `parseJsonWith` (`scripts/lib/common/guards.ts`), with no fix;
  - `use-function-declarations`: the AGENTS.md §2 rule on function declarations;
  - in `scripts/**` only (the root `oxlint.config.ts`, `scripts/lib/tool-config.ts` aside): `use-root`, which asks for `ROOT` (`scripts/lib/tool-config.ts`) where a script calls `resolve(import.meta.dirname, …)`, with no fix; other uses of `import.meta.dirname` pass.
  - `scripts/lib/common/lists.ts` and `guards.ts` are left out of the rules that point to them.
- **Only the rules whose fix calls `fixWith` (`use-find-by-id`, `use-type-guards`) add an import, a relative one to `scripts/lib/common/`**; in a file that already declares that name, they offer no fix. Run `lint:fix` until it reports nothing ([AGENTS.md](../AGENTS.md) §2).
- **`knip` ignores the exports of `scripts/lib/common` (the lint fixes insert them) and the types of `scripts/lib/generated`** (`knip.config.ts`).

## Deliberate limits

- **`lint:zig` builds the wrapper in Debug as a check** into `target/zig-check`; it is not the build Cargo links.
