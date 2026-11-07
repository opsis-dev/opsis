# Native editor bridge

How the editor's Qt Widgets views and the Rust session talk: the `cxx` bridge, which side owns what, how events and frames cross, and the rules for C++ views. Paths are relative to the repository root. The editor's windows and panels are in [editor.md](editor.md), the document they edit in [document.md](document.md), the build of Qt and KDDockWidgets in [toolchain.md](toolchain.md), the scenarios that drive the views in [e2e.md](e2e.md).

## Code map

| Concern                          | Where                                                                                                                                                                                                 |
| -------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| The app binary and its host      | `crates/ui/src/main.rs`, `crates/ui/src/ui.rs` (`Ui`, the `opsis_qt::Host` implementation)                                                                                                            |
| View builders                    | `crates/ui/src/` one module per view: `graph.rs`, `inspector.rs`, `timeline.rs`, `mixer.rs`, `run_panel.rs`, `replay.rs`, `live.rs`, `streams.rs`, `terminal_window.rs`, `player.rs`, `font_views.rs` |
| The bridge                       | `crates/qt/src/bridge.rs` (`cxx::bridge`: the view structs, `extern "Rust"` session calls, `extern "C++"` functions), `crates/qt/src/host.rs` (`Host`, `Session`, `run`)                              |
| Frames                           | `crates/qt/src/frame.rs` (`FrameRef`)                                                                                                                                                                 |
| Event delivery on the GUI thread | `crates/qt/cpp/event_hub.cc` (`EventHub`, `post_event`, `post_edited`), `crates/qt/cpp/wake_hub.cc` (`WakeHub`, `wake`)                                                                               |
| One undo step for a gesture      | `crates/qt/cpp/form_hold.cc` (`FormHold`)                                                                                                                                                             |
| The application object           | `crates/qt/cpp/app.cc` (`App`: `Run`, tokens, icons, fonts)                                                                                                                                           |
| Look                             | `crates/ui/src/look.rs` (`TOKENS`, `style_sheet`), `crates/ui/resources/style.qss`, `crates/ui/glyphs.generated.json` (`bun run glyphs:bundle`)                                                       |
| C++ format and lint              | `scripts/cpp.ts` (pinned clang-format and clang-tidy, the check list)                                                                                                                                 |

## Rules

- **Rust owns every piece of state; a view is a plain struct Rust builds when C++ asks** (`GraphView`, `InspectorView`, `TimelineView`, `MixerView`…). C++ keeps only the last view it drew, never project data of its own.
- **C++ calls Rust through the `Session` methods of the bridge; Rust reaches C++ only through the `extern "C++"` functions**, and each of those queues its work onto the GUI thread (`QMetaObject::invokeMethod` with `Qt::QueuedConnection`), so any Rust thread may call them and none waits for the GUI.
- **A panel rebuilds once per event loop turn and only when its view changed.** `EventHub::Post` merges the kinds and edited node ids that arrive in one turn; the view structs derive `PartialEq`, `cxx` generates `operator==`, and a panel compares the new view with the last before rebuilding.
- **Pictures cross without copies.** Rust returns a `FrameRef` (a boxed `Arc` of RGB or JPEG bytes); C++ builds a `QImage` over those bytes whose cleanup drops the box. Rust calls `wake(target)` when a new frame or state is ready, and the widgets listening on that target (`WakeHub::Listen`) pull it; nothing polls.
- **A drag, a picker or live typing is one undo step**: `FormHold::Begin` and `End` wrap the session's `begin_gesture` and `end_gesture`, and panels hold their rebuilds while a hold is open.
- **Every colour is a token and every icon a bundled glyph.** `style.qss` names tokens as `@name`, which `look::style_sheet` replaces from `TOKENS`; C++ asks `App::Token` and `App::Icon`. Glyphs come only from `crates/ui/glyphs.generated.json`.
- **A modal file dialog is asked for from Rust and answered back**: the session calls `request_dialog` with an id, C++ shows the system's dialog (the desktop portal on Linux) and returns the choice through `answer_dialog`.
- **`unsafe` stays in `opsis-qt`**, which `cxx` needs; `opsis-ui` and every other crate deny it.
- **`cxx` refuses attributes inside the bridge**, so the lint expectations its generated code needs sit on `mod bridge;` in `crates/qt/src/lib.rs`.
- **C++ follows the Google style**, formatted and checked by `scripts/cpp.ts` (`bun run format:fix`, `bun run lint:fix`), with no comments, like the rest of the codebase.

## How it works

- **Start:** `main.rs` builds the Tokio runtime, the `opsis_session::Session` and the `Ui`, then `opsis_qt::run` hands the host to C++. `App::Run` makes the `QApplication`, loads the fonts and the style sheet, opens the main window and starts the host's event pump (`Host::start_pump`), a thread that turns each session event into `post_event` and the payload calls (`post_edited`, `post_replay`, `post_disk_change`).
- **Exit:** `App::Run` deletes the remaining windows and drops the host before the `QApplication` ends, so nothing in Rust outlives the GUI it posts to; dropping the `Ui` closes the players and the sound output.
