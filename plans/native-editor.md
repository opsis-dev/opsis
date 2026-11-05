# Native editor

How Opsis replaces its editor, a React app in each OS's own webview (WebKitGTK, WKWebView, WebView2), with Qt 6.12 LTS Widgets: a C++ view layer that Rust drives through `cxx`, drawn the same on Linux, Windows and macOS, in the same process as the document, the compile and the run.

Status: plan only. Nothing here is built yet. Facts were checked on 4–5 November 2026.

## How to read this plan

- **Session** comes first: it takes everything that isn't Tauri out of `crates/desktop`, and the Tauri editor keeps working on it until the cut-over.
- **Build** and **Bridge** come next; every view needs them.
- **Shell**, **Look**, **Docking**, then the views (**Graph canvas**, **Inspector**, **Timeline**, **Mixer and audio**, **Playback and pictures**, **Other windows**) in any order once the shell runs.
- **Testing**, **Packaging** and **Cut-over** close it.
- Each part has the same sections: **What it changes**, **Fit with Opsis** (In place / Missing), **Work**, **Decisions**, **Risks**, **Sources**.

## Conventions

These restate rules from `AGENTS.md` and `agent-docs/`; when they disagree with this plan, they win. When a part ships, its rules move into the domain docs (`editor.md`, `document.md`, `e2e.md`, `packaging.md`, `toolchain.md`), and a new `agent-docs/ui.md` holds the bridge and the view layer.

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
- **`QGraphicsView` for the graph editor and the timeline**, the scene that Natron's `NodeGraph` and Olive's `NodeView` are built on.
- **KDDockWidgets 2.5.1 (KDAB) for docking**: nesting, tabs, and floating native windows that dock back, built for Widgets first.
- **`cxx` 1.0 for the bridge** (1.0.205, 85 million downloads), with `cxx-build` compiling the C++ side from `build.rs`, and `qt-build-utils` (KDAB) finding Qt and running `moc` and `rcc`.
- **Why not the alternatives** is under [Alternatives](#alternatives).

## Summary

| Part                  | What it changes                                                            | Needs          |
| --------------------- | -------------------------------------------------------------------------- | -------------- |
| Session               | a crate with no UI holding everything in `crates/desktop` that isn't Tauri | —              |
| Build                 | Qt 6.12, `moc`, `rcc`, KDDockWidgets and the C++ side built from Cargo     | —              |
| Bridge                | typed calls from C++ into the session; queued events back                  | Session, Build |
| Shell                 | windows, title bar, menus, shortcuts, exit flow                            | Bridge         |
| Look                  | tokens, fonts, icons, motion                                               | Shell          |
| Docking               | panels, tabs, collapse, pop-out windows, saved layouts                     | Shell          |
| Graph canvas          | cards, ports, edges in lanes, selection, minimap, tidy-up                  | Docking        |
| Inspector             | the schema form and its 30 widget kinds                                    | Docking        |
| Timeline              | tracks, filmstrips, waveforms, envelopes, playhead                         | Docking        |
| Mixer and audio       | strips, meters, EQ; audio output replacing Web Audio                       | Docking        |
| Playback and pictures | film playback, image preview, streamed views, live picture                 | Bridge         |
| Other windows         | Settings, start screen, setup, fonts, Live, terminal window                | Shell          |
| Testing               | Rust tests, Qt view tests, end-to-end through the Local API                | each view      |
| Packaging             | deb, rpm, AppImage, NSIS, macOS app with Qt bundled                        | Build          |
| Cut-over              | remove Tauri, the webview editor, WebdriverIO; rewrite the docs            | everything     |

## Session

### What it changes

Everything in `crates/desktop` that isn't Tauri moves into `crates/session` (`opsis-session`): state, the document, the compile, the run, shared state, settings, fonts, engine setup, views, the exit decision. Events become one Rust channel instead of Tauri emits. Both the Tauri app and the Qt app run on it, so the Tauri editor keeps working until the cut-over.

### Fit with Opsis

**In place.**

- Only `crates/desktop` depends on Tauri. `opsis-document`, `opsis-compile`, `opsis-runner` and the rest are Tauri-free.
- `crates/desktop/src`: `state.rs` (`AppState`), `document.rs`, `compile.rs`, `run.rs`, `shared.rs`, `settings.rs`, `fonts.rs`, `engine.rs`, `projects.rs`, `views.rs`, `terminal_live.rs`, `terminal_paint.rs`, `frames.rs`, `exit.rs`, `node_results.rs`, `analysis.rs`, `gradients.rs`, `api.rs` (`ApiHost`), `windows.rs`, `menu.rs`, `native_menu.rs`, `file_server.rs`, `page_log.rs`, `startup.rs`, `paths.rs`.
- 95 commands and 28 events registered in `bindings.rs`.

**Missing.** A crate with no `AppHandle`: `AppState` behind `Arc<Session>`, a `tokio` runtime handle, and a `SessionEvent` enum with one variant per event `bindings.rs` registers today.

### Work

1. Create `crates/session` with the modules above minus `windows.rs`, `menu.rs`, `native_menu.rs`, `file_server.rs`, `page_log.rs` and `bindings.rs`. Window bounds logic (`visible_bounds`, `bounds_key`, the database rows) moves too; only the code that builds windows stays behind.
2. Replace every `Event::emit(app)` with a send on a `tokio::sync::broadcast::Sender<SessionEvent>`; `SessionEvent` derives `Debug` and carries the same payload types (`opsis-types`).
3. Replace `AppHandle` parameters with `&Session`; keep the per-window concerns (which window a selection came from) as plain ids.
4. `crates/desktop` keeps its commands as thin wrappers that call the session and forward `SessionEvent`s to `emit`, so the Tauri editor behaves exactly as before. `ApiHost` (`crates/desktop/src/api.rs`) moves into the session.
5. Register long work in the session's task list (`begin`/`end`), as `AppState.tasks` does today.
6. Move the session's rules into `agent-docs/document.md` and name the crate in `AGENTS.md` §6.

### Decisions

- **Broadcast or one receiver per window.** A broadcast channel keeps today's "every window follows" model; a lagging receiver loses old events, so each view re-reads current state on `Lagged`, as windows take `document_state` on mount today.

### Risks

- A missed `emit` becomes a silent gap; the existing e2e suite runs against the Tauri app on the session before any Qt code lands.

### Sources

- tokio broadcast: https://docs.rs/tokio/latest/tokio/sync/broadcast/ (checked 2026-11-05)

## Build

### What it changes

A new crate `crates/ui` (`opsis-ui`) builds the C++ view layer and links Qt 6.12 and KDDockWidgets, from `cargo build`.

### Fit with Opsis

**In place.** One pinned Rust nightly; `build.rs` scripts already compile foreign code (`crates/vt/build.rs` builds the Zig wrapper and refuses another Zig version).

**Missing.** Qt 6.12 on every developer machine and CI runner; a C++ compiler; `moc` for classes with signals; KDDockWidgets built from source; a comment check, formatter and linter for C++.

### Work

1. `crates/ui/build.rs`:
   1. find Qt with `qt-build-utils` (`qtpaths`), and refuse any Qt below 6.12, as `crates/vt/build.rs` refuses another Zig;
   2. run `moc` on the headers that declare `Q_OBJECT`, and `rcc` on the resource file (fonts, icons, the style sheet);
   3. build KDDockWidgets with the `cmake` crate (`-DKDDockWidgets_FRONTENDS=qtwidgets`), pinned to a release tag by URL and hash like Ghostty in `crates/vt`;
   4. compile the bridge and the C++ sources with `cxx-build`;
   5. link Qt Widgets, Gui, Core, Svg, KDDockWidgets.
2. Qt for developers: install Qt 6.12 with `aqtinstall` into `~/.cache/opsis-qt` (large files go to `~/.cache`), found through an `OPSIS_QT` environment variable; a script `bun run qt:install` does it. Distro packages are not used, because their Qt version varies.
3. CI: `jurplel/install-qt-action` (v4.5.0) installs Qt 6.12 on Linux, Windows and macOS runners.
4. C++ standard: C++17, the standard Qt 6 requires; raise it only when a dependency asks. KDDockWidgets' Qt 6 Widgets build needs Qt's private Widgets headers (`find_package(Qt6 … COMPONENTS WidgetsPrivate)` in its `CMakeLists.txt`), which `aqtinstall` installs with Qt.
5. Tooling: `scripts/comments.ts` reads `.cpp` and `.h` line by line like Rust; `clang-format` with a checked-in `.clang-format` runs in `format:fix`; `clang-tidy` in `lint:fix`. Record both in `agent-docs/toolchain.md` with their pinned versions.

### Decisions

- **KDDockWidgets from source or from a package.** From source keeps every OS on the same pinned version; it needs CMake on developer machines and runners.
- **Which C++ linter checks.** Start with `clang-tidy`'s `bugprone-*`, `modernize-*` and `cppcoreguidelines-*` sets, tuned once against the first views.

### Risks

- Qt and KDDockWidgets add a C++ build to every `cargo build` of `opsis-ui`; keep `opsis-ui` out of crates that don't need it so their tests don't build Qt.
- KDDockWidgets builds against Qt's private headers, which have no compatibility promise between Qt minor versions; moving Qt means rebuilding and re-checking KDDockWidgets.
- `qt-build-utils` is 0.x (0.10.0) and moves with CXX-Qt's releases; Opsis uses only Qt discovery, `moc` and `rcc`, which can be done in `build.rs` directly if it breaks.

### Sources

- Qt 6.12 release: https://qt.io/blog/qt-6.12-released (checked 2026-11-04)
- qtbase: https://github.com/qt/qtbase (checked 2026-11-05)
- cxx and cxx-build: https://github.com/dtolnay/cxx (1.0.205, checked 2026-11-05)
- qt-build-utils: https://github.com/KDAB/cxx-qt/tree/main/crates/qt-build-utils (0.10.0, checked 2026-11-05)
- cmake crate: https://github.com/rust-lang/cmake-rs (0.1.58, checked 2026-11-05)
- KDDockWidgets: https://github.com/KDAB/KDDockWidgets (2.5.1, checked 2026-11-05)
- aqtinstall: https://github.com/miurahr/aqtinstall (v3.3.0, checked 2026-11-05)
- install-qt-action: https://github.com/jurplel/install-qt-action (v4.5.0, checked 2026-11-05)

## Bridge

### What it changes

The C++ views call the session through a `cxx::bridge` module; the session's events reach the views on Qt's GUI thread.

### Fit with Opsis

**In place.** Every command and event is already a typed Rust function or struct (`bindings.rs`, `opsis-types`); the Inspector's schema is JSON Schema from `schemars`.

**Missing.** The bridge module, the event pump, and C++ views of the shapes they need.

### Work

1. `crates/ui/src/bridge.rs`: `#[cxx::bridge]` with
   - `Session` as an opaque Rust type, created by `start_session()` and owned by C++ through `rust::Box`;
   - one Rust function per command the views use (`edit_workflow`, `undo`, `redo`, `begin_gesture`, `end_gesture`, `edit_timeline`, `run_workflow`, `cancel_run`, …), taking and returning shared structs;
   - shared structs for the hot paths: the document snapshot (nodes, edges, positions, revision), selection, playhead, compile results, run progress, meter levels;
   - `serde_json` strings for the cold paths that are JSON already (the Inspector's schema, settings sections).
2. The event pump: a Rust thread reads `SessionEvent`s and calls one C++ function, `post_event(kind, payload)`, which queues a lambda onto the GUI thread with `QMetaObject::invokeMethod(qApp, …, Qt::QueuedConnection)`. Views subscribe to the event kinds they show.
3. Models: C++ `QAbstractItemModel` subclasses over the snapshot structs for lists and tables (node list, run list, fonts, recent projects), so views virtualise rows.
4. Errors: a Rust function that fails returns `Result`; `cxx` turns it into a C++ exception, caught at the view boundary and shown as today's toast or dialog.
5. Panics: `std::panic::catch_unwind` at every bridge entry; a caught panic is logged and reported like a page error today.

### Decisions

- **Typed structs or JSON for each call.** Typed for anything a view reads every frame or every edit; JSON where Rust already produces JSON (schemas). Revisit any JSON call that shows in a profile.
- **One bridge module or one per domain.** One per domain (`document`, `run`, `settings`, `media`) keeps rebuilds small.

### Risks

- `cxx` is effectively one maintainer (232 of 243 commits in 12 months by dtolnay), offset by its 1.0 stability and wide use; pin it like every dependency.
- A lambda queued after its view is deleted touches freed memory; queue onto the view (`QPointer` checked in the lambda), never onto raw pointers.

### Sources

- cxx book: https://cxx.rs (checked 2026-11-05)
- `QMetaObject::invokeMethod`: https://doc.qt.io/qt-6/qmetaobject.html#invokeMethod (checked 2026-11-05)

## Shell

### What it changes

Windows, the title bar, menus, shortcuts and the exit flow, without a system title bar, as today.

### Fit with Opsis

**In place.** `menu.rs` → `app_menu` (one list for the native macOS menu and the in-app menubar); `windows.rs` (bounds per project and globally, the macOS overlay title bar with traffic lights at 16, 16); `exit.rs` (`ConfirmClose`, drafts, `close_on_signals`); `app/window-drag.ts` (`DRAG_START_PX`, double-click toggles maximize); `app/app-bar.tsx`, `WindowControls`, `WindowBorder`.

**Missing.** Qt versions of each.

### Work

1. Windows: `QMainWindow` for the main window; `Qt::FramelessWindowHint` on Linux and Windows; on macOS the title bar extended into the content with `Qt::ExpandedClientAreaHint` (Qt 6.9+), traffic lights inside the app bar at 16, 16.
2. App bar widget: drag after `DRAG_START_PX` with `QWindow::startSystemMove`, double-click toggles maximize, edges resize with `startSystemResize`; our own window buttons on Linux and Windows; a 1 px border while neither maximized nor full screen.
3. Menus: `app_menu` from the session builds a `QMenuBar`; on macOS it is the native menu bar, on Linux and Windows the in-app menubar from 1280 px and one Menu button below.
4. Shortcuts: each menu accelerator is a `QAction` shortcut; cut, copy, paste, select all, undo and redo go to a focused text field first.
5. Windows by role: main (start screen, setup, editor), Settings, panel pop-outs (Docking), Live, streamed view, terminal window; a `WindowRole` enum replaces the hash routes; bounds come from the session.
6. Exit: `closeEvent` asks the session; the in-app exit dialog lists unfinished work as today. The 2 s and 3 s watchdogs for a page that dies go: there is no page.
7. Signals: `close_on_signals` stays in the session.
8. Notifications (`tauri-plugin-notification` today): `QSystemTrayIcon::showMessage` or the platform notification service, decided below.
9. Opening files and links (`tauri-plugin-opener`): `QDesktopServices::openUrl`. File dialogs (`tauri-plugin-dialog`): `QFileDialog`, which uses the desktop portal on Linux.

### Decisions

- **Notifications.** `QSystemTrayIcon::showMessage` needs a tray icon on Linux; check what Opsis notifies today and whether an in-app toast covers it.

### Risks

- Wayland gives clients no window positions; pop-out and Settings bounds keep only their size there, as with any Wayland client.

### Sources

- Frameless windows and system move: https://doc.qt.io/qt-6/qwindow.html#startSystemMove (checked 2026-11-05)
- Expanded client areas (Qt 6.9): https://www.qt.io/blog/expanded-client-areas-and-safe-areas-in-qt-6.9 (checked 2026-11-04)
- `QMenuBar` on macOS: https://doc.qt.io/qt-6/qmenubar.html (checked 2026-11-05)

## Look

### What it changes

The dark zinc look, Inter and JetBrains Mono, icons and motion, in Qt.

### Fit with Opsis

**In place.** Colour tokens in `apps/frontend/src/index.css` (hue 286, panels on `--sidebar`, canvas on `--background`, `--toggle-on`, `--success`, `--live`); bundled fonts in `public/fonts/bundled`; the glyph sets bundled by `glyphs:bundle`; `app/motion.tsx` respecting reduced motion.

**Missing.** A token source both the style sheet and painted widgets read; SVG icons for Qt; animations.

### Work

1. Tokens: one Rust table of colour and size tokens (moved out of `index.css`), exported to a Qt style sheet template with variables, the way OBS Studio's themes use `var(--border_radius)` in their style sheets. Painted widgets read the same table through the bridge.
2. Fonts: Inter and JetBrains Mono from `public/fonts/bundled` through `QFontDatabase::addApplicationFont` at start.
3. Icons: `glyphs:bundle` writes SVG files instead of `node-glyphs.generated.ts`; `QIcon` and `QSvgRenderer` (Qt Svg) draw them, tinted from tokens.
4. Motion: `QPropertyAnimation` and `QVariantAnimation` for panel collapse, popovers and toasts, skipped when the platform asks for reduced motion.
5. A `QProxyStyle` for what style sheets can't express (focus rings, pressed toggle colour).

### Decisions

- **Where the token table lives.** In `opsis-ui`, since nothing else draws the editor.

### Risks

- Qt style sheets don't do every CSS effect (no backdrop blur; shadows only through `QGraphicsDropShadowEffect`, which is slow on large widgets); keep the design to what the zinc tokens already use: flat panels, borders, rounded corners.

### Sources

- Qt style sheets: https://doc.qt.io/qt-6/stylesheet-reference.html (checked 2026-11-05)
- OBS Studio's themes: https://github.com/obsproject/obs-studio/tree/master/frontend/data/themes (checked 2026-11-05)

## Docking

### What it changes

The node list, the Inspector and the bottom tabs (Timeline, Mixer, Run) as KDDockWidgets docks: resizable, collapsible, and any tab opens in its own window and docks back when closed.

### Fit with Opsis

**In place.** `editor/panels.ts` (handles, open state, active tab), default sizes in `editor/editor.tsx` (node list 17% within 12–30%, centre 59%, canvas 60% over the bottom panel 40%, Inspector 24% within 16–40%), the 44 px category rail, pop-out bounds in the database, per-project view state (`view_state` table).

**Missing.** Docks, the collapse rail, layout saving.

### Work

1. Dock widgets for the node list, Inspector, Timeline, Mixer and Run; the canvas is the central widget.
2. Collapse: Ctrl/Cmd+B, Ctrl/Cmd+Alt+B and Ctrl/Cmd+J as today; the collapsed node list becomes the 44 px rail with popovers.
3. Pop-out: KDDockWidgets' floating windows replace `open_panel_window`; closing one docks it back.
4. Layout: KDDockWidgets' `LayoutSaver` writes the layout into the `view_state` row the session keeps per project.

### Risks

- KDDockWidgets is GPL-2.0-only or GPL-3.0-only (or commercial); Opsis takes GPL-3.0-only, so the shipped binary is GPL-3.0-only while Opsis's own source stays GPL-3.0-or-later. Record it in `agent-docs/packaging.md`.

### Sources

- KDDockWidgets: https://github.com/KDAB/KDDockWidgets (2.5.1, checked 2026-11-05)

## Graph canvas

### What it changes

The node graph moves from React Flow to a `QGraphicsScene`: cards with typed ports, step edges in lanes, selection, connecting, groups, the minimap, the zoom slider, mouse and touchpad modes, tidy-up layout.

### Fit with Opsis

**In place.**

- `editor/edge-lanes.ts` (`stepPositions`), `editor/layout.ts` (`measuredPorts`, ELK options: `layered`, `RIGHT`, `ORTHOGONAL`, `NETWORK_SIMPLEX`, ports with `FIXED_POS`/`FIXED_ORDER`), `editor/node-motions.ts`, `editor/use-flow-elements.ts` (the store owns positions, a drag writes one position at drag stop).
- Card looks from the catalogue (`nodes/looks.ts`), node glyphs, card rows (`card-rows.tsx`).
- Drops from the palette and from the file manager (`use-canvas-drop.ts`, `app/tauri.ts`).

**Missing.** Scene items, the lane and layout logic in Rust, the minimap view.

### Work

1. Port `edge-lanes.ts`, the port measuring in `layout.ts` and `node-motions.ts` to a Rust module in `opsis-compile` (pure, beside the catalogue) with property tests; the C++ side asks for lane points through the bridge.
2. Tidy-up: `elk-layout` (a Rust port of ELK 0.12 that takes elkjs's JSON), called from the session with today's options.
3. Items: a `CardItem` per node (header, rows, ports), `PortItem`s, `EdgeItem`s whose paths come from Rust lane points, `GroupItem`s; rubber-band selection; selection shared through the session.
4. Connecting: drag from a port draws a temporary edge; Rust validates the connection (`isValidConnection` today) before `edit_workflow`.
5. View: wheel zoom and drag pan in mouse mode; two-finger pan and pinch (`QNativeGestureEvent`) in touchpad mode; the zoom slider; reveal a node.
6. Minimap: a second `QGraphicsView` on the same scene, scaled to fit, with the visible rectangle drawn on top.
7. Drops: the palette starts a `QDrag` with Opsis's own MIME type; files from the file manager arrive as `QDropEvent` URLs and go to `import_assets`.
8. Positions: a drag moves items locally and writes one position at drag stop, inside a gesture, as today.

### Decisions

- **Where the graph logic lives.** `opsis-compile` has the catalogue and cards; a new pure crate is the alternative if `opsis-compile` grows too large.

### Risks

- `elk-layout` is young (created February 2026, one author). It reports 1,989 of 1,989 test models matching Java ELK; its licence (EPL-2.0 with GPL-3.0 as a secondary licence) must be added to `deny.toml`'s allow-list under its GPL-3.0 option.

### Sources

- Graphics View: https://doc.qt.io/qt-6/graphicsview.html (checked 2026-11-05)
- Natron's node graph: https://github.com/NatronGitHub/Natron/blob/master/Gui/NodeGraph.h (checked 2026-11-05)
- Olive's node view: https://github.com/olive-editor/olive/blob/master/app/widget/nodeview/nodeview.h (checked 2026-11-05)
- elk-layout (elk-rs): https://github.com/openedges/elk-rs, validation report https://github.com/openedges/elk-rs/blob/main/tests/release_0_12_2.md (0.12.2, checked 2026-11-04)

## Inspector

### What it changes

The schema-driven form and its 30 widget kinds (`FieldWidget` in `crates/compile/src/elements/introspect.rs`: Action, Asset, Backdrop, Boxes, Camera, Code, Color, Cut, Datetime, Envelope, Font, Gradient, Keys, Loudness, Marker, Marks, Multiselect, Music, Offsets, Port, Quality, Sound, Spot, Target, TerminalTheme, Textarea, Theme, Timezone, Transition, Variable), as Qt widgets.

### Fit with Opsis

**In place.** The schema and its `x-` keywords come from Rust (`#[setting]`, `schemars`); `InspectorContext` sends file kinds and take kinds; drafts held while typing count as one undo step (`use-typing-as-one-undo-step.ts`, `useHeldDrafts`); presets; the project Inspector.

**Missing.** A Rust reader of the schema for the form (today `editor/form/schema-kind.ts`, `schema-meta.ts`, `schema.ts`, `list-values.ts`, `decimal.ts`, `marks.ts` in TypeScript) and the widgets.

### Work

1. Port the schema reading (`schema-kind.ts`, `schema-meta.ts`, `x-show-if`, `x-show-if-file`, `x-show-if-take`, `x-unit`, `x-frames-at`, `x-zero-label`, list values, decimals) to Rust; it returns a field list the C++ form lays out.
2. One widget class per `FieldWidget` kind and per plain type (text, number with scrubbing, switch, enum), on `QLineEdit`, `QSpinBox`/`QDoubleSpinBox`, `QComboBox`, `QSlider`; custom painted editors for gradient, envelope, marks, spot, boxes, keys.
3. Drafts and gestures: focus starts a gesture, commit or blur ends it; the exit flow asks for held drafts as today.
4. Lists: text, number, palette and row lists with reorder, duplicate and collapse.
5. The font picker draws each family from bytes `opsis-fonts` provides (`QFontDatabase::addApplicationFontFromData`).

### Risks

- The TypeScript form has tests (`schema-form.test.tsx`, `schema-form-tabs.test.tsx`, field tests); port their properties to the Rust schema reader before the widgets.

### Sources

- Qt Widgets classes: https://doc.qt.io/qt-6/qtwidgets-module.html (checked 2026-11-05)
- `QFontDatabase`: https://doc.qt.io/qt-6/qfontdatabase.html (checked 2026-11-05)

## Timeline

### What it changes

Tracks, video and audio bars with filmstrips and waveforms, envelopes, markers, transition marks, snapping, zoom and the playhead, on a `QGraphicsView`.

### Fit with Opsis

**In place.** `apps/frontend/src/timeline/` (`duck.ts`, `envelopes.ts`, `local-frames.ts`, `playhead.ts`, `snap.ts`, each tested) and `editor/timeline/` (layout, drag, gestures, keys, zoom, clock, commands); filmstrips and waveforms computed in Rust (`opsis-media` `analyze_filmstrip`, `analyze_waveform`, `waveform_levels`); the playhead shared through the session (`share_playhead`).

**Missing.** The maths in Rust, the scene items, the clock.

### Work

1. Port `timeline/*.ts` and the pure parts of `editor/timeline/*.ts` to Rust with their property tests.
2. Items: track rows, bar items painting cached filmstrip and waveform pixmaps, envelope handles, marker and transition items, the playhead item.
3. Clock: the main window's clock ticks at the film's frame rate (`QElapsedTimer` with a timer per frame) and shares every frame through the session; other windows send play, pause, seek and rate to it, as today.
4. Scrub audio through the audio engine (next part).

### Sources

- Graphics View: https://doc.qt.io/qt-6/graphicsview.html (checked 2026-11-05)

## Mixer and audio

### What it changes

The Mixer's strips, faders, knobs, meters and EQ curves as painted widgets, and an audio engine in Rust replacing the editor's Web Audio (stem preview and scrubbing).

### Fit with Opsis

**In place.** `editor/mixer/` (strip settings, knob maths, meter levels, all tested); `editor/stem-player.ts` (an `AudioContext` per stem: media element source → `MAX_PREVIEW_FILTERS` biquad filters → stereo panner → gain, parameters smoothed over `SMOOTHING_S` = 0.02 s, drift correction over `DRIFT_FRAMES` = 2); `editor/timeline/scrub.ts`; the bundled FFmpeg decodes anything.

**Missing.** Audio output, a DSP chain, decoding stems to PCM for playback.

### Work

1. Decode: the FFmpeg sidecar writes `f32le` PCM per stem, as `opsis-media` runs it today.
2. DSP in Rust: RBJ biquads (peaking, shelves, pass filters) per `previewTotals`, constant-power pan, gain, each parameter smoothed over 0.02 s; property tests on the filter maths.
3. Output: one audio stream mixing the playing stems; the audio clock drives the playhead during playback.
4. Meters: peak and RMS per strip from the mixed buffers, read by the painted meters each frame.
5. Port `knob-math.ts`, `meter-levels.ts`, `strip-settings.ts` to Rust with their tests.

### Decisions

- **Audio output: cpal or `QAudioSink`.** cpal (0.18.2, Rust, the standard crate, 27 million downloads, mostly one active maintainer) keeps audio in Rust with the DSP; `QAudioSink` (Qt Multimedia, LGPL) keeps it in Qt. Recommended: cpal, so the audio clock lives beside the DSP.

### Sources

- cpal: https://github.com/RustAudio/cpal (0.18.2, checked 2026-11-04)
- Audio EQ cookbook (RBJ biquads): https://www.w3.org/TR/audio-eq-cookbook/ (checked 2026-11-05)

## Playback and pictures

### What it changes

The media player for exported films and sounds, image previews, the streamed views (sign-in, element picker) and the live picture, all drawn on the CPU.

### Fit with Opsis

**In place.** `opsis-footage`'s `Decoder` runs the FFmpeg sidecar and returns `RawFrame`s with timestamps; `opsis-probe` reads lengths and chapters' containers; streamed views and live frames arrive as JPEG with a header (`crates/desktop/src/frames.rs`), with input sent back through `views::view_input` and `terminal_live::terminal_input`.

**Missing.** A player widget, A/V sync, chapter display, picture widgets.

### Work

1. Player: the session decodes the film with the `Decoder` (scaled to the widget size), hands frames to C++ as RGBA, and a widget paints the newest frame whose time is at or before the audio clock (`QImage` in `paintEvent`). Seeking restarts the decoder at the time. Controls, chapters and full screen are Qt widgets.
2. Not `QVideoWidget`: it renders through a native GPU window (`QVideoWindow`), and the editor draws on the CPU everywhere.
3. Images: `QImage` from the file, full size in a dialog on click, as today.
4. Streamed views and live picture: JPEG decoded off the GUI thread into a `QImage`, the newest frame only; mouse, wheel, keys and text mapped through the letterboxing (`stream-view.tsx` → `toPage`) and sent to the session; dialogs, file choosers and tabs answered from the window as today.

### Risks

- A 1080p frame painted 30 times a second on the CPU is about 8 MB copied per frame; scale in the decoder to the widget size first.

### Sources

- `QImage`: https://doc.qt.io/qt-6/qimage.html (checked 2026-11-05)
- `QVideoWidget` (not used): https://doc.qt.io/qt-6/qvideowidget.html (checked 2026-11-05)

## Other windows

### What it changes

Settings (its pages in `app/settings/`), the start screen, setup, the fonts dialogs (`fonts/`), Live, the terminal window, Replay, the disk-change dialog, the new-project dialog.

### Work

1. Settings window with a sidebar and pages; settings written one section at a time through the session, every window following `SettingsChanged`.
2. Start screen and setup (engine install with progress).
3. Fonts: installed, online, review and search-list dialogs on item models.
4. Live window and the terminal window: picture widgets from Playback and pictures.
5. Licences and About keep listing what Opsis includes by name, now with Qt and KDDockWidgets.

## Testing

### What it changes

The editor's Vitest tests (1,672 lines) and the WebdriverIO end-to-end suite (`e2e/`, 9 spec files, `tauri-plugin-wdio-webdriver`) are replaced.

### Work

1. Rust: every function ported from TypeScript keeps its property tests (`proptest`), in the crate it moves to.
2. Views: QTest (`QTest::mouseClick`, `keyClicks`, `mouseMove`) on the `offscreen` platform plugin, no display. The tests are C++ functions in `opsis-ui` exposed through the bridge and run by `cargo test` (`QT_QPA_PLATFORM=offscreen`), so `bun run test` covers them.
3. End to end: the real app under Xvfb only, driven through the Local API. An `e2e` Cargo feature adds routes that list windows, read the widget tree (`QAccessible`), send input (QTest functions from inside the app) and grab windows (`QWidget::grab`). The runner keeps today's data folder seeding (`e2e/src/support/user-data.ts`) and close check.
4. Wayland checks: a headless Wayland compositor (weston or sway headless) runs the input-method, drag-and-drop and pop-out specs, since Xvfb covers only X11.

### Decisions

- **Runner language.** Keep the e2e runner in TypeScript on bun (it only talks to the Local API), or move it to Rust; recommended: keep it in TypeScript.

### Sources

- QTest: https://doc.qt.io/qt-6/qtest.html (checked 2026-11-05)
- Offscreen platform: https://github.com/qt/qtbase/tree/dev/src/plugins/platforms/offscreen (checked 2026-11-05)

## Packaging

### What it changes

`bun run make` builds the Qt app instead of `tauri build`, with Qt and KDDockWidgets bundled, FFmpeg still a sidecar program.

### Fit with Opsis

**In place.** `scripts/stage.ts`, the FFmpeg sidecar beside the main program, deb/rpm/AppImage, NSIS, the macOS app signed and notarized in CI; `THIRD_PARTY_NOTICES.md`; `deny.toml`.

**Missing.** A bundler for each format without Tauri; Qt deployment; resources found without Tauri's resource folder.

### Work

1. Resources: `AppPaths` finds the film page, terminal page, licences and notices beside the executable (or in the macOS bundle's `Resources`), replacing Tauri's resource folder.
2. Linux: `linuxdeploy` with its Qt plugin for the AppImage; `cargo-deb` and `cargo-generate-rpm` for deb and rpm, with Qt bundled or depended on (decided below). The deb no longer needs `gstreamer1.0-libav` or WebKitGTK.
3. Windows: `windeployqt`, then NSIS (per-user install as today).
4. macOS: `macdeployqt`, `codesign` on every Mach-O, notarization in CI as today.
5. Licences: Qt's LGPL-3.0 text and KDDockWidgets' GPL-3.0 notice in `THIRD_PARTY_NOTICES.md` and Settings → Licences; Qt libraries shipped as shared libraries so the LGPL relinking terms hold.

### Decisions

- **deb and rpm: bundle Qt or depend on the distro's Qt.** Distro Qt versions vary and Opsis needs 6.12; recommended: bundle, like the AppImage.

### Sources

- linuxdeploy Qt plugin: https://github.com/linuxdeploy/linuxdeploy-plugin-qt (checked 2026-11-05)
- Qt deployment: https://doc.qt.io/qt-6/deployment.html (checked 2026-11-05)
- Qt licensing (LGPL-3.0): https://doc.qt.io/qt-6/licensing.html (checked 2026-11-05)

## Cut-over

### What it changes

The Tauri editor and everything that exists only for it go.

### Work

1. Delete `apps/frontend`, `crates/desktop` (after the session took its logic), `crates/desktop/capabilities`, the CSP, the editor's file server (`file_server.rs`; the film's file server in `opsis-engine` stays), `page_log.rs`, `tauri.conf.json` and `tauri.release.conf.json`.
2. Drop `tauri`, `tauri-build`, `tauri-specta` and the `tauri-plugin-*` crates. Keep `specta` and `specta-typescript` only for `packages/video/src/generated/spec.ts`, and move the `=2.0.0-rc.26` pin note in `AGENTS.md` §5 (it exists for `tauri-specta`).
3. Delete `e2e/`'s WebdriverIO setup and its 9.x pin; the new runner replaces it.
4. `@opsis/video/common` helpers the editor imported (format, math, lists, guards, time; 830 lines) stay for the film; the editor's own copies in Rust replace its imports.
5. Rewrite `AGENTS.md` §1 (the editor is Qt Widgets; no "system webview"), §2 (installed-code table without shadcn, React Flow UI, TanStack Router and `bindings.ts`; the `DropdownMenuContent`/`TooltipContent` rule goes), §3, §4, §6 (crates `opsis-session` and `opsis-ui`; "only `opsis-ui` opens windows or dialogs").
6. Rewrite `agent-docs/editor.md`, `document.md`, `e2e.md`, `packaging.md`, `toolchain.md`; add `agent-docs/ui.md` for the bridge and view rules.

### Risks

- The e2e suite moves at the same time as the app it tests; keep the old suite running against the Tauri app until the new runner covers its 9 specs.

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
