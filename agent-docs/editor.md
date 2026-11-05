# The editor

The editor's windows and routes, panels, the title bar and menus, the canvas, its look and motion, how it is hosted, starting and closing, the Settings window and per-project view state. Editor paths are relative to `apps/frontend/src/`, Rust paths to `crates/desktop/src/`. How windows share the document is in [document.md](document.md), the Timeline in [timeline.md](timeline.md), the database in [app-state.md](app-state.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                        | Where                                                                                                                                                                                                        |
| ------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Root and main layout routes    | `routes/__root.tsx`, `routes/_main.tsx` (`useMenuNavigation`, `useApiHost`, `useStartup`, `ExitDialog`)                                                                                                      |
| Panels                         | `editor/panels.ts`; tabs `dock.tsx`; pop-outs `windows.rs` → `open_panel_window`                                                                                                                             |
| Settings window                | `windows.rs` → `open_settings`; pages `app/settings/`; `store/app-settings.ts` → `useAppSettings`                                                                                                            |
| Exit flow                      | `exit.rs` → `ConfirmClose`, `close_even_if_page_dies`, `close_on_signals`; editor `ExitDialog`, `useHeldDrafts`, `commitHeldDrafts`                                                                          |
| Error screen                   | `app/error-screen.tsx` → `ErrorScreen`                                                                                                                                                                       |
| View state                     | `store/view-prefs.ts` → `loadViewState`, `readPref`, `writePref`, `layoutStorage`                                                                                                                            |
| Title bar                      | `app/app-bar.tsx` → `AppBar`; `app/window-drag.ts` → `windowDragHandlers`, `DRAG_START_PX`; `WindowControls`, `WindowBorder`                                                                                 |
| Menus                          | `menu.rs` → `app_menu`, `app_menu_sections`; `native_menu.rs`; `useMenuShortcuts`; `run_role`, `send_menu_to_editor`                                                                                         |
| Canvas                         | `editor/use-flow-elements.ts` → `useFlowNodes`; `editor/card-shapes.ts`; `editor/card-size.ts` → `cardSize`, `measuredPorts`; `editor/layout.ts`, `editor/tidy.ts`; `editor/edge-lanes.ts` → `stepPositions` |
| Tokens                         | `index.css`; window background `windows.rs` → `BACKGROUND`, checked by `app/window-background.test.ts`                                                                                                       |
| Editor motion                  | `app/motion.tsx` → `MotionProvider`                                                                                                                                                                          |
| Media player and image preview | `file-player.tsx`, `image-preview.tsx`                                                                                                                                                                       |
| Boot                           | `apps/frontend/public/page-errors.js`, `app/boot.ts`, `lib/platform.ts`                                                                                                                                      |
| CSP                            | `app.security.csp` in `crates/desktop/tauri.conf.json`                                                                                                                                                       |

## Rules

### Windows and routes

- **Each window mounts only what it uses.** The root route holds only the outlet and the window border.
  - The main window's routes (start screen, setup, editor) sit under the pathless `_main.tsx`, which mounts menu shortcuts, menu navigation (menu commands reach only the main window), the Local API's open requests, the start-up choice and `ExitDialog`.
  - The editor route mounts the document and run mirrors; a popped-out panel mounts the document mirror, menu shortcuts and, for the Run panel only, the run mirror; Settings mounts only menu shortcuts; the streamed views mount nothing, so their keys reach the page they stream.
- **Hash history only.**
- **Render errors show `ErrorScreen`, the router's `defaultErrorComponent`**, so every route has its own boundary and the root, the exit dialog and the title bar stay up: the app bar, Try again, Reload window and the message under Details.

### Panels

- **The node list, Inspector and bottom panel collapse** (Ctrl/Cmd+B, Ctrl/Cmd+Alt+B, Ctrl/Cmd+J, also in the View menu and the top bar); `panels.ts` owns their handles, open state and the active tab, and layouts persist through `useDefaultLayout`.
- **A collapsed node list becomes a 44 px category rail with popovers**; its group titles stay pinned while their group scrolls (sidebar list only).
- **The bottom panel is tabs, not a split row**: Timeline, Mixer, Run (Ctrl/Cmd+1…3), Timeline by default.
- **Any tab opens in its own window** (`open_panel_window`, route `#/panel/<name>`, bounds in the app database); closing that window docks the tab back.
- **Files play in Dice UI's media player (`file-player.tsx`), never the browser's own controls**: videos and sounds in the Run panel and Replay, and sounds in the Inspector's Files list (a made sound plays below its node's settings). Images there open full size in a dialog on click.

### Title bar and menus

- **The window has no system title bar.**
  - macOS uses the overlay title bar style with a hidden title (`windows.rs`, traffic lights inside our bar at 16, 16) and the native menu in the system menu bar.
  - Windows and Linux use undecorated windows with no native menu; the window buttons are ours (`WindowControls` through `window_role`, icon following `WindowStateChanged`), and `WindowBorder` draws a 1 px edge while the window is neither maximized nor full screen.
- **Every screen renders `AppBar`, and `windowDragHandlers` makes it the drag handle**: a press on anything that isn't a control starts `startDragging()` only once the pointer has moved `DRAG_START_PX`, and `dblclick` toggles maximize, so a double-click never starts a drag. Mark an element `data-no-drag` to keep it from dragging.
  - Tauri's own drag region starts the move on the first press, and under Wayland the compositor then takes the pointer, so its double-click never arrives. Electron's `-webkit-app-region` is ignored by WebKitGTK.
- **On Windows and Linux the bar shows the in-app menu**: the full Menubar from 1280 px, one Menu button below.
- **Menus come from one list, `app_menu`**, built natively on macOS and sent to the editor for the in-app menu (`app_menu_sections`).
  - `useMenuShortcuts` handles its accelerators in the page on Windows and Linux, leaving cut, copy, paste, select all, undo and redo to a focused text field; roles run through `run_role`; navigation commands are handled once in the main window's layout route.

### Canvas

- **`settings.editor.canvasMode` is `mouse` (default: wheel zooms, drag pans) or `touchpad` (two-finger scroll pans, pinch zooms)**, chosen in the canvas toolbar and kept for every project.
- **The minimap shows or hides per project** (`minimap` view pref; toolbar, View → Minimap, Ctrl/Cmd+Shift+M).
- **The React Flow attribution is hidden (`proOptions.hideAttribution`)**; the library prints a console note asking for a Pro subscription.
- **The document owns node positions and the UI store owns selection; React Flow's own per-node state (measured size, a drag or resize in progress) lives only in `useFlowNodes`**, merged over the store's nodes, so a drag writes one position at drag stop.
- **The canvas draws only the cards and edges on screen (`onlyRenderVisibleElements`)**, so nothing reads a card from the page: its size and handles come from React Flow's measurement (`card-shapes.ts`), and before a card has been measured, from the catalogue's estimate (`cardSize`).
- **Each card's measured size and handle positions are saved per project (key `cardShapes`)** a second after the last measurement, without the nodes that are gone, and given back to React Flow on open as `measured` and `handles`, so cards off screen aren't drawn even once to be measured.
  - A card on screen is measured again by React Flow whenever its size changes; one that changed while off screen keeps its saved shape until it is shown.
- **Tidy up layout lays the top-level cards out with ELK from the handles each card shows**, from its measured or saved shape (`measuredPorts`) or worked out from the edges for a card never measured; every edge end is a port, so a numbered input past the catalogue's own is never missing.
- **ELK loads only when Tidy up runs and lays out in a Web Worker** (`editor/tidy.ts`, imported on demand; elkjs's own `elk-worker.min.js` as a file through `?url`), so its 1.6 MB stays out of the editor's start and a large graph never holds the page. The tests pass `elk.bundled.js`, which lays out without a worker, since Node has none.
- **Edges are smoothstep paths; the edges entering one column turn in their own lane where their runs overlap** (`stepPositions`, from the cards' live positions), and ELK spaces the layers to fit those lanes.

### Look and motion

- **Panels, menus and the top bar sit on `bg-sidebar`; the canvas and page backgrounds on `bg-background`.** Colours are dark zinc greys (hue 286, slight colour, never pure black or white) and change only through the tokens in `index.css`; the window background (`BACKGROUND`) matches the canvas token.
- **Editor animation uses `motion` through the small components in `app/motion.tsx`** (`MotionProvider` respects reduced motion). Motion Primitives components are installed only when they pass the type check as-is; otherwise write a small equivalent in `motion.tsx`. `motion` stays banned in `packages/video`.
- **Pressed toggles and toggle-group items use `--toggle-on` (and `--toggle-on-border` for outline groups), unpressed ones the muted text colour**, set once in `index.css` because the installed toggle's pressed colour barely shows on panels; a toggle with its own pressed colour (Mixer mute and solo) sets `data-custom-pressed`.
- **Run success is the `--success` token**: the Run panel's checks and the node card's Done border, which `flow-node.tsx` draws itself because the installed status indicator's success border is a plain Tailwind colour.
- **Vertical separators in a centred row need `data-vertical:self-center`**, since the separator stretches by default.
- **Never name a keyframe `spin`.** React Flow UI's `node-status-indicator.tsx` injects `@keyframes spin` (with a −50% translate) while a node runs, so Tailwind's `animate-spin` is remapped to `icon-spin` in `index.css`.
- **The live stream's badge is `LiveBadge` on the `--live` token.**

### Hosting

- **Tauri serves `apps/frontend/dist`; in development the Vite server on `localhost:5173`.** The CSP allows the file server's `http://127.0.0.1:*` for pictures, media, fonts and fetches ([assets.md](assets.md)).
- **`index.html` loads `public/page-errors.js` (the editor's only public file) and `src/app/boot.ts` before the app**: uncaught errors, rejections, `console.error` and `console.warn` from every window go to the app's log (`page_log`, target `page`).
- **The platform comes from the user agent (`lib/platform.ts`), never from a call made while a module loads**: a module that runs before boot can leave the window black.

### Starting and closing

- **Settings → Starting and closing picks what start shows** (the start screen, the last project, or New project), once per app launch (`useStartup`), and whether to ask before closing.
- **Closing or quitting the main window goes through `exit.rs`**: it prevents the close and asks the editor to show `ExitDialog`, in-app, never a native dialog (`ConfirmClose`).
  - With asking on: "Are you sure you want to exit?" with "Don't ask again". With it off, the dialog appears only when work is unfinished, without the checkbox.
  - Unfinished work is a run, engine setup, a font download, unsaved changes, or a value still being typed in a field (`report_drafts` from `useHeldDrafts`); it is always listed and the button becomes "Close anyway".
  - Closing first asks the editor to hand held values over (`FlushSave`, `commitHeldDrafts`), then Rust saves; closing flushes the pending save and stops the run.
- **The dialog answers `close_shown` as soon as it receives the request; if nothing does within 2 s, Rust saves and closes without asking**, so a broken window can always close.
- **Once a close is allowed, a window whose page dies or doesn't finish unloading within 3 s is destroyed (`close_even_if_page_dies`) and the quit goes on.**
- **SIGTERM, SIGINT and SIGHUP save and close like a quit** (`close_on_signals`, Unix only).

### Settings window

- **Settings is its own window** (`open_settings`, route `#/settings?page=<page>`, bounds in the app database), opened from the menu, Ctrl/Cmd+, and any Settings button; it closes with the main window. A sidebar picks the page.
- **Its app bar has no menu; menu shortcuts pressed there go to the main window** (`send_menu_to_editor`).
- **Settings are written with `patchSettings`, one section at a time, so windows never overwrite each other**, and every window follows them through `useAppSettings`, which Rust keeps current with `SettingsChanged`.
- **Saved results (clear cache) belong to the project, in the project Inspector.**

### View state

- **View state is per project, on this computer**: the active tab, panel sizes and collapsed panels, timeline ruler and snapping, card shapes, in the database's `view_state` table, one row per project and key.
- **`open_project` returns the project's view with the workflow, and `loadViewState` puts it in the editor's cache**, which `readPref` and the panel layouts (`layoutStorage`) read synchronously.
- **`writePref` updates the cache and sends `set_view_pref`, which Rust broadcasts as `ViewPrefChanged`** so every window's cache follows; panel windows read it with `view_state` when their project changes.
- **Nothing is saved while no project is open, and a project made again in the same folder starts with no saved view.**
- **The main window's size and maximized state are saved per project (key `mainWindow`, written by `windows.rs`)**: a project with none opens maximized, and the start screen leaves the window as it is (1600×1000 at launch). Pop-out window bounds stay global.
