# Native editor bridge

How the editor's Qt Widgets views and the Rust session talk: the `cxx` bridge, which side owns what, how events and frames cross, and the rules for C++ views. Paths are relative to the repository root. The editor's windows and panels are in [editor.md](editor.md), the document they edit in [document.md](document.md), the build of Qt and KDDockWidgets and why the editor uses them in [toolchain.md](toolchain.md), the scenarios that drive the views in [e2e.md](e2e.md), the Inspector's forms in [inspector.md](inspector.md), installing an update at exit in [updates.md](updates.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                          | Where                                                                                                                                                                    |
| -------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| The app binary and its host      | `crates/ui/src/main.rs` → `start`; `crates/ui/src/ui.rs` (`Ui`, the `opsis_qt::Host` implementation); `crates/ui/src/platform.rs` → `QtPlatform`                         |
| View builders                    | `crates/ui/src/`, one module per view (`graph.rs`, `inspector.rs`…)                                                                                                      |
| The bridge                       | `crates/qt/src/bridge.rs` (`cxx::bridge`: the view structs, `extern "Rust"` session calls, `extern "C++"` functions), `crates/qt/src/host.rs` (`Host`, `Session`, `run`) |
| The C++ side of the bridge       | `crates/qt/cpp/bridge_ffi.h`; `crates/qt/cpp/platform.cc` (`OnGuiThread`, `request_dialog`, the window, path and notification calls)                                     |
| Building the C++                 | `crates/qt/build.rs` → `MOC_HEADERS`, `SOURCES`; `crates/qt/resources/resources.qrc`                                                                                     |
| Frames                           | `crates/qt/src/frame.rs` (`FrameRef`)                                                                                                                                    |
| Session events to editor kinds   | `crates/ui/src/ui.rs` → `start_pump`, `event_kind`                                                                                                                       |
| Event delivery on the GUI thread | `crates/qt/cpp/event_hub.cc` (`EventHub`, `post_event`, `post_edited`, `post_replay`, `Dispatch`), `crates/qt/cpp/wake_hub.cc` (`WakeHub`, `wake`)                       |
| One undo step for a gesture      | `crates/qt/cpp/form_hold.cc` (`FormHold`)                                                                                                                                |
| The application object           | `crates/qt/cpp/app.cc` (`App`: `Run`, tokens, icons, fonts)                                                                                                              |
| Look                             | `crates/ui/src/look.rs` (`TOKENS`, `style_sheet`), `crates/ui/resources/style.qss`, `crates/ui/glyphs.generated.json` (`bun run glyphs:bundle`)                          |
| C++ format and lint              | `scripts/cpp.ts` ([toolchain.md](toolchain.md))                                                                                                                          |

## Rules

- **Rust owns every piece of state and decides everything a Rust function can; C++ draws and forwards input.** A view is a plain struct Rust builds when C++ asks (`GraphView`, `InspectorView`, `TimelineView`, `MixerView`…), and C++ keeps only the last view it drew, never project data of its own.
  - Pure logic (edge lanes, snapping, ducking, envelopes, layout, schema reading) lives in Rust with property tests, never in C++.
- **Every Qt call happens on the GUI thread, and C++ owns every `QObject` through Qt's parent tree**: Rust never holds or deletes one, and C++ holds the Rust host as a `rust::Box<Session>` (`App::Run`).
- **C++ calls Rust through the `Session` methods of the bridge.**
  - A call that can fail returns `Result` in the bridge (`HostResult` in `host.rs`), so C++ catches `rust::Error` and shows it (a toast or `App::ShowError`); an uncaught one is a C++ exception.
- **Rust reaches C++ only through the `extern "C++"` functions, and every one that Rust threads call queues its work onto the GUI thread** (`QMetaObject::invokeMethod` with `Qt::QueuedConnection`, `OnGuiThread` in `platform.cc`), so any thread may call it and none waits for the GUI (`EventHub` and `WakeHub` queue the events and wakes the same way); the exceptions are `run_app`, which starts the GUI, and `qt_version`, which only reads Qt's version.
  - Without a `QCoreApplication` there is nothing to queue on: `request_dialog` then answers at once with no choice, and posted events and wakes are dropped.
- **Session events reach the views as editor event kinds through the host's pump (`start_pump`)**, and `EventHub::Dispatch` turns each kind into a Qt signal.
  - The pump drops events addressed to another window and handles those with payloads itself: it keeps each `Compiled` and starts its mix, shows the project on `OpenRequested`, posts `DocumentChanged`'s edited nodes (`post_edited`), the replay asked for (`post_replay`) and the disk question (`post_disk_change`), and keeps engine setup progress, live notes and streamed-view events for their views.
  - Every other event it maps one to one (`event_kind`); a lagging pump posts only `Document` ([document.md](document.md)).
- **Events of one kind that arrive in one event loop turn are merged** (`EventHub::Post` merges each kind and unions the edited node ids; `post_replay` isn't merged), so a view that listens to one kind rebuilds at most once per turn.
  - The Inspector, the Mixer, the Run panel and the Timeline canvas compare the new view with the last and skip an unchanged one; their view structs derive `PartialEq`, for which `cxx` generates `operator==`, and most other views don't.
- **A player's frames cross without copies**: Rust returns a `FrameRef` (a boxed `Arc` of bytes), and `FilePlayer` builds a `QImage` over its RGB bytes whose cleanup drops the box.
- **Other pictures cross as JPEG and are decoded**: the streamed views, the live picture and the terminal window decode their bytes (`QImage::fromData`).
  - Rust calls `wake(target)` when a new frame or state is ready, and the widgets listening on that target (`WakeHub::Listen`) pull it; a playing `FilePlayer` also reads its player's state on a timer.
- **A drag, a picker or live typing is one undo step.**
  - Form fields and pickers (the Inspector, the Mixer, the colour, font and gradient pickers) hold it through `FormHold`, whose `Begin` and `End` wrap the session's `begin_gesture` and `end_gesture`; holds nest, and only the outermost `End` ends the gesture and emits `Released`.
  - The canvas and the Timeline call `begin_gesture` and `end_gesture` themselves around a drag (`graph_canvas.cc`, `timeline_canvas.cc`).
  - The Inspector and the Mixer skip rebuilds while a hold is open (`Held`) and refresh on `Released`; the Inspector's check (`InspectorPanel::Editing`) also counts a focused field that holds a draft.
- **Colours are tokens and icons bundled glyphs** (`style.qss` names tokens as `@name`, which `look::style_sheet` replaces from `TOKENS`; C++ asks `App::Token` and `App::Icon`); the editor's look is in [editor.md](editor.md).
  - Glyphs come only from `crates/ui/glyphs.generated.json`; `resources.qrc` holds only the logo.
- **A modal file dialog is asked for from Rust and answered back**: the session calls `request_dialog` with an id, C++ shows the system's dialog and returns the choice through `answer_dialog`.
  - On Linux `App::Run` sets `QT_QPA_PLATFORMTHEME` to `xdgdesktopportal` unless it is set, so the dialog is the desktop portal's; scenario runs use Qt's own dialogs ([e2e.md](e2e.md)).
- **A new C++ file is listed in `SOURCES` in `crates/qt/build.rs`, and in `MOC_HEADERS` when its header declares a `Q_OBJECT`**: the build compiles only those lists, so a `.cc` left out isn't built.
  - `scripts/cpp.ts` formats every tracked `.cc` and `.h` in `crates/qt/cpp` and lints every tracked `.cc`, reading the compile flags from the `compile_commands.json` that `build.rs` writes from `SOURCES`, so a `.cc` left out of `SOURCES` is linted without its flags.
- **`opsis-qt` is one of the two crates allowed `unsafe`, for what `cxx` generates** ([AGENTS.md](../AGENTS.md) §7).
- **`cxx` refuses attributes inside the bridge**, so the lint expectations its generated code needs sit on `mod bridge;` in `crates/qt/src/lib.rs`.
- **C++ follows the repository's C++ style and checks** ([toolchain.md](toolchain.md), C++).

## How it works

- **Start:** `start` (`crates/ui/src/main.rs`) builds the Tokio runtime, the `opsis_session::Session` and the `Ui`, then `opsis_qt::run` hands the host to C++; a failed start logs why and exits before Qt exists. `App::Run` makes the `QApplication`, loads the fonts and the style sheet, starts the host's event pump (`Host::start_pump`), a thread that turns each session event into `post_event` and the payload calls (`post_edited`, `post_replay`, `post_disk_change`), then opens the main window.
- **Adding a call from C++ to Rust, or a view:** declare the struct (with `PartialEq` only when C++ compares it) and the call (`fn x(self: &Session, …)`, `Result` when it can fail, `Box<FrameRef>` for a frame) in `crates/qt/src/bridge.rs`; add the method to `Host` and its `Session` forwarder in `crates/qt/src/host.rs`; export the struct from `crates/qt/src/lib.rs`; implement it in `impl Host for Ui` (`crates/ui/src/ui.rs`), building the view in its own module; list new C++ files as above.
- **Adding a call from Rust to C++:** declare it in the `extern "C++"` block of `crates/qt/src/bridge.rs` and in `crates/qt/cpp/bridge_ffi.h`; define it in C++ (`platform.cc`, or `event_hub.cc` and `wake_hub.cc` for events), queuing onto the GUI thread as the rules above say; wrap it in a `pub fn` in `crates/qt/src/host.rs` and export that from `crates/qt/src/lib.rs`.
- **Exit:** `App::Run` deletes the remaining windows and drops the host before the `QApplication` ends, so nothing in Rust outlives the GUI it posts to; dropping the `Ui` closes the players and the sound output. `main` then ends the session (`Session::end`), installs a downloaded update ([updates.md](updates.md)) and exits with Qt's code.
