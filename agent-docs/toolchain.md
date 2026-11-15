# Toolchain

The Rust nightly and how to move it, `bun run setup` and its pins, clang and libc++ for the C++ on Linux, Zig's build of `opsis-vt`'s wrapper, the one static Qt and KDDockWidgets for `opsis-qt`, why the editor uses Qt Widgets, KDDockWidgets and `cxx`, Skia and Filament for the renderer, the C++ checks, the Clippy workspace allows and the repository's own oxlint rules. Paths are relative to the repository root. The everyday commands and the lint exception format are in [AGENTS.md](../AGENTS.md) §3 and §4; Ghostty itself is in [terminal.md](terminal.md), the editor's look and docks in [editor.md](editor.md), the packaging container in [packaging.md](packaging.md).

## Code map

| Concern                              | Where                                                                                                                                                                                                                                                                                                                                 |
| ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Nightly pin                          | `rust-toolchain.toml` (with `rustfmt` and `clippy`)                                                                                                                                                                                                                                                                                   |
| Minimum compiler                     | `rust-version` in `[workspace.package]`, `Cargo.toml`, inherited by every crate                                                                                                                                                                                                                                                       |
| CI toolchain steps                   | every `dtolnay/rust-toolchain` step of `.github/workflows/build.yml` and `.github/workflows/release.yml`                                                                                                                                                                                                                              |
| Workspace lints                      | `[workspace.lints.rust]`, `[workspace.lints.clippy]` in `Cargo.toml`; `[lints] workspace = true` in every crate                                                                                                                                                                                                                       |
| Zig build of the wrapper             | `crates/vt/build.rs` → `zig_target`; `crates/vt/zig/build.zig`, `crates/vt/zig/build.zig.zon`                                                                                                                                                                                                                                         |
| Setup                                | `bun run setup [--no-engine] [--prune]`, `bun run setup pin <item> <version>` → `scripts/setup.ts`; `scripts/lib/setup/` (`manifest.ts`, `store.ts` → `step`, `recipeHash`, `writePaths`; `downloads.ts`, `pin.ts`, `system.ts`); pins `scripts/lib/setup/pins.generated.json`; per-checkout `.setup/paths.json` and `.setup/bin/zig` |
| clang and libc++                     | `.cargo/config.toml` (`CXX_*`, `CXXFLAGS_*`, `CXXSTDLIB_*`, `CXXSTDLIB_STATIC_*`); `scripts/lib/setup/system.ts` → `hasStaticLibcxx`                                                                                                                                                                                                  |
| Opsis oxlint rules                   | `scripts/oxlint-plugin-opsis.ts`; switched on in `scripts/lib/oxlint-base.ts` and the root `oxlint.config.ts`                                                                                                                                                                                                                         |
| Shared oxlint, oxfmt, Vitest configs | `scripts/lib/oxlint-base.ts`, `scripts/lib/oxfmt-base.ts`, `scripts/lib/vitest-base.ts`, `scripts/lib/tool-config.ts`                                                                                                                                                                                                                 |
| Comment check                        | `scripts/comments.ts`                                                                                                                                                                                                                                                                                                                 |
| Qt and KDDockWidgets for `opsis-qt`  | `crates/qt/build.rs` → `QT_MIN_MINOR`, `setup_path`, `pinned_kddw_version`, `build_kddw`, `LINUX_PLUGINS`                                                                                                                                                                                                                             |
| The static Qt                        | `scripts/lib/static-qt.ts` → `buildStaticQt`, `configureArgs`, `buildSteps`; `scripts/lib/trixie.ts` → `runInTrixie`, `LINUX_LIBRARIES`                                                                                                                                                                                               |
| C++ format and lint                  | `scripts/cpp.ts` → `STYLE`, `CHECKS`; `compile_commands.json` written by `crates/qt/build.rs`                                                                                                                                                                                                                                         |
| Rust test runner                     | `scripts/test-rust.ts` (sets `OPSIS_FFMPEG` to `setup`'s FFmpeg)                                                                                                                                                                                                                                                                      |
| Skia                                 | `scripts/lib/skia.ts` → `buildSkia`, `ARGS`, `CHROME_LINUX_DEFINES`, `SKIA_RECIPE`; `crates/skia/build.rs` reads the `consumer.json` in `out/`                                                                                                                                                                                        |
| Filament                             | `crates/filament/build.rs` → `LIBRARIES`, `compile_materials` (`matc` over `materials/`)                                                                                                                                                                                                                                              |

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

### Setup

- **`bun run setup` makes a development machine: every pinned download and every one-time build, once.** It fetches Zig, the Qt and KDDockWidgets sources, Filament, FFmpeg and ffprobe, and the font and shader packages, builds the static Qt and Skia, installs the recording browser into `~/.config/Opsis/engine` (unless `--no-engine`), and writes `.setup/paths.json`.
- **Every pin is in one file, `scripts/lib/setup/pins.generated.json`, written only by `bun run setup pin <item> <version>`** (Zig from ziglang.org's index, Qt and KDDockWidgets by downloading their tarballs, Filament from its GitHub release, FFmpeg from ffmpeg-static's release, an npm package as `name@version` from the registry, Skia by commit). The recording browser's pins are the app's own (`crates/engine/src/downloads.generated.json`, [engine.md](engine.md)).
- **Each item lives in `~/.cache/opsis-dev/<item>/<recipe hash>/`**, the hash covering its pins, its build options and the scripts that build it (`recipeHash`); a `.complete` file written last marks it done (`step`). Setup redoes an item only when its hash changes or the marker is missing, and downloads go once into `~/.cache/opsis-dev/downloads`, checked by sha256 and size.
  - Checkouts with different pins share the cache without rebuilding each other's items; `--prune` deletes every cached version the current pins don't name.
- **`.setup/` (ignored by git) belongs to one checkout**: `paths.json` names each item's folder, read by `crates/qt`, `crates/skia`, `crates/filament`, `opsis-golden` and the scripts, and `bin/zig` links to the pinned Zig, which `crates/vt/build.rs` uses when `ZIG` is unset. `OPSIS_QT`, `OPSIS_KDDW_SOURCE`, `OPSIS_SKIA` and `OPSIS_FILAMENT` override the folders.
- **On Linux setup first checks the programs and system libraries building the app needs** (`checkSystem`: Docker, clang, CMake, make, pkg-config, git, `uvx`, clang's static libc++, and `LINUX_LIBRARIES` through `dpkg-query`), prints the `apt install` line for what is missing and goes on; it never runs `sudo`.

### Zig

- **Zig 0.17 builds the wrapper because Ghostty needs the same major and minor version.** `crates/vt/build.rs` runs `zig build` and refuses any other version.
- **`build.rs` always names the Zig target (`zig_target`)**, because Zig's native target compiles for the building computer's processor.
- **CI runs `bun run setup` in every job that builds Rust, then `zig build --fetch` through `.setup/bin/zig` before Cargo**: Zig unpacks the packages into `crates/vt/zig/zig-pkg` (ignored by git). Moving Zig is `bun run setup pin zig <version>`, `ZIG_MAJOR_MINOR` in `crates/vt/build.rs`, `minimum_zig_version` in `build.zig.zon`, and a Ghostty pin that builds with it.

### clang and libc++

- **On Linux all C++ is compiled by clang against LLVM's libc++, and Rust links libc++ statically**: `.cargo/config.toml` sets `CXX=clang++`, `CXXFLAGS=-stdlib=libc++`, `CXXSTDLIB=c++` and `CXXSTDLIB_STATIC` for the Linux targets, which the `cc` crate and the `cmake` crate (KDDockWidgets) both read. Rust links with the system `cc`, and C stays with the system compiler.
  - The static Qt and Skia are built with trixie's clang and libc++ in the container; libc++ keeps its ABI in its `std::__1` namespace, so their archives link against the building machine's newer libc++.
  - Filament's archives were built with libc++ too, so they link directly.
- **Zig isn't the C++ compiler**: its driver searches the system include folders before every `-isystem` folder, so a bundled library's headers lose to the system's (Skia's libpng then calls the system zlib), and it rejects `--exclude-libs`.

### Qt and KDDockWidgets

- **Why Qt 6.12 LTS Widgets**: five years of fixes; widgets paint with QPainter's raster engine, so no GPU, driver, virtual machine or Xvfb question; native Wayland with text-input-v3 input methods, drag and drop including files from the file manager, and fractional scaling; accessibility through UIA, NSAccessibility and AT-SPI.
- **Why not another toolkit**: Avalonia, the runner-up, runs on XWayland by default with an experimental Wayland backend; Qt Quick with CXX-Qt draws on the GPU and its API breaks twice a year; Slint, GTK 4, egui, Iced, Flutter, Compose, JavaFX, Godot and others each lack a stable release, more than one active maintainer, drawing without a GPU, native Wayland, docking or tests.
- **Why KDDockWidgets and `cxx`**: KDDockWidgets nests, tabs and floats native windows that dock back, built for Widgets first; `cxx` is 1.0 and stable, with `cxx-build` compiling the C++ from `build.rs` and `qt-build-utils` finding Qt and running `moc` and `rcc`.
- **`opsis-qt` builds against the one static Qt and the KDDockWidgets source `setup` names** (`setup_path`); `build.rs` never downloads, and refuses any Qt but Qt 6 from `QT_MIN_MINOR` up. KDDockWidgets' version for the About page comes from the pins (`pinned_kddw_version`).
  - Qt is linked statically, with the Qt plugins its target needs imported by name (`Toolchain::plugins`: `LINUX_PLUGINS`, `MACOS_PLUGINS`, `WINDOWS_PLUGINS`), the offscreen, minimal and Wayland ones included for the tests, the JPEG, GIF, ICO and SVG image formats and the SVG icon engine, which the views read pictures and icons through, and KDDockWidgets built static too; `build.rs` stops when a listed plugin's archive is missing.
  - The link line follows the target's linker (`Toolchain`): on Linux the archives come as one `--start-group` group, while Apple's linker and MSVC's resolve archives in any order and get them plain. System libraries come from Qt's `.prl` files, so a Linux machine needs the development packages of `LINUX_LIBRARIES`; a `-framework` name stays paired with its flag, and on MSVC `-lname` becomes `name.lib` and `-Ldir` `/LIBPATH:dir`.
  - No style plugin is linked: the editor draws with Fusion (`app.cc`), which is built into Qt Widgets.
  - **On Linux the editor exports nothing from its static archives** (`-Wl,--exclude-libs,ALL`, `crates/ui/build.rs`): the linker otherwise exports any symbol a shared library asks for, so the FreeType inside the static Skia would stand in for the system FreeType fontconfig loads, and the editor's fonts would come out wrong.
  - **Skia's own libraries are private to it**: on Linux the Skia step renames every symbol its libjpeg, libpng, libwebp and FreeType define, in all of its archives, to `opsis_skia_…` (`makePrivate`, `PRIVATE_LIBRARIES`, `scripts/lib/skia.ts`). The editor links Qt and Skia into one program, where the linker would otherwise give Qt's JPEG plugin Skia's libjpeg of another version (it aborts) and Qt's text the FreeType inside Skia instead of the system's.
  - On MSVC every C++ file is compiled with `/Zc:__cplusplus /permissive- /utf-8` and Qt's Windows defines (`Toolchain::qt_flags`), which Qt 6's headers need.
  - KDDockWidgets builds against Qt's private Widgets headers, so moving Qt means rebuilding and re-checking it.
  - Moving Qt is `bun run setup pin qt <version>` (and `QT_MIN_MINOR` when the minor moves); KDDockWidgets' is `bun run setup pin kddw <version>` and the About page check in `crates/qt/cpp/e2e_settings.cc`. After a move, run `bun run qt:notices` again and update the versions written in prose: AGENTS.md (§3 and the `opsis-qt` row), README.md, the reasons above and the Qt and KDDockWidgets headings of `THIRD_PARTY_NOTICES.md`.
- **The one static Qt is built by `setup` (`buildStaticQt`)**: the pinned `qtbase` and `qtsvg` tarballs, configured by `configureArgs`, into the item's `prefix`. Development and release builds both link it; why a release links Qt statically is in [packaging.md](packaging.md).
  - The build is a list of steps (`buildSteps`: unpack with `cmake -E tar`, Qt's `configure`, build, install, then the same for Qt SVG), configured per platform (`configureArgs`): Linux asks for xcb, fontconfig and the system FreeType and builds with clang and libc++ under the `linux-clang-libc++` mkspec, Qt's programs (`moc`, `rcc`) linking libc++ statically (`-static-libstdc++`), macOS adds `-no-framework` and the 14.4 deployment target, and no platform asks for a static C runtime, since Rust links the shared one.
  - On Linux the steps run inside the `debian:trixie` packaging container (`runInTrixie`, so Docker is needed, `shellScript`), so the Qt it makes links on trixie's glibc and on anything newer. On macOS and Windows they run natively (`buildsNatively`, `runNatively`), each step's output in a log beside it whose last lines are printed when it fails.
  - On Windows Qt's own scripts are batch files, started through `cmd.exe`, and the build needs Visual Studio's x64 C++ tools: the script loads their environment through `vswhere` and `vcvars64.bat` (`msvcEnv`) unless it already runs in one, and builds under `$RUNNER_TEMP/qtb` in CI to keep paths short.

### C++

- **C++ is formatted to Google style with namespace comments off (`STYLE`)**, since closing-brace comments are comments. `clang-format` and `clang-tidy` run from their PyPI builds through `uvx`, pinned in `scripts/cpp.ts`.
- **`lint:cpp` runs `clang-tidy` with the `bugprone`, `modernize` and `cppcoreguidelines` checks on the `.cc` files of `crates/qt/cpp` only, minus the ones in `CHECKS` that fight Qt or the code's style**: owning memory (Qt's parents own widgets) and member initializers (Qt builds child widgets in the constructor body, in layout order) among them.
  - It first runs `cargo check -p opsis-qt`, which writes the `compile_commands.json` it reads, so it needs `bun run setup` first.

### Skia and Filament

- **Skia is built by `setup` from one pinned commit (`chrome/m157`) with GN, CPU only** (`buildSkia`): its own FreeType, libpng, libjpeg-turbo, libwebp and zlib, Skottie and the primitive shaper; no fontconfig, HarfBuzz, ICU, PartitionAlloc or Perfetto, because PartitionAlloc as `malloc` would replace the whole app's allocator.
  - On Linux it builds inside the `debian:trixie` container with clang and `-stdlib=libc++` (`CXXFLAGS`), like Qt, so it uses the same libc++.
  - It takes Chrome's Linux defines (`CHROME_LINUX_DEFINES`: text gamma and contrast, and the legacy blur, rounded-rect and sampling flags), so its blur, gradient and edge maths stay closest to the browser's.
  - The build writes `consumer.json` (defines, include folders, libraries) into `out/`; `crates/skia/build.rs` compiles the wrapper's C++ and links with exactly that.
- **Filament comes from Google's release archives, pinned by sha256 and size per platform**, unpacked by `setup`. `crates/filament/build.rs` compiles `crates/filament/cpp` with the same C++ compiler as everything else (`-fno-rtti`, as Filament is built), links Filament's archives (`LIBRARIES`) directly, and compiles `materials/*.mat` with the archive's `matc`.
  - Rust reaches the wrapper through the C functions in `opsis_filament.h`.
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
