# Native editor

How Opsis ships its native editor: Qt 6.12 LTS Widgets with a C++ view layer that Rust drives through `cxx`, drawn the same on Linux, Windows and macOS, in the same process as the document, the compile and the run.

Status: everything up to the cut-over is built and the web editor, the Tauri app and the WebdriverIO suite are gone; their rules are in [ui.md](../agent-docs/ui.md), [editor.md](../agent-docs/editor.md), [document.md](../agent-docs/document.md), [e2e.md](../agent-docs/e2e.md), [toolchain.md](../agent-docs/toolchain.md) and [packaging.md](../agent-docs/packaging.md). **Packaging** remains, and starts only on a go-ahead.

## Conventions

These restate rules from `AGENTS.md` and `agent-docs/`; when they disagree with this plan, they win. When a part ships, its rules move into the domain docs.

- **Rust owns the document, the compile, the run and every file.** C++ draws and forwards input; it decides nothing a Rust function can decide. Pure logic the editor has in TypeScript today (edge lanes, snapping, ducking, envelopes, layout, schema reading) moves to Rust with property tests, not to C++.
- **No comments in C++ either**: not in `.cpp`, `.h` or `.ui` files. `scripts/comments.ts` gains a C++ path.
- **Nothing uses the internet unless the user asks**, as today.
- **User-facing copy is unchanged**: no library, tool or format names, sentence case, durations in seconds, browser nodes are "actions".
- **Every Qt call happens on Qt's GUI thread.** Rust work runs on its own threads and runtime and reaches the GUI thread only through the bridge's queued events.
- **C++ owns every `QObject` through Qt's parent tree; Rust never holds or deletes one.** C++ holds the Rust session through a `rust::Box`.
- **Checks when a part is done:** `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`, plus the part's own Qt tests, and a check in the app under Xvfb.

## Decided

- **Qt 6.12 LTS, Qt Widgets.** Qt 6.12 is a long-term-support release with five years of fixes. Qt Widgets had 387 commits from 77 authors in `qtbase/src/widgets` over 12 months, and qtbase carries 612 widget test files.
  - **Drawing on the CPU everywhere.** Widgets paint with QPainter's raster engine into the platform's backing store (shared memory on Wayland, `qwaylandshmbackingstore.cpp`), so no GPU, driver, virtual machine or Xvfb question.
  - **Native Wayland.** qtbase's Wayland client has text-input-v3 input methods (`qwaylandtextinputv3.cpp`), drag and drop including files from the file manager (`qwaylanddnd.cpp`, `qwaylanddatadevice.cpp`) and fractional scaling (`qwaylandfractionalscale.cpp`).
  - **Accessibility** through UIA, NSAccessibility and AT-SPI (`src/gui/accessible/linux`).
- **`QGraphicsView` for the graph editor**, the scene that Natron's `NodeGraph` and Olive's `NodeView` are built on; **the timeline paints itself** on a `QAbstractScrollArea`, because it zooms on time only (a scene transform would stretch its labels), its ruler and track labels stay put, and painting from the Rust view keeps no items in step.
- **KDDockWidgets 2.5.1 (KDAB) for docking**: nesting, tabs, and floating native windows that dock back, built for Widgets first.
- **`cxx` 1.0 for the bridge** (1.0.205, 85 million downloads), with `cxx-build` compiling the C++ side from `build.rs`, and `qt-build-utils` (KDAB) finding Qt and running `moc` and `rcc`.
- **Why not the alternatives** is under [Alternatives](#alternatives).

## Packaging

### What it changes

`bun run make` builds installers for the native app, with Qt and KDDockWidgets bundled and FFmpeg a program beside the app; today it stages and builds the release binary only.

### Fit with Opsis

**In place.** `scripts/stage.ts`; the release binary (`target/release/opsis`) that finds FFmpeg and its resources beside itself, or in `Resources` on macOS (`crates/ui/src/folders.rs`); `THIRD_PARTY_NOTICES.md`; `deny.toml`.

**Missing.** A bundler for each format; Qt deployment; the CI builds on Windows and macOS, signing and notarization, which left with the Tauri bundler.

### Work

1. Resources: each format puts the licences and notices, and the film page and terminal page while they exist ([native-renderer.md](native-renderer.md) removes both at its cut-over), where the app looks: beside the program, or in the macOS bundle's `Resources`.
2. Linux: `linuxdeploy` with its Qt plugin for the AppImage; `cargo-deb` and `cargo-generate-rpm` for deb and rpm, with Qt bundled or depended on (decided below). The deb no longer needs `gstreamer1.0-libav` or WebKitGTK.
3. Windows: `windeployqt`, then NSIS (per-user install as today).
4. macOS: `macdeployqt`, `codesign` on every Mach-O, notarization in CI as today.
5. Licences: the Qt libraries ship as shared libraries so the LGPL relinking terms hold, with the third-party attributions of the Qt build they come from; Qt's LGPL-3.0 text and KDDockWidgets' notices are already in `THIRD_PARTY_NOTICES.md`.
6. CI: build every format on its platform again, check the licences each ships and every Mach-O's signature, as the Tauri bundler's job did.
7. Profiling: `scripts/profile.ts` and `scripts/profile-inside.ts` install the new deb and start its program; they still name the Tauri app's `/usr/bin/opsis-desktop` and its session.

### Decisions

- **Which formats ship.** None is required: a tarball per platform (the program plus its resources folder) runs as it is. AppImage adds one file that runs on most distros; deb and rpm add a distro install with its dependencies, menu entry and updates; the macOS app and the Windows installer are what those systems expect from a signed program. Decided at the start of this part.
- **deb and rpm: bundle Qt or depend on the distro's Qt.** Distro Qt versions vary and Opsis needs 6.12; recommended: bundle, like the AppImage.

### Sources

- linuxdeploy Qt plugin: https://github.com/linuxdeploy/linuxdeploy-plugin-qt (checked 2026-11-05)
- Qt deployment: https://doc.qt.io/qt-6/deployment.html (checked 2026-11-05)
- Qt licensing (LGPL-3.0): https://doc.qt.io/qt-6/licensing.html (checked 2026-11-05)

## Alternatives

Checked against the same gates: a stable release with a compatibility policy, more than one active maintainer, building blocks (input methods, menus, dialogs, popovers, virtualised lists, focus, drag and drop including files from the file manager), drawing without a GPU, and tests.

**Discarded, on stable releases.**

- **Avalonia 12.1.5 (.NET), the runner-up**: better styling (XAML styles, transitions, shadows) and MIT throughout, but it runs on Wayland through XWayland by default; its native Wayland backend (opt-in since 12.1.0) and Linux client-side decorations are experimental; its node-graph libraries are barely active (17 and 32 commits in 12 months); its Dock library is mainly one author. https://github.com/AvaloniaUI/Avalonia, https://docs.avaloniaui.net/docs/platform-specific-guides/linux, https://github.com/wieslawsoltes/Dock
- **Qt Quick with CXX-Qt 0.10**: draws through OpenGL, Direct3D or Metal and falls back to software only when they fail; CXX-Qt has broken its API about twice a year; the graph would be assembled from handlers and shapes. https://github.com/KDAB/cxx-qt, https://doc.qt.io/qt-6/qtquick-visualcanvas-adaptations-software.html
- **Godot 4.7.2 as an application UI**: native Wayland, built-in `GraphEdit`, screen readers; but no docking with pop-out windows, no software renderer (Xvfb needs Mesa GL or lavapipe), a main-thread-only engine API, a game-style frame loop, and a pre-1.0 Rust binding (`godot` 0.5.5). https://github.com/godotengine/godot, https://github.com/godot-rust/gdext
- **Swing with FlatLaf 3.8 on JetBrains Runtime**: native Wayland only through JetBrains Runtime's WLToolkit; FlatLaf's Linux window move and resize are X11-only; no Linux screen reader in the JDK; FlatLaf is one maintainer; no node-graph library. https://github.com/JFormDesigner/FlatLaf, https://github.com/JetBrains/JetBrainsRuntime
- **JavaFX 27.0.1**: XWayland only; no Linux screen reader; its headless platform is a prototype; the maintained graph library (eckig/graph-editor) is EPL-1.0, GPL-incompatible. https://github.com/openjdk/jfx, https://github.com/eckig/graph-editor
- **Compose Multiplatform Desktop 1.12.2**: XWayland only; no Linux screen reader; drag and drop and the test API are experimental; the docking library is 0.4.0. https://github.com/JetBrains/compose-multiplatform
- **Uno Platform 6.7.135**: XWayland only; no custom title bar or native menu on macOS; Linux screen reader planned; headless test host unpublished. https://github.com/unoplatform/uno
- **Slint 1.18.1**: no files dropped from the file manager in its desktop backend; drag and drop within one window only; no drawing API ([#432](https://github.com/slint-ui/slint/issues/432)); no docking ([#1723](https://github.com/slint-ui/slint/issues/1723)); its CPU renderer ignores `scale()`. https://github.com/slint-ui/slint
- **GTK 4.22 with gtk4-rs 0.11.5**: 67 open macOS and 131 open Windows issues; no in-process input simulation; the GNOME look on every OS. https://gitlab.gnome.org/GNOME/gtk, https://github.com/gtk-rs/gtk4-rs
- **egui 0.37.0**: breaking releases every two to three months; no CPU renderer; heavy CPU on WARP and llvmpipe ([#8695](https://github.com/emilk/egui/issues/8695)); winit 0.30 delivers no files dropped on Wayland. https://github.com/emilk/egui
- **Iced 0.14.0**: its README calls it experimental; one main author; no accessibility ([#552](https://github.com/iced-rs/iced/issues/552)). https://github.com/iced-rs/iced
- **Freya 0.4.3**: one author wrote 922 of 987 commits in 12 months; rewritten in July 2026. https://github.com/marc2332/freya
- **GPUI with gpui-kit 0.9**: pre-1.0 weekly snapshots; no CPU renderer; a transparent window on llvmpipe ([zed #66309](https://github.com/zed-industries/zed/issues/66309)). https://github.com/zed-industries/zed/tree/main/crates/gpui, https://github.com/longbridge/gpui-component
- **Flutter 3.47.9 with flutter_rust_bridge 2.13.0**: multiple windows are internal and disabled on stable; file drops, window chrome and extra windows only through 0.x plugins. https://github.com/flutter/flutter, https://github.com/fzyzcjy/flutter_rust_bridge
- **Dear ImGui 1.93.0**: docking only on a branch; multiple viewports disabled on Wayland; no accessibility. https://github.com/ocornut/imgui
- **JUCE 9.0.5**: on Linux no input methods, video, accessibility or Wayland; AGPL-3.0. https://github.com/juce-framework/JUCE
- **wxWidgets 3.2.12, Lazarus/LCL, Eto.Forms, SWT, AppKit with WinUI 3 and GTK**: native widgets per OS, so not drawn the same everywhere. https://github.com/wxWidgets/wxWidgets
- **CEF used directly**: still a web stack and a second Chromium; no embedding in a Wayland host window ([cef #2804](https://github.com/chromiumembedded/cef/issues/2804)). https://github.com/chromiumembedded/cef
- **Floem 0.2.0, Vizia 0.4.0, Kas 0.17.0, Makepad, wxDragon 0.9.24, qmetaobject-rs 0.2.10**: no release in a year, a single main maintainer, or no compatibility policy. https://github.com/lapce/floem, https://github.com/vizia/vizia, https://github.com/kas-gui/kas, https://github.com/makepad/makepad, https://github.com/AllenDang/wxDragon, https://github.com/woboq/qmetaobject-rs
- **Sciter, Ultralight, NoesisGUI**: proprietary or single-developer engines.
- **RmlUi 6.3, FLTK 1.4.5, Tk 9.1.0**: no OS dialogs, menus or multiple windows (RmlUi); no accessibility (FLTK); no Wayland (Tk). https://github.com/mikke89/RmlUi, https://github.com/fltk/fltk, https://github.com/tcltk/tk
- **.NET MAUI, React Native for Windows and macOS**: no Linux.

**Pre-release, experimental or unreleased.**

- `tauri-runtime-cef` 3.0.0-alpha.6 (one Chromium on every OS for the current editor). https://github.com/tauri-apps/tauri/blob/v3/crates/tauri-runtime-cef/CHANGELOG.md
- Avalonia's native Wayland backend (`Avalonia.Wayland`, since 12.1.0); Uno's native Wayland host (PR #25503, open). https://github.com/unoplatform/uno/pull/25503
- Slint 1.19 (unreleased master: drag and drop across windows on winit 0.31.0-beta.4). https://github.com/rust-windowing/winit/pull/4571
- Freya 0.5.0-rc.13; Iced 0.15.0-dev; Compose Multiplatform 1.13.0-beta02; Godot 4.8-beta2.
- `gpui-pre` 0.3.11 snapshots, GPUI Fast 0.1.13, gpui-ce 0.3.0. https://github.com/longbridge/gpui-fast, https://github.com/gpui-ce/gpui-ce
- Xilem and Masonry 0.4.0, Blitz 0.3.0-beta.4, Dioxus Native 0.8.0-alpha.3, Ribir 0.4.0-alpha.65, libcosmic (git only). https://github.com/linebender/xilem, https://github.com/DioxusLabs/blitz, https://github.com/RibirX/Ribir, https://github.com/pop-os/libcosmic
