# Terminal takes

Ghostty behind `opsis-vt`, terminal sessions and takes, tapes and cast files, the live tape window, take previews, live pictures from the terminal page, terminal grading and the one renderer every terminal picture uses. Rust paths are relative to `crates/terminal/src/` unless they name another crate; page paths to `packages/video/src/`. The take value shared with browser takes is in [capture.md](capture.md), footage in [footage.md](footage.md), the Zig toolchain in [toolchain.md](toolchain.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                           | Where                                                                                                                                                                                                                                                                                                             |
| --------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Zig wrapper over Ghostty          | `crates/vt/zig/src/vtwrap.zig`, `crates/vt/build.rs`                                                                                                                                                                                                                                                              |
| Rust declarations and safe handle | `crates/vt/src/ffi.rs`, `crates/vt/src/terminal.rs` → `Terminal` (`key_bytes`, `mouse_bytes`, `paste_bytes`, `focus_bytes`, `pointer_shape`)                                                                                                                                                                      |
| Nodes                             | `crates/compile/src/nodes/terminal.rs`                                                                                                                                                                                                                                                                            |
| Session value and steps           | `crates/types/src/terminal.rs` → `TerminalSessionValue`, `TerminalStep`                                                                                                                                                                                                                                           |
| Shells and their setups           | `shell.rs`, `integration/`                                                                                                                                                                                                                                                                                        |
| Pseudo-terminal session           | `session.rs` (`portable-pty`)                                                                                                                                                                                                                                                                                     |
| Driver                            | `driver.rs`; `input.rs` (`parse_chord`, `target_cell`, `find_on_screen`); `recording.rs` → `Recording` (events, actions, caret and pointer of a take)                                                                                                                                                         |
| Take files                        | `take.rs` (`take.cast`, `take.log.json`)                                                                                                                                                                                                                                                                          |
| Cast files                        | `cast/` (asciinema's module, adapted)                                                                                                                                                                                                                                                                             |
| Visible timeline                  | `timeline.rs` (from the take's `opsis:` markers)                                                                                                                                                                                                                                                                  |
| Tapes                             | `tape` module → `write`, `tape_steps`, `session_tape`; `executors.rs` → `tape_stamp`; `live.rs` → `TapeRecorder`, `PAUSE`                                                                                                                                                                                         |
| Cast import                       | `import.rs` → `import_cast`                                                                                                                                                                                                                                                                                       |
| Prepared frames                   | `prepare.rs`; `frames.rs` (the frames file, `MAGIC`)                                                                                                                                                                                                                                                              |
| Live feed                         | `live_feed.rs` → `TakeLive`; `crates/types/src/live.rs` → `LiveEvent`                                                                                                                                                                                                                                             |
| Live tape window                  | `executors.rs` → `LiveTapeHost`; `live.rs` → `LiveTape`; `crates/session/src/terminal_live.rs` → `AppLiveTapes`, `terminal_input`, `finish_terminal`; `crates/qt/cpp/terminal_window.cc` → `TerminalWindow`, `TerminalCanvas`; `crates/ui/src/terminal_window.rs` → `cell_at`, `key_input`                        |
| Paint queue                       | `crates/session/src/terminal_paint.rs` → `PaintQueue`; the live take's painter `crates/session/src/run.rs` → `TerminalLive`                                                                                                                                                                                      |
| Terminal page painter             | `crates/capture/src/terminal_paint.rs` → `TerminalPainter`; `crates/compile/src/terminal/mod.rs` → `terminal_fonts`                                                                                                                                                                                               |
| Preview spec                      | `crates/compile/src/terminal/preview.rs` → `preview_spec`, `preview_size`, `PREVIEW_FPS`; `executors.rs` → `TakePreview` (in `TerminalDeps`); `crates/render/src/preview.rs` → `TakePreviews`                                                                                                                     |
| Cells to pixels                   | `crates/compile/src/terminal/mod.rs` → `regrid_log`; `ActionLog.cells` and `ActionLog.caret_track` (`crates/types/src/actions.rs`), followed in `crates/compile/src/recording.rs` → `followed_track`                                                                                                              |
| Grading                           | `crates/media/src/grade.rs` → `COLOUR_STEPS`, `colour_steps`; `crates/render/src/footage.rs` → `terminal_of`; `crates/compile/src/nodes/grade.rs` → `page_effects`; `frames.rs` → `Frames::colours`, `Frames::recolour`; `colours.rs` → `swatch`, `read_swatch`; `crates/footage/src/terminal.rs` → `swatch_args` |
| Drawing on the film               | `core/terminal-screen.tsx` → `TerminalScreen`; `terminal/view.tsx` → `TerminalView`; `terminal/sprites.ts`; `terminal/effects.tsx`                                                                                                                                                                                |
| Sprite check                      | `crates/render/tests/sprites.rs` (`DRAWN`, `ATLASES`); `terminal/sprites.test.ts` (`RANGES`)                                                                                                                                                                                                                      |
| Spot pictures' proportions        | `crates/compile/src/inspector.rs` → `SpotBackground`                                                                                                                                                                                                                                                              |
| Themes                            | `crates/compile/src/terminal/themes.generated.json` (generated, [AGENTS.md](../AGENTS.md))                                                                                                                                                                                                                       |

## Rules

### Ghostty

- **Ghostty is reached only through `opsis-vt`, and only `opsis-terminal` depends on it.**
- **`vtwrap.zig` imports Ghostty's `ghostty-vt` Zig module and exports a C interface shaped for Opsis**; `build.rs` links it as static archives (the wrapper and Ghostty's SIMD libraries), and Rust calls it through hand-written declarations in `src/ffi.rs`.
- **`Terminal` is one owned handle, `&mut self` everywhere, `Send` and not `Sync`, and it checks the interface's version and struct sizes when it is made.**
- **Input turns into the bytes the program expects through Ghostty's own encoders, with the modes the program set**: keys (by the editor's key names), mouse presses, moves and wheel turns at a cell, pastes and focus changes.
- **`opsis-terminal` has no `unsafe`.**

### Sessions and takes

- **Terminal nodes are virtual like browser nodes**: Open terminal starts a `terminal-session` value (shell, folder, size, variables, typing and pointer speeds) and every action node appends its `TerminalStep`; Record terminal take (`capture.terminal`) plays them in one take.
- **A Clean shell starts from Opsis's own bash, zsh or PowerShell setup in `integration/`, with Opsis's prompt and OSC 133 marks** (PowerShell on Windows, zsh on macOS, bash elsewhere); the Own choice (`ShellChoice::Own`, `shell.rs` → `own`) starts the user's `$SHELL`, or PowerShell on Windows, with their own setup.
- **A reader thread feeds Ghostty and writes its replies straight back to the program** (`session.rs`).
- **A take is a `.cast` v3 file (written to the millisecond) and a log in cells**; cast files v1, v2 and v3 are read.
- **Clip from take turns a terminal take into `ScreenProps::Terminal` and scales the log into pixels of its Terminal look's cell grid** (`regrid_log`), so a later Terminal look moves every mark with its cells.
- **Follow cursor follows the text cursor (`caret_track`) when the take has no pointer.**
- **Terminal look is `only_for` terminal takes.**

### Tapes

- **Play tape (`terminal.playTape`) plays a VHS tape file.** The `tape` module lexes and parses VHS's grammar (strings in three quotes, bare words as text, times in seconds unless a unit says otherwise), inlining `Source` from the tape's folder and stamping it into the node's key (`tape_stamp`).
  - A sourced tape must be inside the project folder, may not source another tape, and its `Output` lines are dropped (`parse_with_sources`).
- **Tapes write back (`write`), and parse → write → parse is the identity.**
- **`tape_steps` maps a tape onto a session**:
  - commands become steps; a `Wait` with neither its own pattern nor a `Set WaitPattern` becomes `WaitPrompt`, which waits for the shell's prompt marks, because Opsis's prompt isn't VHS's `> `;
  - `Set Shell` picks the Own shell;
  - sizes come from VHS's defaults;
  - the look, window bar, corners and margin become Terminal look, Window frame and Shot entries on the node's style output, a JSON theme the take's own theme, `PlaybackSpeed` a Speed ramp on the take's edits;
  - the rest, `Output` and unknown `Set` names among it, is listed as not used (`Measured::Tape`).
- **With Record it live when the file is missing on, the executor asks `LiveTapeHost` for a tape when the file is missing or the node was asked to run (`ctx.forced`, the node a run targets and those after it)**: a clean 80 × 24 shell (`LIVE_COLS`, `LIVE_ROWS`) in the default theme, in a native window of its own (`TerminalWindow`, opened through `Platform::open_terminal_window`). Without a live tape host it fails with "The tape file isn't in the project folder."
  - The window's `TerminalCanvas` shows pictures the terminal page paints, sent with the size, cell grid and padding they were painted at, so Rust maps the pointer to a cell (`cell_at`) and the editor's keys to terminal input (`key_input`); keys, text, pastes, pointer, wheel and focus go back as `TerminalInput` (`terminal_input`), which `LiveTape` turns into the program's bytes.
  - `TapeRecorder` records typing as `Type`, keys as key commands with counts, Ctrl, Alt and Shift chords as `Ctrl`, `Alt` and `Shift`, pastes as `Copy` and `Paste`, gaps of at least `PAUSE` as `Sleep`, and starts the tape with its `Set Columns` and `Set Rows`; Save tape (`finish_terminal`) saves it.
- **Export tape (`output.tape`) writes a session's steps as a tape (`session_tape`)**; actions with no tape command are left out and logged.

### Cast files

- **Cast file (`terminal.castFile`) takes a recorded `.cast` as it is** (`import_cast`): it refuses one with no output or size and shortens pauses to the node's limit or else the file's own `idle_time_limit`.
  - It replays the cast through Ghostty for the caret track and titles, turns input events into Type and key actions and plain markers into chapters, and writes the node's `take.cast` and `take.log.json`, with the file's colours as the take's theme and its event count as `Measured::Cast`.

### Every terminal picture comes from the recording browser

- **The editor never draws a terminal itself**: the terminal page in the recording browser paints every picture it shows, so the editor shows exactly what the film draws.
- **Record terminal take, Play tape and Cast file each end by rendering the take's preview** (`TakePreview`): `preview_spec` (pure) shoots the take with no zoom, window, corners, shadow, motion blur or edits, so the preview runs as long as the take, at `PREVIEW_FPS` and its screen's size in the default look made even (`preview_size`), encoded as the node's `preview.mp4` (`TerminalSource.preview`, carried onto `ScreenProps::Terminal.preview`).
  - Replay plays it like a browser take's video, with the action list seeking it; the Timeline's terminal bars are filmstrips of it; spot and area pictures show it at the terminal's own proportions (`SpotBackground.aspect`), so areas are fractions of its screen.
- **Live pictures come from the terminal page (`apps/terminal`, built into `terminal-bundle/` without `public/`)**: `TerminalPainter` serves the app's public files at the root and the bundle at `/.opsis/terminal/`, opens it in the recording browser with the film's switches, registers the terminal families and paints one-frame frames files with `window.opsisTerminal.paint`, taking each as a JPEG screenshot at the screen's size.
- **The session runs one `PaintQueue` per live take (`TerminalLive`, `crates/session/src/run.rs`) and per live tape window**: the newest frame wins, and the browser opens once the engine check is done.
- **While Record terminal take or Play tape runs, `TakeLive` sends the start (its viewport in pixels), each step and the end as `LiveEvent`s, and one-frame frames files at the live frame rate**; the session paints them and sends them as ordinary live frames to the editor's picture-in-picture and Live window ([document.md](document.md)).

### Drawing a terminal

- **The film page and the terminal page draw through one renderer, `TerminalView`** (`@opsis/video/terminal/view`): text runs placed by column and held to their cells, and a glyph wider than its cell narrowed to fit once the fonts have loaded.
- **Every character Ghostty draws itself is drawn as SVG by `terminal/sprites.ts`**, ported from Ghostty's `src/font/sprite/draw` with its inner strokes, inverted shapes and clips to the cell; the ranges are `RANGES` in `terminal/sprites.test.ts`.
  - The drawn ranges (`DRAWN`) are checked against Ghostty's own reference atlases at its four test cell sizes (`crates/render/tests/sprites.rs`), glyphs spaced a cell apart since some overshoot their cell; `sprites.test.ts` holds the property tests.
  - They are in one layer, and fills of the same colour and opacity share one path, so neighbouring cells join without seams; clipped, inner-stroke and inverted shapes are drawn on their own.
- **`prepare.rs` replays the cast into Ghostty once per film frame and resolves every colour and redaction in Rust**; the page reads the frames file from the footage route (a terminal screen is a footage use with its theme, `FootageUse.terminal`, and `hosted` gives it `framesUrl`, `.opsis/footage/<id>/frames`), which `opsis-footage` prepares once per render.
- **Redact take's Text to hide (patterns, terminal takes only) and its areas, widened to whole cells, remove the text before frames are made.**

### Grading a terminal take

- **A terminal take is graded in two halves.**
  - The colour steps (`COLOUR_STEPS`, `crates/media/src/grade.rs`) become one filter graph (`grade_filters` over `colour_steps`, built by `terminal_of`), which `opsis-footage` runs once over a swatch of every colour the prepared frames use (raw RGB piped through FFmpeg) and applies as a colour map (`read_swatch`, `Frames::recolour`).
  - Texture and Lens are resolved in Rust into `TerminalEffects` (`page_effects`, set by Grade footage) and drawn by the film page over the screen (`terminal/effects.tsx`).

## Deliberate limits

- **Sharpen and Auto fix do nothing to drawn text.**
- **A live-recorded tape keeps only what a tape can say**: keys with no tape name are skipped and counted (`skipped`, shown by the window's status), and pointer, wheel and focus input reach the program but are never recorded.
