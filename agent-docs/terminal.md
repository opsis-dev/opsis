# Terminal takes

Ghostty behind `opsis-vt`, terminal sessions and takes, tapes and cast files, the live tape window, take previews, live pictures, terminal grading and the one drawing every terminal picture uses. Paths are relative to `crates/terminal/src/` unless they name another crate. The renderer is in [film.md](film.md). The take value shared with browser takes is in [capture.md](capture.md), footage in [footage.md](footage.md), the Zig toolchain in [toolchain.md](toolchain.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                           | Where                                                                                                                                                                                                                                                                                                                                       |
| --------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Zig wrapper over Ghostty          | `crates/vt/zig/src/vtwrap.zig`, `crates/vt/build.rs`                                                                                                                                                                                                                                                                                        |
| Rust declarations and safe handle | `crates/vt/src/ffi.rs`, `crates/vt/src/terminal.rs` → `Terminal` (`key_bytes`, `mouse_bytes`, `paste_bytes`, `focus_bytes`, `pointer_shape`)                                                                                                                                                                                                |
| Nodes                             | `crates/compile/src/nodes/terminal.rs`                                                                                                                                                                                                                                                                                                      |
| Session value and steps           | `crates/types/src/terminal.rs` → `TerminalSessionValue`, `TerminalStep`                                                                                                                                                                                                                                                                     |
| Shells and their setups           | `shell.rs`, `integration/`                                                                                                                                                                                                                                                                                                                  |
| Pseudo-terminal session           | `session.rs` (`portable-pty`)                                                                                                                                                                                                                                                                                                               |
| Driver                            | `driver.rs`; `input.rs` (`parse_chord`, `target_cell`, `find_on_screen`); `recording.rs` → `Recording` (events, actions, caret and pointer of a take)                                                                                                                                                                                       |
| Take files                        | `take.rs` (`take.cast`, `take.log.json`)                                                                                                                                                                                                                                                                                                    |
| Cast files                        | `cast/` (asciinema's module, adapted)                                                                                                                                                                                                                                                                                                       |
| Visible timeline                  | `timeline.rs` (from the take's `opsis:` markers)                                                                                                                                                                                                                                                                                            |
| Tapes                             | `tape` module → `write`, `tape_steps`, `session_tape`; `executors.rs` → `tape_stamp`; `live.rs` → `TapeRecorder`, `PAUSE`                                                                                                                                                                                                                   |
| Cast import                       | `import.rs` → `import_cast`                                                                                                                                                                                                                                                                                                                 |
| Prepared frames                   | `prepare.rs`; `crates/types/src/terminal_frames.rs` (the frames file, `MAGIC`)                                                                                                                                                                                                                                                              |
| Live feed                         | `live_feed.rs` → `TakeLive`; `crates/types/src/live.rs` → `LiveEvent`                                                                                                                                                                                                                                                                       |
| Live tape window                  | `executors.rs` → `LiveTapeHost`; `live.rs` → `LiveTape`; `crates/session/src/terminal_live.rs` → `AppLiveTapes`, `terminal_input`, `finish_terminal`; `crates/qt/cpp/terminal_window.cc` → `TerminalWindow`, `TerminalCanvas`; `crates/ui/src/terminal_window.rs` → `cell_at`, `key_input`                                                  |
| Paint queue                       | `crates/session/src/terminal_paint.rs` → `PaintQueue`; the live take's painter `crates/session/src/run.rs` → `TerminalLive`                                                                                                                                                                                                                 |
| Live painter                      | `crates/film/src/terminal.rs` → `TerminalPainter`; `crates/compile/src/terminal/mod.rs` → `terminal_fonts`                                                                                                                                                                                                                                  |
| Preview spec                      | `crates/compile/src/terminal/preview.rs` → `preview_spec`, `preview_size`, `PREVIEW_FPS`; `executors.rs` → `TakePreview` (in `TerminalDeps`); `crates/render/src/preview.rs` → `TakePreviews`                                                                                                                                               |
| Cells to pixels                   | `crates/compile/src/terminal/mod.rs` → `regrid_log`; `ActionLog.cells` and `ActionLog.caret_track` (`crates/types/src/actions.rs`), followed in `crates/compile/src/recording.rs` → `followed_track`                                                                                                                                        |
| Grading                           | `crates/media/src/grade.rs` → `COLOUR_STEPS`, `colour_steps`; `crates/render/src/footage.rs` → `terminal_of`; `crates/compile/src/nodes/grade.rs` → `page_effects`; `crates/types/src/terminal_frames.rs` → `Frames::colours`, `Frames::recolour`; `colours.rs` → `swatch`, `read_swatch`; `crates/footage/src/terminal.rs` → `swatch_args` |
| Drawing                           | `crates/compile/src/elements/terminal/`: `screen.rs`, `view.rs` → `terminal_view`, `sprites.rs` → `sprite_for`, `effects.rs` → `perlin_tables`; materials `crates/effects/materials/terminal-{grain,lens}.mat`                                                                                                                              |
| Sprite checks                     | `crates/render/tests/sprites.rs` (`DRAWN`, `GRIDS`, goldens `fixtures/sprites`); `crates/compile/tests/unit/terminal_sprites.rs` (`fixtures/scene/terminal-sprites.json`)                                                                                                                                                                   |
| Spot pictures' proportions        | `crates/compile/src/inspector.rs` → `SpotBackground`                                                                                                                                                                                                                                                                                        |
| Themes                            | `crates/compile/src/terminal/themes.generated.json` (generated, [AGENTS.md](../AGENTS.md))                                                                                                                                                                                                                                                  |

## Rules

### Ghostty

- **Ghostty is reached only through `opsis-vt`, and only `opsis-terminal` depends on it.**
- **`vtwrap.zig` imports Ghostty's `ghostty-vt` Zig module and exports a C interface shaped for Opsis**; `build.rs` links it as static archives (the wrapper and Ghostty's SIMD libraries), and Rust calls it through hand-written declarations in `src/ffi.rs`.
- **`Terminal` is one owned handle, `&mut self` everywhere, `Send` and not `Sync`, and it checks the interface's version and struct sizes when it is made.**
- **Input turns into the bytes the program expects through Ghostty's own encoders, with the modes the program set**: keys (by the editor's key names), mouse presses, moves and wheel turns at a cell, pastes and focus changes.
- **`opsis-terminal` has no `unsafe`.**

### Sessions and takes

- **Terminal nodes are virtual like browser nodes**: Open terminal starts a `terminal-session` value (shell, folder, size, variables, typing and pointer speeds) and every action node appends its `TerminalStep`; Record terminal take (`capture.terminal`) plays them in one take.
- **A Clean shell starts from Opsis's own bash, zsh or PowerShell setup in `integration/`, with Opsis's prompt and OSC 133 marks** (PowerShell on Windows, zsh on macOS, bash elsewhere); the Own choice (`ShellChoice::Own`, `shell.rs` → `own`) starts the user's `$SHELL` when that file exists, else `sh` from the `PATH`, or PowerShell on Windows, with their own setup.
- **A reader thread feeds Ghostty and writes its replies straight back to the program** (`session.rs`).
- **A take is a `.cast` v3 file (written to the millisecond) and a log in cells**; cast files v1, v2 and v3 are read.
- **Clip from take turns a terminal take into `ScreenProps::Terminal` and scales the log into pixels of its Terminal look's cell grid** (`regrid_log`), so a later Terminal look moves every mark with its cells.
- **A Camera that follows the cursor follows the text cursor (`caret_track`) when the take has no pointer.**
- **Terminal look is `only_for` terminal takes.**

### Tapes

- **Play tape (`terminal.playTape`) plays a VHS tape file.** The `tape` module lexes and parses VHS's grammar (strings in three quotes, bare words as text, times in seconds unless a unit says otherwise), inlining `Source` from the tape's folder and stamping it into the node's key (`tape_stamp`).
  - A sourced tape must be inside the project folder, may not source another tape, and its `Output` lines are dropped (`parse_with_sources`).
- **Tapes write back (`write`), and parse → write → parse is the identity.**
- **`tape_steps` maps a tape onto a session**:
  - a `Wait` with neither its own pattern nor a `Set WaitPattern` becomes `WaitPrompt`, because Opsis's prompt isn't VHS's `> `;
  - the look settings become entries on the node's style output and `PlaybackSpeed` a Speed ramp on the take's edits, so a tape's look is edited like any take's;
  - what has no step or entry, `Output` and unknown `Set` names among it, is listed as not used (`Measured::Tape`).
- **`WaitPrompt` waits for the shell's prompt mark once the take has seen one, and otherwise for the output to go quiet** (`wait_prompt`, `driver.rs`): Opsis's own setups send the marks, so the Own shell waits for quiet unless the user's setup sends them too.
- **With Record it live on, the executor asks `LiveTapeHost` for a tape when the file is missing or the node is forced (`ctx.forced`: one of the run's targets, or the node a run starts from and every node after it)**: a clean 80 × 24 shell (`LIVE_COLS`, `LIVE_ROWS`) in the default theme, in a native window of its own (`TerminalWindow`, opened through `Platform::open_terminal_window`). Without a live tape host it fails with "The tape file isn't in the project folder."
  - The window's `TerminalCanvas` shows pictures `TerminalPainter` paints, sent with the size, cell grid and padding they were painted at, so Rust maps the pointer to a cell (`cell_at`) and the editor's keys to terminal input (`key_input`); keys, text, pastes, pointer, wheel and focus go back as `TerminalInput` (`terminal_input`), which `LiveTape` turns into the program's bytes.
  - `TapeRecorder` (`live.rs`) writes what was typed as tape commands, a gap of at least `PAUSE` as a `Sleep`, and starts the tape with the window's size; Save tape (`finish_terminal`) saves it.
- **Export tape (`output.tape`) writes a session's steps as a tape (`session_tape`)**; actions with no tape command are left out and logged.

### Cast files

- **Cast file (`terminal.castFile`) takes a recorded `.cast` as it is** (`import_cast`): it refuses one with no output or size and shortens pauses to the node's limit or else the file's own `idle_time_limit`.
  - It replays the cast through Ghostty for the caret track and titles, turns input events into Type and key actions and plain markers into chapters, and writes the node's `take.cast` and `take.log.json`, with the file's colours as the take's theme and its event count as `Measured::Cast`.

### Every terminal picture comes from the film's drawing

- **The editor's views never draw a terminal themselves**: every picture they show is drawn by the same code as the film, so the editor shows exactly what the film draws.
- **Record terminal take, Play tape and Cast file each end by rendering the take's preview** (`TakePreview`): `preview_spec` (pure) shoots the take with no zoom, window, corners, shadow, motion blur or edits, so the preview runs as long as the take, at `PREVIEW_FPS` and its screen's size in the default look made even (`preview_size`), encoded as the node's `preview.mp4` (`TerminalSource.preview`, carried onto `ScreenProps::Terminal.preview`).
  - Replay plays it like a browser take's video, with the action list seeking it; the Timeline's terminal bars are filmstrips of it; spot and area pictures show it at the terminal's own proportions (`SpotBackground.aspect`), so areas are fractions of its screen.
- **Live pictures are drawn by `TerminalPainter` (`opsis-film`)**: it registers the terminal families from `public/`, lays out `terminal_view` for a one-frame frames file at the screen's size and paints it with Skia on a blocking thread (`crates/session/src/terminal_paint.rs`), encoded as JPEG (`JPEG_QUALITY`).
- **The session runs one `PaintQueue` per live take (`TerminalLive`, `crates/session/src/run.rs`) and per live tape window**: the newest frame wins.
- **While Record terminal take or Play tape runs, `TakeLive` sends the start (its viewport in pixels), each step and the end as `LiveEvent`s, and one-frame frames files at the live frame rate**; the session paints them and sends them as ordinary live frames to the editor's picture-in-picture and Live window ([document.md](document.md)).

### Drawing a terminal

- **Films and live pictures draw through one function, `terminal_view`** (`crates/compile/src/elements/terminal/view.rs`): cell backgrounds as boxes, text runs placed by column and held to their cells, and a glyph wider than its cell narrowed to fit.
- **Every character Ghostty draws itself is drawn as inline SVG by `sprites.rs`**, ported from Ghostty's `src/font/sprite/draw` with its inner strokes, inverted shapes and clips to the cell (`is_sprite`).
  - `crates/render/tests/sprites.rs` draws the ranges (`DRAWN`) as grids at Ghostty's four test cell sizes, glyphs spaced a cell apart since some overshoot their cell, and compares them with their recorded goldens (`fixtures/sprites`); `terminal_sprites.rs` checks the shapes themselves against `fixtures/scene/terminal-sprites.json`.
  - They are in one layer, and fills of the same colour and opacity share one path, so neighbouring cells join without seams; clipped, inner-stroke and inverted shapes are drawn on their own.
- **`prepare.rs` replays the cast into Ghostty once per film frame and resolves every colour and redaction in Rust**; the film reads the frames file through `Measure::terminal` (a terminal screen is a footage use with its theme, `FootageUse.terminal`), which `opsis-footage` prepares once per render.
- **Redact take's Text to hide (patterns, terminal takes only) and its areas, widened to whole cells, remove the text before frames are made.**

### Grading a terminal take

- **A terminal take is graded in two halves.**
  - The colour steps (`COLOUR_STEPS`, `crates/media/src/grade.rs`) become one filter graph (`grade_filters` over `colour_steps`, built by `terminal_of`), which `opsis-footage` runs once over a swatch of every colour the prepared frames use (raw RGB piped through FFmpeg) and applies as a colour map (`read_swatch`, `Frames::recolour`).
  - Texture and Lens are resolved into `TerminalEffects` (`page_effects`, set by Grade footage) and drawn over the screen as two shader passes: `terminal-grain` runs Skia's Perlin noise from the tables `perlin_tables` sends, and `terminal-lens` displaces through the bulge map, sampling the nearest pixel as Skia's displacement filter does. `crates/render/tests/terminal_effects.rs` checks both against the SVG filters they stand for.

## Deliberate limits

- **Sharpen and Auto fix do nothing to drawn text.**
- **A live-recorded tape keeps only what a tape can say**: keys with no tape name are skipped and counted (`skipped`, shown by the window's status), and pointer, wheel and focus input reach the program but are never recorded.
