# Native editor bridge

How the editor's Qt Widgets views and the Rust session talk: the `cxx` bridge, which side owns what, how events and frames cross, and the rules for C++ views. Paths are relative to the repository root. The editor's windows and panels are in [editor.md](editor.md), the document they edit in [document.md](document.md), the build of Qt and KDDockWidgets in [toolchain.md](toolchain.md), the scenarios that drive the views in [e2e.md](e2e.md).

## Code map

| Concern                          | Where                                                                                                                                                                                         |
| -------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| The app binary and its host      | `crates/ui/src/main.rs` → `start`; `crates/ui/src/ui.rs` (`Ui`, the `opsis_qt::Host` implementation); `crates/ui/src/platform.rs` → `QtPlatform`                                              |
| View builders                    | `crates/ui/src/`, one module per view, such as `graph.rs`, `inspector.rs`, `timeline.rs`, `mixer.rs`, `run_panel.rs`, `replay.rs`, `live.rs`, `streams.rs`, `terminal_window.rs`, `player.rs` |
| The bridge                       | `crates/qt/src/bridge.rs` (`cxx::bridge`: the view structs, `extern "Rust"` session calls, `extern "C++"` functions), `crates/qt/src/host.rs` (`Host`, `Session`, `run`)                      |
| The C++ side of the bridge       | `crates/qt/cpp/bridge_ffi.h`; `crates/qt/cpp/platform.cc` (`OnGuiThread`, `request_dialog`, the window, path and notification calls)                                                          |
| Building the C++                 | `crates/qt/build.rs` → `MOC_HEADERS`, `SOURCES`; `crates/qt/resources/resources.qrc`                                                                                                          |
| Frames                           | `crates/qt/src/frame.rs` (`FrameRef`)                                                                                                                                                         |
| Event delivery on the GUI thread | `crates/qt/cpp/event_hub.cc` (`EventHub`, `post_event`, `post_edited`, `post_replay`), `crates/qt/cpp/wake_hub.cc` (`WakeHub`, `wake`)                                                        |
| One undo step for a gesture      | `crates/qt/cpp/form_hold.cc` (`FormHold`)                                                                                                                                                     |
| The application object           | `crates/qt/cpp/app.cc` (`App`: `Run`, tokens, icons, fonts)                                                                                                                                   |
| Look                             | `crates/ui/src/look.rs` (`TOKENS`, `style_sheet`), `crates/ui/resources/style.qss`, `crates/ui/glyphs.generated.json` (`bun run glyphs:bundle`)                                               |
| C++ format and lint              | `scripts/cpp.ts` (pinned clang-format and clang-tidy, the check list)                                                                                                                         |

## Rules

- **Rust owns every piece of state and decides everything a Rust function can; C++ draws and forwards input.** A view is a plain struct Rust builds when C++ asks (`GraphView`, `InspectorView`, `TimelineView`, `MixerView`…), and C++ keeps only the last view it drew, never project data of its own.
  - Pure logic (edge lanes, snapping, ducking, envelopes, layout, schema reading) lives in Rust with property tests, never in C++.
- **Every Qt call happens on the GUI thread, and C++ owns every `QObject` through Qt's parent tree**: Rust never holds or deletes one, and C++ holds the Rust host as a `rust::Box<Session>` (`App::Run`).
- **C++ calls Rust through the `Session` methods of the bridge; Rust reaches C++ only through the `extern "C++"` functions.** Every one that Rust threads call queues its work onto the GUI thread (`QMetaObject::invokeMethod` with `Qt::QueuedConnection`, `OnGuiThread` in `platform.cc`), so any thread may call it and none waits for the GUI (`EventHub` and `WakeHub` queue the events and wakes the same way); the exceptions are `run_app`, which starts the GUI, and `qt_version`, which only reads Qt's version.
  - Without a `QCoreApplication` there is nothing to queue on: `request_dialog` then answers at once with no choice.
- **Events that arrive in one event loop turn are merged** (`EventHub::Post` merges the kinds and edited node ids), so a panel rebuilds at most once per turn.
  - The Inspector, the Mixer and the Run panel compare the new view with the last and skip an unchanged one; their view structs derive `PartialEq`, for which `cxx` generates `operator==`, and most other views don't.
- **A player's frames cross without copies; other pictures are JPEG and decoded.** Rust returns a `FrameRef` (a boxed `Arc` of bytes), and `FilePlayer` builds a `QImage` over its RGB bytes whose cleanup drops the box; the streamed views, the live picture and the terminal window decode their JPEG bytes (`QImage::fromData`).
  - Rust calls `wake(target)` when a new frame or state is ready, and the widgets listening on that target (`WakeHub::Listen`) pull it; a playing `FilePlayer` also reads its player's state on a timer.
- **A drag, a picker or live typing is one undo step**: `FormHold::Begin` and `End` wrap the session's `begin_gesture` and `end_gesture`.
  - Holds nest: only the outermost `End` ends the gesture and emits `Released`.
  - The Inspector and the Mixer skip rebuilds while a hold is open (`Held`) and refresh on `Released`.
- **Every colour is a token and every icon a bundled glyph.** `style.qss` names tokens as `@name`, which `look::style_sheet` replaces from `TOKENS`; C++ asks `App::Token` and `App::Icon`. Glyphs come only from `crates/ui/glyphs.generated.json`; `resources.qrc` holds only the logo.
- **A modal file dialog is asked for from Rust and answered back**: the session calls `request_dialog` with an id, C++ shows the system's dialog and returns the choice through `answer_dialog`.
  - On Linux `App::Run` sets `QT_QPA_PLATFORMTHEME` to `xdgdesktopportal` unless it is set, so the dialog is the desktop portal's; scenarios (`OPSIS_QT_SCENARIO`) turn native dialogs off (`AA_DontUseNativeDialogs`).
- **A new C++ file is listed in `SOURCES` in `crates/qt/build.rs`, and in `MOC_HEADERS` when its header declares a `Q_OBJECT`**: the build compiles only those lists, while `scripts/cpp.ts` formats and lints every file in `crates/qt/cpp`.
- **`unsafe` in Rust is allowed only in `opsis-qt`, for what `cxx` generates, and `opsis-vt`**; `opsis-ui` and every other crate deny it.
- **`cxx` refuses attributes inside the bridge**, so the lint expectations its generated code needs sit on `mod bridge;` in `crates/qt/src/lib.rs`.
- **C++ follows the Google style**, formatted and checked by `scripts/cpp.ts` (`bun run format:fix`, `bun run lint:fix`), with no comments, like the rest of the codebase.

## How it works

- **Why Qt 6.12 LTS Widgets**: five years of fixes; widgets paint with QPainter's raster engine, so no GPU, driver, virtual machine or Xvfb question; native Wayland with text-input-v3 input methods, drag and drop including files from the file manager, and fractional scaling; accessibility through UIA, NSAccessibility and AT-SPI. Checked on 2026-11-05 against Avalonia (the runner-up, on XWayland by default with an experimental Wayland backend), Qt Quick with CXX-Qt (GPU drawing, an API that breaks twice a year), Slint, GTK 4, egui, Iced, Flutter, Compose, JavaFX, Godot and others, each of which failed a stable release, more than one active maintainer, drawing without a GPU, native Wayland, docking or tests.
- **Why KDDockWidgets and `cxx`**: KDDockWidgets nests, tabs and floats native windows that dock back, built for Widgets first; `cxx` is 1.0 and stable, with `cxx-build` compiling the C++ from `build.rs` and `qt-build-utils` finding Qt and running `moc` and `rcc`.
- **Start:** `start` (`crates/ui/src/main.rs`) builds the Tokio runtime, the `opsis_session::Session` and the `Ui`, then `opsis_qt::run` hands the host to C++; a failed start logs why and exits before Qt exists. `App::Run` makes the `QApplication`, loads the fonts and the style sheet, starts the host's event pump (`Host::start_pump`), a thread that turns each session event into `post_event` and the payload calls (`post_edited`, `post_replay`, `post_disk_change`), then opens the main window.
- **Exit:** `App::Run` deletes the remaining windows and drops the host before the `QApplication` ends, so nothing in Rust outlives the GUI it posts to; dropping the `Ui` closes the players and the sound output. `main` then ends the session (`Session::end`), installs a downloaded update ([updates.md](updates.md)) and exits with Qt's code.
