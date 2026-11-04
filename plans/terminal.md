# Terminal takes

Record real terminal sessions in Opsis the way browser takes are recorded: a program runs in a pseudo-terminal, Ghostty's terminal engine keeps the screen, the take is saved as an asciicast (`.cast` v3) file, and the film page draws the screen frame by frame as text, with its own fixed font stack and cell-exact sprites. Takes are built from nodes, played from VHS `.tape` files (recorded live when the file doesn't exist yet), or imported from `.cast` files.

A terminal take is a `recording`, like a browser take. To make that true rather than bolted on, the recording itself is rebuilt first: a take is source-independent (its log, length and edits) with the source as an enum, the window draws a pluggable screen, and Auto zoom splits into a plain applier plus style nodes. Every camera move, style and modifier then works on both kinds of take.

Status: `opsis-vt` is built (commit `74a02df1`), the take model with Clip from take (phase 2, for browser takes), the input encoders (phase 3), the `opsis-terminal` core (phase 4), the terminal nodes and compile (phase 5: the session port, action nodes, Record terminal take, Terminal look, terminal screens in Clip from take, the frames route, themes; Clip from take's speed is optional and falls back to the take's usual speed), and the film page and fonts (phase 6; sprites cover box drawing, blocks, Braille, Powerline, Ghostty's geometric shapes, sextants, smooth mosaics, eighths and octants, the rest falls to the fonts; emoji come from Google's colour bitmap build of Noto Color Emoji, since Fontsource's is OpenType-SVG, which the recording browser doesn't draw), and tapes (phase 7: the lexer, parser, writer and mapping, Play tape with its live recording window, Export tape; a `Wait` without a pattern waits for the prompt marks, `WaitPrompt`, since Opsis's prompt doesn't end in `>`; a Terminal look with no theme keeps the take's own), and Cast file (phase 8: input events become Type and key actions, plain markers chapters, the caret track and titles come from replaying the file), and the editor (phase 9: the terminal player in Replay and the Run panel, live terminal frames in the PiP and the Live window, timeline tiles, spot and area pictures, the theme picker), and grading terminal takes (phase 10: the colour map from a graded swatch, Texture and Lens drawn on the page as `TerminalEffects`). Every phase is built.

## Decisions

| Area                       | Decision                                                                                                                                                                                                                                                         |
| -------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Terminal engine            | Ghostty's libghostty-vt, Ghostty v1.4.0, consumed as a Zig dependency from Ghostty's lib-vt source tarball pinned by URL and hash                                                                                                                                |
| Language bridge            | A Zig wrapper of our own imports Ghostty's `ghostty-vt` Zig module and exports a C interface shaped for Opsis; Rust calls it through hand-written `extern "C"` declarations. No `libghostty-vt-sys`, no `libghostty-vt` crate, no bindgen, no `cc`, no submodule |
| Toolchain                  | Zig 0.17.0 (Ghostty needs the same major and minor version)                                                                                                                                                                                                      |
| Crates                     | `opsis-vt` (`crates/vt`) holds the Zig wrapper and every line of `unsafe`; `opsis-terminal` (`crates/terminal`) uses it under the workspace's `unsafe_code = "deny"`                                                                                             |
| Take model                 | One `recording` port and value for every take. The value holds what every take has (log, length, node, edits) and a `source` enum (`Browser`, `Terminal`) for what only one kind has. The log is in the take's own units (CSS pixels or cells)                   |
| Window content             | `DemoClip` draws a pluggable screen (footage or terminal) wherever it draws footage today, so every camera style, the window frame, Device frame, motion blur, annotations and the pointer work around both                                                      |
| Making a clip              | Auto zoom becomes a plain applier, Clip from take (`recording + style → clip`, Playback speed only). Its look settings move into the style chain: a new Camera style, a new Shot style, and the existing Backdrop, Window frame and Motion blur styles           |
| Terminal look              | A style node (theme, font size, line height, letter spacing, padding, cursor), so a font or theme change never records a take again and Apply style gives joined terminal takes one look                                                                         |
| Keys, mouse, paste         | Encoded by Ghostty (`encodeKey`, `encodeMouse`, `encodePaste`, `encodeFocus`) with options read from the terminal, through `opsis-vt`; no hand-written key table                                                                                                 |
| Pointer                    | Terminal takes can click, drag, move and turn the mouse wheel for programs that read the mouse; they log a pointer track and click actions like browser takes, so Cursor, Click ripple and Follow cursor work unchanged                                          |
| Pseudo-terminal            | `portable-pty` 0.9 (WezTerm's, MIT); the system ConPTY on Windows                                                                                                                                                                                                |
| Clean shell                | Linux `bash --noprofile --rcfile <file> -i`; macOS `zsh -d -i` with `ZDOTDIR`; Windows `pwsh -NoLogo -NoProfile -NoExit` (else `powershell.exe`); each with Opsis's prompt and OSC 133 marks. The user's own shell setup is a setting                            |
| Timing                     | Real only. A take is recorded once and again only when its key changes or the node is run on its own                                                                                                                                                             |
| `.cast`                    | asciinema 3.2.0's `asciicast` module (GPL-3.0-or-later), copied into `opsis-terminal` and adapted; reads v1, v2 and v3, writes v3                                                                                                                                |
| `.tape`                    | Our own parser and writer following VHS's grammar (VHS is MIT): Play tape, Export tape and live recording                                                                                                                                                        |
| Frames                     | Rust replays the cast into Ghostty and writes a frames file with every colour resolved (theme, palette changes, inverse, faint, invisible, grade) and redacted text removed. The film gets it through the footage route, the editor through a command            |
| Drawing                    | On the page, as text runs placed by column and held to their cells, with sprites as SVG for cell-filling characters. One pure renderer, used by the film and the editor. No WebAssembly anywhere                                                                 |
| Font stack                 | Fixed, under family names of its own: JetBrains Mono 2.304 (JetBrains' release, variable regular and italic) → Symbols Nerd Font Mono 3.4.0 → Noto Sans Symbols 2 → Noto Sans SC (regular and bold) → Noto Color Emoji. Bundled, hidden from the pickers         |
| Sprites                    | Box drawing, blocks, Braille, Powerline, Symbols for Legacy Computing (and its supplement), geometric shapes and branch symbols, drawn as SVG, ported from Ghostty's `src/font/sprite/draw` (MIT); they override those ranges in every font                      |
| Redaction                  | Redact take masks a terminal take's text in Rust before frames are made, by areas and by text patterns; the hidden text never reaches the page                                                                                                                   |
| Grade                      | Grade footage grades a terminal take's colours exactly (the same FFmpeg grade graph run over a swatch of every colour the take uses) and draws Texture and Lens as page effects on the terminal screen                                                           |
| Saving a take              | One Save take output saves any take's own file: a browser take's video, a terminal take's `.cast`                                                                                                                                                                |
| Not used                   | cosmic-text, ttyd, xterm.js, ghostty-web, wterm, any WebAssembly build of Ghostty, `asciicast-rs`, `betamax-core`                                                                                                                                                |
| Terminal simulator element | Unchanged                                                                                                                                                                                                                                                        |
| Licences                   | Ghostty MIT (with uucode, simdutf, Highway and Wuffs compiled in), asciinema GPL-3.0-or-later, VHS MIT, JetBrains Mono OFL-1.1, Nerd Fonts Symbols Only MIT, Noto OFL-1.1, `portable-pty` MIT; all in `THIRD_PARTY_NOTICES.md`                                   |

## Test artefacts

Outside the repository, for reference:

- `~/.cache/opsis-test/`: the prototype (Zig wrapper, Rust FFI, a tape, a sandbox with `spinner.sh` and `table.sh`, a recorded `demo.cast`, a drawing page and its rendered `demo.mp4`, and comparison stills of fonts against sprites).
- `~/.cache/opsis-research/`: the sources read: `ghostty/`, `vt-1.4.0/` (the v1.4.0 lib-vt tarball, unpacked), `vhs/`, `crates/asciinema-3.2.0/`, `crates/portable-pty-0.9.0/`, `fonts/` (JetBrains Mono 2.304, Nerd Fonts Symbols Only 3.4.0, Noto Sans Symbols 2, Noto Color Emoji, Noto Sans SC), and the prior art listed under Sources.

Measured in the prototype:

| What                                                                  | Result                                                                       |
| --------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| Preparing 277 frames at 30 fps from the `.cast`                       | 25 ms in total; 18 µs per frame call through the wrapper; 98 distinct states |
| Rendering 277 frames through the headless shell, 8 at a time          | 9.7 s                                                                        |
| Box lines with a font (CaskaydiaCove NF) at line height 1.25, 3× zoom | breaks at every row                                                          |
| Box lines with sprites, any line height, 3× zoom                      | seamless                                                                     |
| Cell width of JetBrains Mono at 20 px                                 | 12.00 px (0.6 em)                                                            |

Coverage of the fonts, from `fc-query`:

|                         | Fontsource Latin (bundled today) | JetBrains Mono 2.304 | Symbols Nerd Font Mono 3.4.0 | Noto Sans Symbols 2 |
| ----------------------- | -------------------------------- | -------------------- | ---------------------------- | ------------------- |
| Code points             | 228                              | 1,362                | 10,410                       |                     |
| `✓ ✗ → λ ●`             | no                               | yes                  | no                           |                     |
| Box `─ ╭`, blocks `█ ░` | no                               | yes                  | no                           |                     |
| Powerline               | no                               | yes                  | yes                          |                     |
| Nerd icons              | no                               | no                   | yes                          |                     |
| `✔ ⏵ ★ ⚠ ❯`, Braille    | no                               | no                   | no                           | yes                 |
| CJK, emoji              | no                               | no                   | no                           | no                  |

## The take model

This part changes browser takes too, and lands before anything terminal (see Order of work).

### What a take is

What every part of a film reads from a take is the same for both kinds: the camera planner (`camera-track.ts`, from actions with `point` and `rect`), annotations (`resolveAnnotations`, from actions of the pointer kinds with a `rect`), the pointer overlay (`cursorAt`, from `cursorTrack` and click actions), keystroke chips (`log.keys` and typed actions), action sounds and chapters (`flatten_spec`, from actions), Skip quiet parts (`idle_spans`, from actions), and Zoom here, Pan and Speed ramp (times and fractions of the frame). Only the picture in the window differs.

```rust
pub struct RecordingValue {
    pub log: Option<ActionLog>,
    pub duration_ms: f64,
    pub node_id: String,
    pub edits: Vec<StyleEntry>,
    pub source: TakeSource,
}

#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum TakeSource {
    Browser(BrowserSource),
    Terminal(TerminalSource),
}

pub struct BrowserSource { pub video: Option<String>, pub viewport: Viewport }
pub struct TerminalSource { pub cast: Option<String>, pub cols: u16, pub rows: u16, pub theme: Option<TerminalTheme> }
```

- `edits` is today's `style` field: the entries Zoom here, Pan, Speed ramp and Redact take append to one take.
- The log is in the take's own units: CSS pixels for a browser take, cells for a terminal take (`viewport` is `cols × rows`, rects and points are in cells, a point is a cell's centre). Clip from take scales a terminal log into design pixels with the take's cell metrics, so the camera, annotations and pointer always work in design pixels, and a font change never records a take again.
- `TerminalSource.theme` is the theme a `.cast` file carries, used when no Terminal look is set.
- `CompileValue::src()` names a browser take's video and a terminal take's `.cast`, so both are followed and stamped as project files.
- `InputTake` (what cards read about a connected take) gains `kind`, so card rows can say which kind they mean.

### What the page draws

```rust
pub struct RecordingProps {
    pub log: ActionLog,
    pub name: String,
    pub speed: f64,
    pub camera: CameraProps,
    pub shot: ShotProps,
    pub window: WindowProps,
    pub pointer: PointerProps,
    pub keystrokes: Option<KeystrokeLook>,
    pub annotations: Vec<AnnotationSpec>,
    pub extra_beats: Vec<Action>,
    pub screen: ScreenProps,
}

#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ScreenProps {
    Footage { src: String, chain: FootageChain, footage_url: Option<String> },
    Terminal { cast: String, cols: u16, rows: u16, look: TerminalLook, chain: FootageChain, frames_url: Option<String> },
}
```

- `camera` holds zoom on or off and `CameraTuning`; `shot` the camera style, window size, ground colour, corners, shadow, lifted colours, background blur and brightness, and motion blur with its shutter angle; `window` the window frame (style, colour, title bar font) and its bar (shown or not, the typed text, and for addresses the mode); `pointer` the cursor look, size, feel and click ripple; `screen` what is in the window.
- `DemoClip` (`packages/video/src/camera/demo-clip.tsx`) draws `<Screen>` at the five places it draws `<Footage src>` today (`WindowGroup`, `FullBleedClip`, `NarrowBleedClip` twice, `ScreenClip`). `Screen` draws the footage frame or the terminal screen. `RecordingClip` only builds the click log, layout and keystroke chips, as today, for both kinds.
- The window frame's bar shows, for a browser take, the page address (`frameAddress`, with the typed address and its mode) and, for a terminal take, the program's title at each frame (the title events the take logs) or the typed text.
- `footage_uses` gives every screen a footage use: a video with its grade and redactions, or a terminal take with its look, grade and redactions. `hosted` fills `footage_url` or `frames_url` the same way for both.

### Making a clip

| Node                                | Ports                       | Settings                                                                                                                                                                                                                              |
| ----------------------------------- | --------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Clip from take (`camera.take`)      | `recording + style? → clip` | Playback speed (default 1.25 for a browser take, 1 for a terminal take, set when the node is added and changed later by hand)                                                                                                         |
| Camera (`camera.camera`)            | `style? → style`            | Zoom to the actions, maximum zoom, hold, least zoom, first zoom at most, room around the target, target size, off centre, join actions within, stay zoomed within, hold after typing, rest at the end, longest pan, easing, move time |
| Shot (`camera.shot`)                | `style? → style`            | Camera style (Classic, Proof, Ledger, Stage), window size, ground colour, corners, shadow, lift the colours, background blur, background brightness                                                                                   |
| Window frame (`camera.windowFrame`) | `style? → style`            | Window style, frame colour, title bar, title bar text, address mode (addresses only), title bar font                                                                                                                                  |
| Backdrop, Motion blur               | `style? → style`            | As today                                                                                                                                                                                                                              |
| Terminal look (`terminal.look`)     | `style? → style`            | Theme, font size (20), line height (1.2), letter spacing, padding (24 px), cursor shape (the program's, block, bar, underline), cursor blink                                                                                          |
| Apply style                         | `clip + style → clip`       | As today                                                                                                                                                                                                                              |

- Clip from take with no style connected uses every style's defaults. Its card shows the take's length and kind, and the chain's style count.
- Auto zoom is removed. Nothing has been released, so there's no migration: `examples/` and the templates are written again with Clip from take and Camera.
- Entries that don't apply to the take's kind are no-ops (Terminal look on a browser take); Clip from take's card says so for each one connected.

### Every style and modifier on both kinds

| Node                                             | Browser take                | Terminal take                                                                                                                                                                           |
| ------------------------------------------------ | --------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Camera zoom                                      | Actions' points and rects   | The typed command's cells for Run command and Type; the output block when a command finishes (a beat, per step "Zoom to the output"); the matched text for Wait for text; mouse actions |
| Spotlight, Highlight box, Tooltip, Callout arrow | Pointer actions with a rect | The same targets: typed commands, outputs, matched text, clicks                                                                                                                         |
| Cursor, Click ripple                             | Pointer track and clicks    | Pointer track and clicks from the take's mouse steps, the shape from the program's pointer request (OSC 22) as each sample's `c`                                                        |
| Follow cursor                                    | Pointer track               | Pointer track when the take has mouse steps, else the text cursor's track, which every terminal take logs                                                                               |
| Keystroke chips, Action sounds                   | Typed and Key actions       | Typed and Key actions with `key_times`                                                                                                                                                  |
| Skip quiet parts, Speed ramp, Zoom here, Pan     | Times, fractions            | The same                                                                                                                                                                                |
| Redact take                                      | Footage pixels              | Text masked in Rust by areas (cells) and by "Text to hide" patterns before frames are made                                                                                              |
| Grade footage                                    | FFmpeg on the footage       | Colour steps exactly, through FFmpeg on a swatch of the take's colours; Texture and Lens as page effects                                                                                |
| Device frame, Crop                               | Through the screen          | Through the screen                                                                                                                                                                      |

### What only one kind has

- **A take into a file input.** A browser take converts to its video file; a terminal take has no video file. The connection is refused: each compile sends the editor, for every output, the port types its current value converts to (`EditorCompile.converts`), the canvas's `isValidConnection` checks it beside the port types, and Rust refuses the same connection in `edit_workflow` (from the editor and the Local API). A connection that becomes invalid after an upstream change gets a validation error on the edge (`take_conversion_issues` in `crates/compile/src/graph.rs`).
- **Saving a take.** Save take (`output.take`, `recording → file`, claims the file) saves a browser take's video or a terminal take's `.cast`, as recorded.
- **Settings only one kind uses** (the address mode, Redact take's "Text to hide") show by the connected take's kind: the Inspector context sends it (`InspectorContext.take_kind`), and fields carry `x-show-if-take` the way `x-show-if-file` works for files.
- **Spot and area pictures** (Zoom here, Pan, Redact take): a browser take shows a frame of its video, a terminal take a frame drawn by the terminal renderer (`SpotBackground` gains a terminal kind).

## Crates

```
opsis-terminal  ──►  opsis-vt  ──►  Ghostty libghostty-vt (Zig, static)
   │  no unsafe (workspace deny)     all unsafe, the Zig wrapper, build.rs
   ├──► portable-pty
   ├──► opsis-types, opsis-engine (temp folders, children), opsis-runner (executors)
   └──► serde, serde_json, thiserror, regex, log
opsis-footage   ──►  opsis-terminal   (terminal frames as a footage use)
opsis-desktop   ──►  opsis-terminal   (executors, the live window, terminal_frames)
```

## `opsis-vt` (`crates/vt`)

Built. What it is today:

- **Zig wrapper (`zig/`).** `build.zig.zon` pins Ghostty's v1.4.0 lib-vt tarball (`https://tip.files.ghostty.org/83fb95624b0439ee660c2679b43b38c8b63769cd/libghostty-vt-source.tar.gz`, hash `ghostty-1.4.0-5UdBCyABPwEUlNNylMEI0O0f7XW8Q6chZlOyf_Ff3Wxe`), written with `zig fetch --save=ghostty`. `build.zig` builds `libopsis_vtwrap` (static, PIC, compiler runtime and undefined-behaviour runtime bundled except on Windows, no stack protector on MSVC) and installs the static libraries Ghostty's module links (with SIMD on, `libsimdutf` and `libhighway`). Zig 0.17 unpacks packages into `zig/zig-pkg` (ignored by git); a package folder outside the project breaks Ghostty's path dependencies.
- **What the wrapper does.** One heap handle owns the Ghostty terminal, its stream, a render state, a reply buffer and an event queue. Grapheme clustering (mode 2027) is on. Ghostty's log is discarded. Effects: `write_pty` (replies), `device_attributes` (Ghostty's defaults), `enquiry` (empty), `xtversion` (`Opsis`), `size` (columns, rows and the cell size set by Rust), `semantic_prompt` (prompt, input, command, done with its exit code), `render_hold` (freezes the render state on begin, thaws on end), `title_changed`, `pwd_changed`, `bell`.
- **Exports.** `opsis_vt_abi`, `opsis_vt_sizes`, `new`, `free`, `feed`, `set_cell_size`, `resize`, `scroll`, `modes`, `pending_replies`, `take_replies`, `pending_events`, `take_events`, `title`, `pwd`, `release_hold`, `palette`, `frame`. Structs: `CellOut` (first code point, grapheme range, width, underline, tagged colours, flags), `RowOut` (`serial`, `y`, dirty, wrapped), `FrameOut` (size, cursor, held, palette changed, reverse, default colours, counts), `EventOut` (`code`, `kind`, `has_code`). The palette is 259 tagged entries (256, foreground, background, cursor), 0 where the program didn't change one.
- **`build.rs`.** Refuses any Zig but 0.17, always passes `-Dtarget` (Zig's native target compiles for the building computer's processor and defaults to the GNU ABI on Windows), builds Debug for debug and ReleaseFast for release into `OUT_DIR`, and links every archive the build installs, the wrapper first.
- **Safe API.** `Terminal` (owned handle, `&mut self`, `Send`, not `Sync`) checks the ABI version and struct sizes when made; `frame()` returns a `Frame` borrowing reused buffers; `Cell`, `Row`, `Colour`, `Flags`, `Underline`, `CellWidth`, `Cursor`, `CursorShape`, `RowId`, `Palette`, `Event`, `Modes`, `VtError`.
- **Tests.** 15 interop tests, properties over generated input: the ABI, ASCII rows, feeding in chunks against feeding at once, SGR attributes, wide characters, combining marks and ZWJ sequences, query replies, cursor walks, scrolling, prompt marks, render holds, resizes and palette changes.
- **Known.** Ghostty drops an escape sequence with more than 24 parameters. CI runs the tests in debug and release on Linux, macOS and Windows; Windows debug linking is the one not yet seen.

Built in phase 3:

- **Input encoders.** Exports for Ghostty's `encodeKey` (options from the terminal: cursor and keypad modes, kitty keyboard flags, modifyOtherKeys, Alt as escape), `encodeMouse` (the program's mouse modes and format, cell size from `set_cell_size`), `encodePaste` (bracketed paste and its safety check) and `encodeFocus`, and the pointer shape the program asked for (`terminal.mouse_shape`). Safe methods `key_bytes`, `mouse_bytes`, `paste_bytes`, `focus_bytes`, `pointer_shape`. Tests: generated chords against Ghostty's own encoder through the safe API, in every mode a program can set.

## `opsis-terminal` (`crates/terminal`)

No `unsafe`. One error enum per module (`thiserror`), numbers through `opsis_types::numbers`, times as `Duration` inside and milliseconds (`f64`) where they leave the crate, the take's clock from `Instant`.

### Modules

```
crates/terminal/src/
  lib.rs          re-exports, terminal_executors()
  shell.rs        clean and own shells per OS, finding programs on PATH
  integration/    bash.sh, zshrc, profile.ps1 (include_str!)
  session.rs      the pseudo-terminal session
  input.rs        chords to Ghostty keys, mouse targets to cells
  driver.rs       plays TerminalSteps, waits, logs
  recording.rs    the cast events and raw actions a take collects
  timeline.rs     the visible timeline from the take's markers
  take.rs         records a take: driver, cast, cell log
  cast/           asciinema's asciicast module, adapted
  tape/           the .tape lexer, parser, model, writer, diagnostics
  steps.rs        tape <-> TerminalStep
  frames.rs       the frames file: writer and reader
  prepare.rs      cast -> frames: replay, resolve, mask
  grade.rs        a take's colours through a grade
  live.rs         the live terminal window's session
  executors.rs    the node executors
```

### Shells

- Clean: Linux `bash --noprofile --rcfile <temp>/opsis-bashrc -i` (no `--norc`, which cancels the rc file); macOS `zsh -d -i` with `ZDOTDIR` at a temp folder holding `.zshrc` (`-f` would skip it); Windows `pwsh` (else `powershell.exe`) `-NoLogo -NoProfile -NoExit -Command ". '<temp>/opsis-profile.ps1'"`.
- The prompt is `opsis:<folder>$ `, green and blue, the folder shortened to `~`. Every clean shell emits OSC 133 `D;<status>` and `A` before the prompt, `B` after it, and `C` before a command's output (bash `PS0` holds no `\[ \]`; PowerShell reports no `C`). History isn't saved; zsh's `PROMPT_SP` is off.
- Own: the user's shell (`$SHELL`, else `sh`; PowerShell on Windows) with its own setup. Run command waits on prompt marks if the shell sends them, else on quiet output.
- Environment: the app's, plus `TERM=xterm-256color`, `COLORTERM=truecolor`, `LANG` and `LC_ALL` (`C.UTF-8` on Linux, `en_US.UTF-8` on macOS), `TMPDIR`/`TEMP`/`TMP` at the run's child temp folder, the session's variables and every Env step (a running shell can't take new variables, so Env applies at the start). Require steps are checked before the shell starts ("<program> isn't installed").
- The working folder is the session's Folder, resolved against the project folder.

### Session

- `portable_pty::native_pty_system().openpty`, the shell spawned on the slave, the slave dropped, the child remembered (`remember_child`) and forgotten when it ends.
- A reader thread feeds each chunk to a shared `opsis_vt::Terminal`, writes the replies back at once (programs wait for them), takes the events and sends `(time, bytes, events)` to the driver over a channel.
- Ending writes `exit\r`, waits up to 2 s, then kills the child, drops the master, joins the reader and keeps the chunks still queued.

### Input

- Chords use the editor's names (`Control+Shift+K`, `ArrowUp`, `Enter`, single characters, the same split as `chordKeys`) and map to Ghostty's keys and modifiers; Ghostty encodes them with the terminal's current modes. Meta is refused ("A terminal can't receive the Meta key").
- Mouse targets are a cell (`column, row`) or text on the screen (the first match, its middle cell); clicks, drags and wheel turns go through Ghostty's mouse encoder when the program has turned mouse reporting on, and are skipped with a run log line when it hasn't.
- Paste goes through Ghostty's paste encoder.

### Driver

| Step            | Behaviour                                                                                                                                                                        |
| --------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Run command     | Types the command at the typing speed, presses Enter, waits for the done mark (or 300 ms of quiet without marks), up to its limit ("“<command>” didn't finish within <limit> s") |
| Type            | Types the text, one character per interval                                                                                                                                       |
| Press keys      | Presses the chord, Repeat times, one interval apart                                                                                                                              |
| Click           | Moves the pointer to the target at the pointer speed, presses and releases (left, right or middle, single or double)                                                             |
| Drag            | Presses on one target, moves to another, releases                                                                                                                                |
| Mouse wheel     | Turns the wheel over a target by a number of notches                                                                                                                             |
| Wait for text   | Waits until the cursor's line or the screen matches a pattern, up to its limit ("“<pattern>” didn't appear within <limit> s")                                                    |
| Sleep           | Waits                                                                                                                                                                            |
| Start recording | Marks where the film starts                                                                                                                                                      |
| Hide, Show      | Off-camera sections, any number                                                                                                                                                  |
| Chapter         | Marks a chapter                                                                                                                                                                  |
| Clear screen    | Sends Ctrl+L                                                                                                                                                                     |
| Scroll          | Scrolls the view by lines (the scrollback, not the program)                                                                                                                      |
| Copy, Paste     | A clipboard of the take's own                                                                                                                                                    |
| Env, Require    | Applied and checked before the shell starts                                                                                                                                      |
| Screenshot      | A marker                                                                                                                                                                         |

- Before the first step the driver waits for the input mark (`B`), or 300 ms of quiet without marks, up to 5 s. After every step but Sleep, the step's Wait after. After the last, 300 ms of quiet, up to 5 s.
- Every wait checks the run's cancellation.

### Take

Writes, in the node's folder:

- `take.cast`: v3, header with columns, rows, `term.type`, time, the shell, and the theme when the session names one. Events: `o` output (decoded with a carry across reads), `i` every byte written, `r` resizes, `x` the exit, and `m` markers: `opsis:start`, `opsis:hide`, `opsis:show`, `opsis:end` (where the visible take ends, before `exit` shows), `opsis:step:<node id>`, `opsis:chapter:<title>`, `opsis:scroll:<lines>` (scrolling makes no output, so the preparer repeats it from the marker), `opsis:screenshot`.
- `take.log.json`: the `ActionLog` in cells, on the visible timeline:
  - Run command and Type: `Type` with `text`, `key_times`, `rect` and `point` over the typed text's cells; Run command, when its command finishes with output, also a `Focus` beat over the output block (unless the step's "Zoom to the output" is off);
  - Press keys: `Key` with the chord;
  - Click, Drag, Mouse wheel: `Click`, `Drag` (with `path` and `drop`), `Scroll`, with points and rects in cells;
  - Wait for text: `Focus` over the matched text;
  - Chapter: `Chapter` with its title;
  - `cursor_track`: the pointer's samples when the take has mouse steps, with the pointer shape as `c`;
  - `caret_track` (new on `ActionLog`): the text cursor's cell over time;
  - `titles` (new on `ActionLog`): the program's title over time, for the window frame's bar.
- The visible timeline starts at Start recording (or the first step), skips Hide sections and ends at `opsis:end`. Action times in hidden sections are left out.

### `.cast`

- asciinema 3.2.0's `src/asciicast.rs`, `src/asciicast/{util,v1,v2,v3}.rs`, the time quantizer and UTF-8 decoder from `src/util.rs`, and its sample casts for its tests.
- Adapted: `CastError` (`Empty`, `Version`, `Header`, `Event { line, message }`, `Io`) instead of `anyhow`; `CastTheme` and `CastColour` instead of `TtyTheme` and `rgb`; no v2 writer; no `unsafe` (the UTF-8 decoder uses checked conversion); no `as`; no comments; its tests kept.
- The v3 writer rounds to milliseconds and carries the error forward, so times read back within half a millisecond.

### `.tape`

- Lexer and parser following VHS (`lexer/`, `parser/`, `token/`): strings in three quote styles, durations, repeat counts, `Type@<speed>`, modifiers, `Wait[+Screen][+Line][@<timeout>] /regex/`, comments.
- Model `Tape { settings, steps, ignored }`; diagnostics name the line, column and what was expected.
- Writer: session steps back to tape text, and live recordings to tape text.
- Mapping (`steps.rs`): `Type` → Type; key commands with counts → Press keys; `Sleep`; `Wait` → Wait for text; `Hide`, `Show`; `ScrollUp`, `ScrollDown` → Scroll; `Copy`, `Paste`; `Env`; `Source` → the other tape's steps inline; `Require`; `Screenshot`; `Set Shell` → My shell setup; `Set Columns`, `Set Rows`, `Set Width`, `Set Height` → the size; `Set TypingSpeed`; `Set Theme`, `Set FontSize`, `Set LineHeight`, `Set LetterSpacing`, `Set Padding`, `Set CursorBlink` → the take's Terminal look (Play tape outputs it as the take's look, see Nodes); `Set WindowBar`, `Set BorderRadius`, `Set Margin` → Window frame and Shot entries on the take's edits; `Set PlaybackSpeed` → a Speed ramp entry; `Set FontFamily`, `Set Framerate`, `Output`, `Set LoopOffset` → ignored and listed on the card.

### Frames

- **Preparing (`prepare.rs`).** Input: a cast, the film's frame rate, the take's Terminal look, its grade entries and its redaction entries. The cast replays into a fresh `opsis_vt::Terminal` (resizes applied, scroll markers repeated, replies dropped). For each film frame `k`, everything up to the visible time `k / fps` is fed and the frame read. A render hold keeps the frame it froze and is released after 1 s of take time. Frame count: `ceil(visible length × fps) + 1`.
- **Colours.** Each cell's colours are resolved in Rust: the theme (or the cast's), the program's palette changes from the frame they happen, inverse, faint and invisible, and the grade's colour map (`grade.rs`). Runs carry plain RGB.
- **Masking.** Redaction areas (fractions of the screen, widened to whole cells) and "Text to hide" patterns (matched per row, across a soft-wrapped line), during each redaction's From and Until, remove the covered text and mark the run with the redaction's look (solid colour, blur, large pixels). The page draws the look over empty cells.
- **Format (`frames.rs`, little-endian).** Header: magic `OPSTERM2`, columns, rows, fps, frame count, row count, text length; one UTF-8 text block; the row table, every distinct row once as runs `[col: u16, cells: u16, text offset: u32, text length: u16, fg: u32, bg: u32, underline colour: u32, flags: u16, underline: u8, redaction: u8]`; per frame its columns and rows (a take can resize), its row indices, the cursor (x, y, visible, shape, blink, colour) and the screen's background.
- Runs merge neighbouring cells of one style when every cell is narrow printable ASCII; every other character is a run of its own, held to its cells. Undecorated trailing spaces are dropped. Rows Ghostty marks clean with the same id as the last frame reuse their index.
- **Where frames come from.** The film: a terminal screen is a footage use (`FootageUse` gains a terminal kind: the cast, the look, the grade and the redactions in its id), and the footage server answers `.opsis/footage/<id>/frames` by preparing it once per render (`opsis-footage` calls `opsis_terminal::prepare`). The editor: the `terminal_frames(project_dir, cast, fps, theme)` command returns a frames file for the Run panel, Replay, spot pictures and timeline thumbnails, unmasked, as the browser take's video is.

### Grade (`grade.rs`)

- The colour steps (Colour and light, Look file, Black and white and tint) run as the same FFmpeg graph footage gets (`grade_filters`), over a swatch image of every colour the take uses (the theme, palette changes and RGB cells, one pixel each); the graded swatch is the colour map. A terminal take and a browser take in one clip get the same look.
- Texture and Lens are drawn on the page over the terminal screen: dark edges as a vignette, grain as seeded noise per frame, soften as a blur, colour fringing as offset channels, bulge as a displacement, motion trails as the previous frames faded. Sharpen and Auto fix correct camera pixels; drawn text has nothing for them to correct.

### Executors

| Node type           | Executor                                                                                                                                                                                                |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `capture.terminal`  | Records the session's steps, writes `take.cast` and `take.log.json`, returns a terminal take                                                                                                            |
| `terminal.playTape` | Parses the tape (with `Source`), records it like Record terminal take; stamped with the tape and every sourced file; opens the live window first when the file is missing or the node is run on its own |
| `terminal.castFile` | Reads and checks the cast, writes `take.log.json` from its `i` and `m` events, returns the take                                                                                                         |
| `output.tape`       | Writes the session's steps as a tape (claims the file)                                                                                                                                                  |
| `output.take`       | Copies the take's video or cast to the chosen file (claims the file)                                                                                                                                    |

Registered in `crates/desktop/src/run.rs` (`terminal_executors`) and listed in `crates/render/tests/executors.rs`. Sessions run in the run's child temp folder.

### Live

- During Record terminal take and Play tape, the session's frames go to watching windows as a live message of their own (`LiveMessage::Terminal`, rows in the frames format, at most the live frame rate), beside the existing `LiveEvent`s.
- Play tape's live recording window (`open_terminal_window`, route `#/terminal/<id>`) draws the same rows, sends keys, pasted text and mouse input back (`terminal_input`), and has Save and Cancel; Save writes the tape (typed text as `Type`, keys as key commands, pauses over 500 ms as `Sleep`).

## Types and compile

- `PortType::TerminalSession`, no conversions into or out of it.
- `TerminalSessionValue { shell: ShellChoice, folder, cols, rows, env: Vec<EnvVar>, typing_ms, pointer_ms, steps: Vec<TerminalStep> }`; `TerminalStep { id, node_id, label, action: TerminalAction, wait_after_ms }`; `TerminalAction`: `Run { command, wait, limit_ms, zoom_output }`, `Type { text, speed_ms }`, `Keys { keys, repeat }`, `Click { target, button, double }`, `Drag { from, to }`, `Wheel { target, notches }`, `WaitText { pattern, scope, limit_ms }`, `Sleep`, `StartRecording`, `Hide`, `Show`, `Chapter { title }`, `Clear`, `Scroll { lines }`, `Copy { text }`, `Paste`, `Env { name, value }`, `Require { program }`, `Screenshot`. `CellTarget` is `Cell { col, row }` or `Text { text }`.
- `TerminalLook { theme, font_size, line_height, letter_spacing, padding, cursor_shape, cursor_blink }`; `TerminalTheme { name, fg, bg, cursor, palette: [String; 16] }`.
- `ActionLog` gains `caret_track` and `titles`.
- Everything exported with specta (`bun run types:export`).

### Nodes (`crates/compile/src/nodes/terminal.rs`)

Category `Terminal` (label "Terminal"). Every node has a description, a card and a look (checked by `crates/compile/tests/unit/looks.rs`; `bun run glyphs:bundle` after).

1. **Open terminal** (`terminal.open`): → session. Shell (Clean, My shell setup), Folder, Columns (80, 20–300), Rows (24, 5–100), Typing speed (50 ms), Pointer speed (advanced), Environment variables.
2. **Run command** (`terminal.run`): session → session. Command (variables), Wait for (Prompt, Quiet output), Time limit (60 s, advanced), Zoom to the output (on), Wait after (0.5 s).
3. **Type** (`terminal.type`): Text, Typing speed (empty = the session's), Wait after.
4. **Press keys** (`terminal.keys`): Keys (`keys` widget), Repeat, Wait after.
5. **Click** (`terminal.click`): Target (a cell or text on the screen), Button, Double-click, Wait after.
6. **Drag** (`terminal.drag`): From, To, Wait after.
7. **Mouse wheel** (`terminal.wheel`): Target, Notches (negative scrolls up), Wait after.
8. **Wait for text** (`terminal.waitText`): Pattern, Look in (Line, Screen), Time limit.
9. **Start recording** (`terminal.startRecording`).
10. **Chapter** (`terminal.chapter`): Title.
11. **Clear screen** (`terminal.clear`).
12. **Record terminal take** (`capture.terminal`): session → `recording`, executes. Name, Pause before the first step, Keep recording after the last step. Card: length, steps, size.
13. **Play tape** (`terminal.playTape`): → `recording` and `style` (the tape's look and settings as style entries), executes. Tape file (`asset`, `.tape`), Record it live when missing (on). Card: length, steps, ignored commands.
14. **Cast file** (`terminal.castFile`): → `recording`, executes. Cast file (`asset`, `.cast`), Shorten pauses over (empty = the file's own limit). Card: length, size, events.
15. **Export tape** (`output.tape`): session → file, executes, claims the file. File name.
16. **Terminal look** (`terminal.look`): `style? → style`. See Making a clip.

Every action node appends its step in `compile()` and passes its input through when skipped. Copy never names asciinema or VHS ("cast files", "tape files"), checked by `crates/compile/tests/unit/nodes_c.rs`.

### Compile

- Clip from take turns a terminal take into `ScreenProps::Terminal` with the resolved look (Terminal look entries, else the cast's theme, else the default), scales the log with the look's cell metrics (`cellWidth = fontSize × 0.6 + letterSpacing`, `cellHeight = round(fontSize × lineHeight)`, plus padding), and adds the terminal font stack to the film's fonts (`used_font_names`).
- `footage_uses` adds a terminal use for every terminal screen; `hosted` fills `frames_url`.
- `flatten_spec` treats both kinds alike (marks, hits, chapters). Timeline bars of terminal takes get `BarThumbnail::Terminal { cast, theme, speed }`, drawn by the editor.
- Terminal themes: VHS's `themes.json` (348 themes, MIT), copied by `bun run terminal:themes` (pinned to a VHS commit) into `crates/compile/src/terminal/themes.generated.json` (a generated file), read by `terminal_themes()` and sent in the catalogue. The default is VHS's default. A tape's `Set Theme "<name>"` looks the name up exactly, a JSON theme is used as it is, an unknown name falls back to the default with a warning on the card.

## Film page (`packages/video`)

- `packages/video/src/terminal/` (exported as `@opsis/video/terminal`, plain code with no runtime hooks):
  - `frames.ts`: decodes a frames file into rows and per-frame indices;
  - `sprites.ts`: `spriteFor(codePoint, cellWidth, cellHeight, weight)`, SVG for box drawing `U+2500–257F`, blocks `U+2580–259F`, Braille `U+2800–28FF`, Powerline `U+E0B0–E0D4`, Legacy Computing `U+1FB00–1FBFF` and `U+1CC00–1CEBF`, geometric shapes `U+25A0–25FF`, branch drawing `U+F5D0–F60D`, with Ghostty's geometry (`src/font/sprite/draw/*.zig`) and stroke widths from the cell the way Ghostty's `Metrics` derives them;
  - `terminal-view.tsx`: `TerminalView({ frames, index, look })`, a box of `cols × cellWidth` by `rows × cellHeight` plus padding: one absolutely placed element per run (text spans held to their cells, sprites as SVG), redaction looks over masked runs, the cursor in its shape, the background colour; `font-family` is the fixed stack.
- `packages/video/src/core/terminal-screen.tsx`: the film's `TerminalScreen`, loading the frames through `useHeldLoad`, picking the frame from `useFrame()` and the clip's speed, blinking the cursor by the frame, and drawing Texture and Lens effects for a graded take.
- `DemoClip`'s `Screen` chooses `FootageFrame` or `TerminalScreen`.
- No animation outside `useFrame()`, no colour literals beyond the themes (`colors.test.ts` gains none).

## Fonts

- `scripts/fonts-bundle.ts` gains the terminal stack, downloaded and checked by sha256 (pinned in the script, cached in `~/.cache/opsis-fonts`), written to `public/fonts/bundled/terminal/` under family names of their own, because "JetBrains Mono" is the code-font role name (`DEFAULT_CODE_FONT`, mapped by `family_for`) and already a bundled Fontsource family: `Opsis Terminal` (JetBrains Mono 2.304's variable regular and italic, `OFL.txt` kept), `Opsis Terminal Symbols` (Symbols Nerd Font Mono 3.4.0, `LICENSE` kept), `Opsis Terminal Symbols 2` (Noto Sans Symbols 2), `Opsis Terminal CJK` (Noto Sans SC 400 and 700), `Opsis Terminal Emoji` (Noto Color Emoji, Google's colour bitmap build, pinned like JetBrains Mono, with Fontsource's emoji ranges from U+2000 up). Symbols 2 and CJK come from `@fontsource/noto-sans-symbols-2` and `@fontsource/noto-sans-sc`, and the emoji ranges from `@fontsource/noto-color-emoji` (`bun add -d`).
- `bundled.generated.json` lists them, marked hidden from the font pickers; the film's `FontProvider` registers them, and the editor registers them for its terminal views.
- `public/fonts/bundled/terminal/**` joins the generated-files table.

## Editor

- **Terminal player** (`apps/frontend/src/editor/terminal-player.tsx`): `TerminalView` over `terminal_frames`, its own clock, the media player's controls. Used by the Run panel for terminal takes and by Replay, with Replay's action list in step.
- **Live PiP and Live window** draw `LiveMessage::Terminal` rows with `TerminalView`.
- **Play tape's live window** as above; its bounds saved like the other pop-outs; it closes with the main window.
- **Timeline**: terminal bars draw thumbnails with `TerminalView`.
- **Inspector**: spot and area pictures of terminal takes drawn with `TerminalView`; the theme field lists the themes with their colours; fields with `x-show-if-take`.
- **Canvas**: `isValidConnection` checks `EditorCompile.converts` as well as the port types.

## Repository changes

Done with `opsis-vt`: the workspace member, `.gitignore` for `zig-pkg`, `.zig-cache` and `zig-out`; `scripts/comments.ts` checks `.zig` and `.zon`; `lint:zig` (part of `lint` and `lint:fix`) and `format:zig` (part of `format` and `format:fix`); CI installs Zig 0.17.0 with `mlugg/setup-zig` and runs `zig build --fetch` in every job, and the build matrix runs the engine tests in debug and release; `AGENTS.md` (Zig, the Ghostty pin, the crate, the `unsafe` exception); Ghostty's notice.

Still to do, with the phases they belong to:

- `crates/terminal` in the workspace, `opsis-vt` and `portable-pty` in the root table.
- `AGENTS.md`: the take model and Clip from take (replacing Auto zoom wherever it's named), `opsis-terminal`, the terminal fonts and `themes.generated.json` in the generated-files table, and a "Terminal takes" paragraph (Ghostty only through `opsis-vt`, takes are `.cast` v3, logs in cells, frames resolved and masked in Rust, the fixed font stack, sprites over cell-filling ranges).
- `THIRD_PARTY_NOTICES.md`: asciinema (the cast module and its sample casts), VHS (the tape grammar and `themes.json`), the fonts, `portable-pty`.
- `examples/` and templates with Clip from take, plus one example with a terminal take (`bun run examples`).

## Tests

- `opsis-vt`: the interop properties, and the input encoders against Ghostty's own.
- `opsis-terminal`:
  - `.cast`: asciinema's own tests; write → read of generated events (times within half a millisecond);
  - `.tape`: VHS's examples parse; parse → write → parse is the identity; diagnostics name the right line; tape → steps → tape for the commands that map both ways;
  - visible timeline: spans ordered and summing to the length, raw times never going back, a visible time mapping back to itself, hidden times having none;
  - prepare: a fixed take (recorded once with the recorder, with a spinner, progress bar, box table, Powerline, emoji and CJK, a hidden section, a chapter and synchronized output) against `fixtures/terminal/` (a readable summary through `opsis-golden` and the frames file byte for byte); the frame count; hidden sections taking no frames; a render hold never showing a half-drawn frame; the frames file reading back as written; masking never leaving masked text in any frame; resolved colours following palette changes;
  - session and take (need a shell, ignored): prompt marks arrive; Run command waits for the command; replies reach the program (`tput cols`); a mouse click reaches a program that reads the mouse.
- `opsis-compile`: Clip from take, Camera and Shot giving the same props Auto zoom gave for the same settings; both kinds through Clip from take; the log scaled by the look; the terminal footage use; refused conversions; `x-show-if-take` values; the nodes' copy.
- `packages/video`: frames decoding against the Rust golden file; sprite properties (inside the cell; lines meeting an edge at its centre with the stroke width); `TerminalView` structure (happy-dom); `Screen` choosing the right content.
- Render (needs the dev engine and the film page, ignored): a terminal take renders through the frame driver; stills against `fixtures/terminal/stills`.
- End to end: Open terminal → Run command (`echo hello`) → Record terminal take → Clip from take → Export video, and the export exists.

## Order of work

1. **`opsis-vt`.** Done.
2. **The take model and Clip from take** (browser only, everything green). Done: `RecordingValue` with `TakeSource`, `RecordingProps` with its groups and `ScreenProps`, `Screen` in `DemoClip`, Auto zoom split into Clip from take, Camera and Shot, Window frame's bar, Save take, refused conversions (`EditorCompile.converts`, the canvas check, `edit_workflow`, the validation error), `x-show-if-take`, `InputTake.kind`, `types:export`, `examples/` and templates, golden files recorded again.
3. **`opsis-vt` input encoders.** Done
4. **`opsis-terminal` core** (done): shells, session, input, driver (mouse steps, targets, caret and title tracks), take, `.cast`, visible timeline, frames, prepare with resolved colours and masking, with their tests.
5. **Terminal nodes and compile** (done): the session port, the action nodes, Record terminal take, Terminal look, the terminal footage use and its server route, `terminal_frames`, the fonts in the film's list, themes, executors.
6. **Film page and fonts** (done): the font bundle, `@opsis/video/terminal` (frames, sprites, `TerminalView`), `TerminalScreen`, `Screen`; the render test and the end-to-end test.
7. **Tapes**: lexer, parser, writer, mapping; Play tape and its live window; Export tape.
8. **Cast file.**
9. **Editor**: terminal player, Replay, live PiP and Live window, timeline thumbnails, spot and area pictures, the theme field.
10. **Grade on terminal takes**: the swatch colour map and the page effects.

## Sources

- Ghostty: https://github.com/ghostty-org/ghostty, the v1.4.0 lib-vt tarball above; `src/lib_vt.zig`, `src/terminal/stream_terminal.zig` (the handler and its effects), `src/terminal/render.zig`, `src/terminal/c/terminal.zig`, `src/input/key_encode.zig`, `src/input/mouse_encode.zig`, `src/input/paste.zig`, `src/build/GhosttyZig.zig`, `src/build/GhosttyLibVt.zig`, `src/build/SharedDeps.zig`, `src/font/sprite/`.
- asciinema 3.2.0 crate (`src/asciicast*`, `src/util.rs`), asciicast v3: https://docs.asciinema.org/manual/asciicast/v3/
- VHS: https://github.com/charmbracelet/vhs (`lexer/`, `parser/`, `token/`, `themes.json`, the command reference).
- portable-pty 0.9.0, CreatePseudoConsole: https://learn.microsoft.com/en-us/windows/console/createpseudoconsole
- JetBrains Mono 2.304: https://github.com/JetBrains/JetBrainsMono/releases/tag/v2.304
- Nerd Fonts Symbols Only 3.4.0: `https://deps.files.ghostty.org/NerdFontsSymbolsOnly-3.4.0.tar.gz`
- Fontsource: `@fontsource/noto-sans-symbols-2`, `@fontsource/noto-sans-sc`, `@fontsource/noto-color-emoji`.
- Prior art read: Betamax, foley, wterm, ghostty-web, ttyd, libghostty-rs.
