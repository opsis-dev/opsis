# Terminal takes

Record real terminal sessions in Opsis the way browser takes are recorded: a program runs in a pseudo-terminal, Ghostty's terminal engine keeps the screen, the take is saved as an asciicast (`.cast` v3) file, and the film page draws the screen frame by frame as text, with its own fixed font stack and cell-exact sprites. Takes are built from nodes, played from VHS `.tape` files (recorded live when the file doesn't exist yet), or imported from `.cast` files; every take is a `recording`, so the camera and recording styles work on it.

Status: plan only. Nothing here is built yet. The prototype that proved the pieces lives outside the repository (see Test artefacts).

## Decisions

Everything below is decided. The sections after this one say how each is built.

| Area                       | Decision                                                                                                                                                                                                                                                              |
| -------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Terminal engine            | Ghostty's libghostty-vt, Ghostty v1.4.0, consumed as a Zig dependency from Ghostty's lib-vt source tarball pinned by URL and hash                                                                                                                                     |
| Language bridge            | A small Zig wrapper of our own that imports Ghostty's `ghostty-vt` Zig module and exports a C ABI shaped for Opsis; Rust calls it through hand-written `extern "C"` declarations. No `libghostty-vt-sys`, no `libghostty-vt` crate, no bindgen, no `cc`, no submodule |
| Toolchain                  | Zig 0.17.x (Ghostty requires the same major and minor version), pinned to 0.17.0 locally and in CI                                                                                                                                                                    |
| Crates                     | Two: `opsis-vt` (`crates/vt`) holds the Zig wrapper and every line of `unsafe`, the one crate with `#![allow(unsafe_code)]`; `opsis-terminal` (`crates/terminal`) uses it under the workspace's `unsafe_code = "deny"`                                                |
| Pseudo-terminal            | `portable-pty` 0.9 (WezTerm's, MIT); the system ConPTY on Windows                                                                                                                                                                                                     |
| Clean shell                | Linux `bash --noprofile --norc`; macOS `zsh -f`; Windows `pwsh -NoProfile` when PowerShell 7 is installed, else `powershell.exe -NoProfile`; each with Opsis's fixed prompt and OSC 133 prompt marks. The user's own shell setup is a setting                         |
| Timing                     | Real only. A take is recorded once and saved as `.cast` v3; it is recorded again only when its key changes or the node is run on its own (Run this). No "record every run" switch                                                                                     |
| `.cast`                    | asciinema's own `asciicast` module (asciinema 3.2.0, GPL-3.0-or-later), copied into `opsis-terminal` and adapted; reads v1, v2 and v3, writes v3                                                                                                                      |
| `.tape`                    | Our own parser following VHS's grammar (VHS is MIT); import (Play tape), export (Export tape) and live recording                                                                                                                                                      |
| Drawing                    | On the film page, in the existing renderer: text rows placed by column, cells held to their width, sprites as SVG for cell-filling characters. No browser during a take. No WebAssembly anywhere (the film page has none today)                                       |
| Font stack                 | Fixed, not swappable, in this order: JetBrains Mono 2.304 (JetBrains' release, variable regular and italic) → Symbols Nerd Font Mono 3.4.0 → Noto Sans Symbols 2 → Noto Sans SC (regular and bold) → Noto Color Emoji. All bundled                                    |
| Sprites                    | Box drawing, blocks, Braille, Powerline, Symbols for Legacy Computing (and its supplement), geometric shapes and branch symbols, drawn as SVG, ported from Ghostty's `src/font/sprite/draw` (MIT); they override those ranges in every font                           |
| Not used                   | cosmic-text, ttyd, xterm.js, ghostty-web, wterm, any WebAssembly build of Ghostty, Ghostty's sprite code as WebAssembly, `asciicast-rs`, `betamax-core`                                                                                                               |
| Output                     | Every take is a `recording`                                                                                                                                                                                                                                           |
| Terminal simulator element | Unchanged                                                                                                                                                                                                                                                             |
| Licences                   | Ghostty MIT, asciinema GPL-3.0-or-later, VHS MIT, JetBrains Mono OFL-1.1, Nerd Fonts Symbols Only MIT, Noto OFL-1.1, `portable-pty` MIT; all go in `THIRD_PARTY_NOTICES.md`                                                                                           |
| Nodes                      | 12 (see Nodes)                                                                                                                                                                                                                                                        |

## Test artefacts

The prototype that proved the design, all outside the repository:

- `~/.cache/opsis-test/vtwrap/`: the Zig wrapper (`build.zig`, `build.zig.zon` pinning the v1.4.0 tarball, `src/vtwrap.zig`, 150 lines). It built against Ghostty v1.4.0 with Zig 0.17.0 into `zig-out/lib/libvtwrap.a` (18 MB archive; its only undefined symbols are libc's). It ran with `-Dsimd=false`.
- `~/.cache/opsis-test/rec/`: the Rust side (`build.rs` runs `zig build` and links the archive; `src/vt.rs` is the FFI and the six interop tests; `src/main.rs` records a tape and prepares frames). Its tape parser and `.cast` writer are throwaway: the plan replaces both.
- `~/.cache/opsis-test/demo.tape` and `~/.cache/opsis-test/sandbox/` (`spinner.sh` with a Braille spinner and a block progress bar, `table.sh` with a rounded box table, colours, Powerline and Nerd icons).
- `~/.cache/opsis-test/out/demo.cast`: the recorded take, `.cast` v3, 13.2 s, 261 events, 8.7 KB.
- `~/.cache/opsis-test/page/`: the drawing prototype (`index.html` with three variants: `plain`, `nerd`, `sprites`; `frames.js` from the preparer).
- `~/.cache/opsis-test/out/demo.mp4`: the full take rendered through the recording browser (277 frames, 148 KB).
- Comparisons in `~/.cache/opsis-test/out/`: `last-plain.png`, `last-nerd.png`, `last-sprites.png` (line height 1.25), `last-nerd-lh15.png`, `last-sprites-lh15.png` (line height 1.5), `zoom-compare.png` (3× zoom, Nerd font on top, sprites below), `spin-compare.png` (Braille from a fallback font, then as sprites).
- `~/.cache/opsis-research/`: the sources read for this plan: `ghostty/` (full clone), `vt-1.4.0/` (the v1.4.0 lib-vt tarball, unpacked), `libghostty-rs/`, `betamax/`, `foley/`, `vhs/`, `vercel-labs_wterm/`, `coder_ghostty-web/`, `anomalyco_ghostty-web/`, `tsl0922_ttyd/`, `Uzaaft_awesome-libghostty/`, `cosmic-text/`, `cosmic-bench/`, `crates/asciinema-3.2.0/`, `crates/portable-pty-0.9.0/`, `crates/asciicast-rs-0.4.2/`, `fonts/` (`jbm/` JetBrains Mono 2.304, `nf/` Nerd Fonts Symbols Only 3.4.0, `sym2/` Noto Sans Symbols 2, `noto-color-emoji/`, `noto-sans-sc/`).

Measured in the prototype:

| What                                                                                                                                 | Result                                                                       |
| ------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------- |
| Interop tests (layouts, 500 generated strings, colours and flags, wide and combined characters, query replies, cursor and scrolling) | 6 of 6 pass                                                                  |
| Preparing 277 frames at 30 fps from the `.cast`                                                                                      | 25 ms in total; 18 µs per frame call through the wrapper; 98 distinct states |
| Rendering 277 frames through the headless shell, one browser per frame, 8 at a time                                                  | 9.7 s                                                                        |
| Box lines with a font (CaskaydiaCove NF) at line height 1.25, 3× zoom                                                                | breaks at every row                                                          |
| Box lines with sprites, any line height, 3× zoom                                                                                     | seamless                                                                     |
| Cell metrics of JetBrains Mono at 20 px                                                                                              | 12.00 px wide (0.6 em)                                                       |

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

## Crates

```
opsis-terminal  ──►  opsis-vt  ──►  Ghostty libghostty-vt (Zig, static)
   │  no unsafe (workspace deny)     all unsafe (#![allow(unsafe_code)]), the Zig wrapper, build.rs
   ├──► portable-pty
   ├──► opsis-types, opsis-engine (files, hashing, temp folders), opsis-runner (executors)
   └──► serde, serde_json, thiserror, tokio
opsis-prepare  ──►  opsis-terminal   (the terminal frames preparer)
opsis-desktop  ──►  opsis-terminal   (registers the executors, the live window)
```

- `opsis-vt` depends on nothing in the workspace except `thiserror`. Nothing else in the workspace depends on it but `opsis-terminal`.
- `opsis-terminal` has no `unsafe` attribute of its own: the workspace's `unsafe_code = "deny"` (`[workspace.lints.rust]` in the root `Cargo.toml`) covers it, as it covers every crate but `opsis-vt`.
- `opsis-vt`'s `src/lib.rs` starts with `#![allow(unsafe_code)]` beside its `#![deny(unsafe_op_in_unsafe_fn)]`, the only crate that allows `unsafe`. Its `unsafe` lives in `src/ffi.rs` (the declarations) and `src/terminal.rs` (the safe type); every other module of it is safe code. `clippy::undocumented_unsafe_blocks` stays off: it asks for `// SAFETY:` comments, which the no-comments rule forbids. The invariants are carried by the types (an owned handle, `&mut self` for every call, `Send` and not `Sync`) and checked by the tests.

## `opsis-vt` (`crates/vt`)

### Files

```
crates/vt/
  Cargo.toml
  build.rs
  zig/
    build.zig
    build.zig.zon
    src/vtwrap.zig
  src/
    lib.rs        safe re-exports, VtError
    ffi.rs        extern "C" block and #[repr(C)] mirrors
    terminal.rs   Terminal: the owned handle and its safe methods
    cell.rs       Cell, Colour, Flags, Underline, CellWidth
    frame.rs      Frame, Row, Cursor, CursorShape
    event.rs      Event
  tests/
    interop.rs
```

### `zig/build.zig.zon`

- `name = .opsis_vtwrap`, `version = "0.0.0"`, `minimum_zig_version = "0.17.0"`, a `fingerprint` written by Zig on first build, `paths = .{ "build.zig", "build.zig.zon", "src" }`.
- One dependency, written with `zig fetch --save=ghostty <url>` (never by hand; a hand-copied hash failed in the prototype):
  - `url = "https://tip.files.ghostty.org/83fb95624b0439ee660c2679b43b38c8b63769cd/libghostty-vt-source.tar.gz"` (the lib-vt-only source tarball Ghostty publishes for every commit; this is the `v1.4.0` tag's commit `83fb956…`, 4.28 MB)
  - `hash = "ghostty-1.4.0-5UdBCyABPwEUlNNylMEI0O0f7XW8Q6chZlOyf_Ff3Wxe"`
- Ghostty's own build fetches `uucode` (and, with SIMD on, `simdutf` and `highway` from its `pkg/`) on first build; `zig build --fetch` fetches everything up front.

### `zig/build.zig`

- `standardTargetOptions` and `standardOptimizeOption`.
- `b.dependency("ghostty", .{ .target, .optimize })`: as a dependency, Ghostty builds in lib-vt mode by default (`emit_lib_vt` defaults to `is_dep`). SIMD stays at Ghostty's default (on); the prototype ran with it off, so the first build of this crate confirms the SIMD libraries link into the static archive.
- A root module from `src/vtwrap.zig` with `link_libc = true`, importing `ghostty.module("ghostty-vt")`.
- `b.addLibrary(.{ .name = "opsis_vtwrap", .linkage = .static })` with `bundle_compiler_rt = true`; on Windows, `bundle_ubsan_rt = false` and, for the MSVC ABI, `stack_protector = false` on the root module, the same settings Ghostty uses for its own static library (`src/build/GhosttyLibVt.zig`).
- The library name is ours, so the archive is `libopsis_vtwrap.a` on Linux and macOS and `opsis_vtwrap.lib` on Windows; no Windows renaming.

### `zig/src/vtwrap.zig`

Builds on the types Ghostty's own C layer uses (`src/terminal/c/terminal.zig`, `render.zig`): `vt.Terminal.init(io, alloc, .{ .cols, .rows })` with `vt.TinyIo` as the I/O, `terminal.vtHandler()`, `vt.TerminalStream.init(.{ .allocator, .handler })`, `stream.nextSlice(bytes)`, and `vt.RenderState` (`.empty`, `update`, `row_data`, `colors`, `cursor`). The handle is heap-allocated and never moves, because the handler keeps a pointer to the terminal and the effect callbacks find the handle with `@fieldParentPtr` from the stream, as Ghostty's C layer does.

Effects it installs on the handler:

| Effect                                 | What the wrapper does                                                                                                     |
| -------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `write_pty`                            | appends to a reply buffer the Rust side drains and writes to the pseudo-terminal                                          |
| `device_attributes`                    | returns Ghostty's defaults (`.{}`: VT220 with ANSI colour); without it, `CSI c` goes unanswered (found in the prototype)  |
| `semantic_prompt`                      | queues prompt start, input start, command start and command done (with the exit code) events (OSC 133)                    |
| `render_hold`                          | on begin, updates the render state inside the callback and freezes it; on end, unfreezes (synchronized output, mode 2026) |
| `title_changed`, `pwd_changed`, `bell` | queue events                                                                                                              |
| `size`                                 | answers size reports with the columns, rows and the design cell size                                                      |
| `enquiry`, `xtversion`                 | answer empty and `Opsis`                                                                                                  |

Exported functions (all `export fn`, C calling convention):

| Function                                                                                                                              | Purpose                                                                                                             |
| ------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| `opsis_vt_abi()`                                                                                                                      | returns an ABI number, plus `opsis_vt_sizes(*Sizes)` with `@sizeOf` of every struct; Rust compares both at creation |
| `opsis_vt_new(cols, rows, scrollback_rows) -> ?*Vt`                                                                                   | creates the terminal; null on a zero size or no memory                                                              |
| `opsis_vt_free(*Vt)`                                                                                                                  | frees everything                                                                                                    |
| `opsis_vt_feed(*Vt, ptr, len)`                                                                                                        | parses output bytes                                                                                                 |
| `opsis_vt_resize(*Vt, cols, rows)`                                                                                                    | resizes (reflow is Ghostty's)                                                                                       |
| `opsis_vt_take_replies(*Vt, out, cap) -> len`                                                                                         | drains the reply buffer                                                                                             |
| `opsis_vt_take_events(*Vt, out: [*]EventOut, cap) -> n`                                                                               | drains the event queue                                                                                              |
| `opsis_vt_title(*Vt, out, cap) -> len`                                                                                                | the current title                                                                                                   |
| `opsis_vt_release_hold(*Vt)`                                                                                                          | ends a render hold that lasted too long                                                                             |
| `opsis_vt_frame(*Vt, *FrameOut, cells: [*]CellOut, cells_cap, rows: [*]RowOut, rows_cap, graphemes: [*]u32, graphemes_cap) -> status` | updates the render state (unless frozen by a hold) and writes the whole viewport in one call                        |

Structs (`extern struct`, mirrored with `#[repr(C)]` in `ffi.rs`):

- `CellOut { cp: u32, extra_start: u32, extra_len: u16, width: u8, underline: u8, fg: u32, bg: u32, underline_colour: u32, flags: u16 }`
  - `cp` is the first code point (0 for an empty cell); `extra_start` and `extra_len` point into the frame's grapheme buffer for combining marks and ZWJ sequences (no cap on grapheme length, unlike the prototype's four).
  - Colours are tagged and never resolved in Zig: the top byte is 0 (default), 1 (palette index in the low byte) or 2 (RGB in the low 24 bits). The page resolves them with the take's theme, so a theme change never prepares the frames again.
  - `width`: 0 spacer, 1 narrow, 2 wide. `underline`: Ghostty's underline style (none, single, double, curly, dotted, dashed).
  - `flags`: bold, italic, faint, blink, inverse, invisible, strikethrough, overline, hyperlink.
- `RowOut { dirty: u8, wrapped: u8, id: u64 }` (Ghostty's row identity, so the preparer reuses unchanged rows).
- `FrameOut { cols, rows, cursor_x, cursor_y, cursor_visible, cursor_shape, cursor_blink, held, palette_changed, default_fg, default_bg, cells_written, graphemes_written }`; when a program changed the palette or the default colours (OSC 4, 10, 11), `palette_changed` is set and `opsis_vt_palette(*Vt, out: [*]u32)` returns the 256 colours and the defaults, which then override the theme for the rest of the take.
- `EventOut { kind: u8, code: i32 }` with kinds `prompt`, `input`, `command`, `done` (code = exit status), `hold_begin`, `hold_end`, `title`, `pwd`, `bell`.

### `build.rs`

1. Reads `ZIG` (default `zig`), runs `zig version`, and fails with "Zig 0.17 is needed to build opsis-vt, found <version>" unless the major and minor are 0.17.
2. Runs `zig build -Doptimize=<ReleaseFast for release, Debug for debug> --prefix $OUT_DIR/zig-out` in `crates/vt/zig`, with `-Dtarget=<zig target>` only when Cargo's `TARGET` differs from `HOST` (`x86_64-unknown-linux-gnu` → `x86_64-linux-gnu`, `aarch64-apple-darwin` → `aarch64-macos`, `x86_64-pc-windows-msvc` → `x86_64-windows-msvc`, and the arm64 and GNU variants). The Zig cache goes to `$OUT_DIR/zig-cache`; Zig's global package cache stays shared.
3. Prints `cargo:rustc-link-search=native=$OUT_DIR/zig-out/lib`, `cargo:rustc-link-lib=static=opsis_vtwrap`, and `cargo:rerun-if-changed` for `zig/build.zig`, `zig/build.zig.zon`, `zig/src`, plus `cargo:rerun-if-env-changed=ZIG`.
4. Debug builds use `Debug` Zig, which links Zig's undefined-behaviour runtime; if that fails to link on any platform, debug builds use `ReleaseSafe` instead.

### The safe API (`src/terminal.rs` and friends)

```rust
pub struct Terminal { /* NonNull handle, reusable buffers */ }
impl Terminal {
    pub fn new(cols: u16, rows: u16, scrollback: u32) -> Result<Terminal, VtError>;
    pub fn feed(&mut self, bytes: &[u8]);
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<(), VtError>;
    pub fn take_replies(&mut self) -> Vec<u8>;
    pub fn take_events(&mut self) -> Vec<Event>;
    pub fn title(&mut self) -> String;
    pub fn release_hold(&mut self);
    pub fn frame(&mut self) -> Frame<'_>;
    pub fn palette(&mut self) -> Option<Palette>;
}
unsafe impl Send for Terminal {}
```

- `Terminal` is `Send` (one owner moves it to a thread) and not `Sync`; every method takes `&mut self`.
- `Frame<'_>` borrows the reusable buffers: `cols()`, `rows()`, `cursor() -> Cursor`, `held()`, `row(y) -> Row<'_>` with `cells()`, `dirty()`, `wrapped()`, `id()`; `Cell` gives `text() -> &str` (the cluster, built from `cp` and the grapheme buffer once per frame), `width() -> CellWidth`, `fg()`, `bg()`, `underline()`, `underline_colour()`, `flags() -> Flags`.
- `Colour` is `Default | Palette(u8) | Rgb(u8, u8, u8)`; `Flags` is a plain struct of `bool`s.
- `VtError` (`thiserror`): `Size { cols, rows }`, `OutOfMemory`, `Abi { expected, found }`.
- Rust and Zig must agree on every struct: `Terminal::new` compares `opsis_vt_abi()` and `opsis_vt_sizes` with Rust's `size_of` and returns `VtError::Abi` on any difference.

### Tests (`crates/vt/tests/interop.rs`)

Property tests with `proptest`, the prototype's six made general:

- The ABI number and every struct size agree.
- Printable ASCII strings shorter than a row come back as the row's text.
- Any byte string fed in chunks gives the same frame as fed at once (split points generated, including inside escape sequences and UTF-8 sequences).
- SGR colour and flag sequences round-trip into `Colour` and `Flags` (generated attributes, checked against the sequence that set them).
- Wide characters take two cells with a spacer; combining marks and ZWJ sequences of any length stay in one cell's text.
- Cursor position reports and device attributes are answered on the reply buffer.
- A cursor walk (generated moves) ends where the moves say, inside the grid.
- Scrolling keeps the last `rows` lines.
- OSC 133 marks produce prompt, command and done events with the exit code.
- A render hold freezes the frame until it ends or is released.
- Resizing to any size within limits keeps every row's cell count equal to the columns.

## `opsis-terminal` (`crates/terminal`)

No `unsafe`, which the workspace's `unsafe_code = "deny"` enforces. Its own error enum per module (`thiserror`), numbers through `opsis_types::numbers`, times in milliseconds (`u32` for settings, `f64` for event times), the take clock from `std::time::Instant` (terminal takes don't use the browser's `TakeClock`).

### Modules

```
crates/terminal/src/
  lib.rs          re-exports, executors()
  shell.rs        Shell, clean shell commands and prompts per OS
  integration/    the prompt scripts (bash, zsh, PowerShell) as include_str! files
  session.rs      the pseudo-terminal session: spawn, read, write, resize, end
  keys.rs         key names to bytes, using the terminal's modes
  driver.rs       plays TerminalSteps on a session, waits, logs
  take.rs         records a take: driver + cast writer + action log
  cast/           asciinema's asciicast module, adapted
  tape/           the .tape lexer, parser, model, writer, diagnostics
  steps.rs        tape <-> TerminalStep mapping
  prepare.rs      .cast -> terminal frames file
  frames.rs       the terminal frames binary format (writer)
  live.rs         the live terminal window's session (Play tape recording)
  executors.rs    the node executors
  errors.rs
```

### Shells (`shell.rs`, `integration/`)

- `ShellChoice::Clean | Own`.
- Clean, per OS:
  - Linux: `bash --noprofile --norc -i`, with `PS1` and `PROMPT_COMMAND` from `integration/bash.sh` passed through `--rcfile` (a file written into the take's child temp folder).
  - macOS: `zsh -f -i` with `ZDOTDIR` pointing at a temp folder holding `integration/zshrc`.
  - Windows: `pwsh -NoLogo -NoProfile -NoExit -Command <integration/profile.ps1>` when `pwsh` is on `PATH`, else the same with `powershell.exe`.
- Every clean shell gets the same prompt (`opsis:<folder>$ ` with the folder shortened to `~`, coloured green and blue) and emits OSC 133 `A` (prompt), `B` (input), `C` (command) and `D;<status>` (done).
- `Own` runs the user's login shell (`$SHELL` on Linux and macOS, `pwsh`/`powershell.exe` on Windows) with its own config; Run command then waits on quiet output.
- Environment for every session: `TERM=xterm-256color`, `COLORTERM=truecolor`, `LANG` and `LC_ALL` set to a UTF-8 locale (`C.UTF-8` on Linux, `en_US.UTF-8` on macOS), `TMPDIR`/`TEMP`/`TMP` set to the take's child temp folder (`crates/engine/src/temp.rs`), plus the node's variables.
- The working folder is the node's Folder setting, resolved against the project folder.

### Session (`session.rs`)

- `portable_pty::native_pty_system().openpty(PtySize { rows, cols, .. })`, `spawn_command`, `try_clone_reader`, `take_writer`.
- A reader thread owns the `opsis_vt::Terminal`: it feeds every chunk, writes the replies back to the pseudo-terminal at once (programs block on their queries), drains the events, and sends `(time, bytes, events)` to the driver over a channel. Bytes are decoded to UTF-8 for the `.cast` with a carry for sequences split across reads.
- The child process is remembered like the browsers are (`remember_child`, `crates/engine/src/children.rs`) and ended on app exit.
- A session ends by writing the shell's exit command, waiting up to 2 s, then killing the process.

### Keys (`keys.rs`)

- Key chords from the `keys` widget and from tapes become bytes: printable text as UTF-8; Enter `\r`; Tab `\t`; Backspace `0x7f`; Escape `0x1b`; Ctrl+letter `0x01..0x1a`; arrows, Home, End, Page Up and Down, Insert, Delete and F1–F12 as xterm sequences, using `CSI` or `SS3` for arrows according to the terminal's application cursor mode, read from the terminal; Alt as an `ESC` prefix.

### Driver (`driver.rs`)

Plays `TerminalStep`s on a session and logs what it did.

| Step                   | Behaviour                                                                                                                                                                                                                                                                      |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Run command            | types the command at the session's typing speed, presses Enter, then waits: for the OSC 133 done event (clean shell) or for 300 ms without output (own shell), up to the step's time limit (default 60 s, then the take fails with "<command> didn't finish within <limit> s") |
| Type                   | types the text, one character per typing interval                                                                                                                                                                                                                              |
| Press keys             | presses the chord, Repeat times                                                                                                                                                                                                                                                |
| Wait for text          | waits until the screen shows a regex (from tapes' `Wait`), up to its limit                                                                                                                                                                                                     |
| Sleep                  | waits                                                                                                                                                                                                                                                                          |
| Start recording        | ends the off-camera part                                                                                                                                                                                                                                                       |
| Hide, Show             | from tapes: off-camera sections, any number                                                                                                                                                                                                                                    |
| Chapter                | marks a chapter                                                                                                                                                                                                                                                                |
| Clear screen           | presses Ctrl+L                                                                                                                                                                                                                                                                 |
| Scroll up, Scroll down | from tapes: scroll the view by lines                                                                                                                                                                                                                                           |
| Copy, Paste            | from tapes: a clipboard of the take's own                                                                                                                                                                                                                                      |
| Env, Source, Require   | from tapes: set a variable for the next commands, include another tape, check a program exists (fails with "<program> isn't installed")                                                                                                                                        |
| Screenshot             | from tapes: a marker; the card counts them                                                                                                                                                                                                                                     |

- After every step but Sleep, the step's Wait after.
- Before the first step the driver waits for the first prompt (OSC 133 `A`) or 300 ms of quiet, up to 5 s.
- After the last step it waits for 300 ms of quiet, up to 5 s, then ends the take.

### Take (`take.rs`)

Produces, in the node's output folder:

- `take.cast`: `.cast` v3 written with the adapted asciinema encoder. The header has the columns, rows, `term.type`, `timestamp`, `command` and the theme. Events:
  - `o` for output;
  - `i` for every input byte the driver wrote;
  - `m` markers for Start recording (`opsis:start`), Hide and Show (`opsis:hide`, `opsis:show`), chapters (`opsis:chapter:<title>`), step starts (`opsis:step:<node id>`) and screenshots;
  - `r` on resize;
  - `x` on exit.
- `take.log.json`: the take's `ActionLog`:
  - `viewport` is the design pixel size (columns × cell width, rows × cell height at the take's font size and line height), `dpr` 1;
  - every action has `node_id`, its time `t` on the visible timeline, and a `rect` of the cursor's line at that moment in design pixels, so Zoom here and Auto zoom frame the active line;
  - Run command and Type log `ActionKind::Type` with `text` and `key_times` (so Keystroke chips and Action sounds work), Press keys logs `ActionKind::Key` with `text`, Chapter logs `ActionKind::Chapter` with `label`, Wait for text logs `ActionKind::Wait`.

The visible timeline starts at Start recording (or the first step) and skips Hide sections.

### `.cast` (`cast/`)

- Copied from asciinema 3.2.0 `src/asciicast.rs` and `src/asciicast/{util,v1,v2,v3}.rs` (1,653 lines with tests).
- Adapted:
  - `anyhow` becomes `CastError` (`thiserror`: `Header`, `Event { line }`, `Version`, `Io`);
  - `crate::tty::TtyTheme` and `rgb::RGB8` become our own `CastTheme { fg, bg, palette }` with a plain RGB type;
  - comments are removed;
  - its own tests stay.
- Kept: the v1, v2 and v3 readers (`open`), the v3 encoder (`V3Encoder`), `limit_idle_time` and `accelerate`. The v2 encoder is removed (we only write v3).
- `THIRD_PARTY_NOTICES.md` gets asciinema's GPL-3.0-or-later notice for the copied files.

### `.tape` (`tape/`)

- Lexer and parser following VHS's grammar (`vhs/lexer/lexer.go`, `parser/parser.go`, `token/token.go`):
  - strings in double quotes, single quotes and backticks;
  - durations in `ms`, `s` and bare seconds;
  - repeat counts (`Enter 3`, `Down 5`);
  - per-command typing speed (`Type@500ms "…"`);
  - modifiers (`Ctrl+Alt+Shift+<key>`, `Alt+<key>`, `Shift+<key>`);
  - `Wait[+Screen][+Line][@<timeout>] /regex/`;
  - comments (`#`).
- Model: `Tape { settings: TapeSettings, steps: Vec<TapeStep>, ignored: Vec<Ignored> }`.
- Diagnostics: every error names the line and column and what was expected, in plain words.
- Writer: `TerminalSession` steps back to tape text (Export tape), and live recordings to tape text.
- Tests: every example in VHS's `examples/` folder parses; parse → write → parse is the identity (property test over generated tapes); errors on generated broken lines name the right line.

How tape commands map (`steps.rs`):

| Tape                                                                                                                                                                   | Opsis                                                                                           |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| `Type`, `Type@<speed>`                                                                                                                                                 | Type step (with its speed)                                                                      |
| `Enter`, `Tab`, `Space`, `Backspace`, `Delete`, `Insert`, `Escape`, arrows, `PageUp`, `PageDown`, `Home`, `End`, `Ctrl+…`, `Alt+…`, `Shift+…`, with counts             | Press keys step                                                                                 |
| `Sleep`                                                                                                                                                                | Sleep step                                                                                      |
| `Wait…`                                                                                                                                                                | Wait for text step                                                                              |
| `Hide`, `Show`                                                                                                                                                         | Hide, Show steps                                                                                |
| `ScrollUp`, `ScrollDown`                                                                                                                                               | Scroll steps                                                                                    |
| `Copy`, `Paste`                                                                                                                                                        | Copy, Paste steps                                                                               |
| `Env`                                                                                                                                                                  | Env step                                                                                        |
| `Source`                                                                                                                                                               | the other tape's steps, inline                                                                  |
| `Require`                                                                                                                                                              | Require step                                                                                    |
| `Screenshot`                                                                                                                                                           | Screenshot marker                                                                               |
| `Set Shell`                                                                                                                                                            | the shell (a tape naming a shell runs it with the user's setup; `Set Shell` absent means Clean) |
| `Set Columns`, `Set Rows`                                                                                                                                              | the size                                                                                        |
| `Set Width`, `Set Height`                                                                                                                                              | the size in pixels, turned into columns and rows from the font size, line height and padding    |
| `Set TypingSpeed`                                                                                                                                                      | the session's typing speed                                                                      |
| `Set Theme` (name or JSON), `Set FontSize`, `Set LineHeight`, `Set LetterSpacing`, `Set Padding`, `Set Margin`, `Set WindowBar`, `Set BorderRadius`, `Set CursorBlink` | the take's look                                                                                 |
| `Set FontFamily`                                                                                                                                                       | ignored: the font stack is fixed                                                                |
| `Set Framerate`                                                                                                                                                        | ignored: the project's frame rate decides                                                       |
| `Set PlaybackSpeed`                                                                                                                                                    | the take's speed                                                                                |
| `Output`, `Set LoopOffset`                                                                                                                                             | ignored                                                                                         |

Ignored commands are listed on the node's card ("Ignored: Output, Set FontFamily").

### Preparing (`prepare.rs`, `frames.rs`)

- Input: a take's `.cast` (or an imported one) and the film's frame rate. Output: a terminal frames file.
- Replays the events into a fresh `opsis_vt::Terminal`, builds the visible timeline from the markers (as the take does), and for every film frame `k` feeds all output up to `k / fps` and reads `frame()`. During a render hold it keeps the held frame and releases the hold after 1 s of take time.
- For imported files, the node's Shorten pauses setting goes through `limit_idle_time` first.
- Format (`frames.rs`, little-endian):
  - header: magic `OPSTERM1`, `cols: u16`, `rows: u16`, `fps: u32`, `frames: u32`, `palette_changes: u32`;
  - a row table: every distinct row once, as runs of `[col: u16, cells: u16, text offset: u32, text length: u16, fg: u32, bg: u32, underline_colour: u32, flags: u16, underline: u8]`, with one UTF-8 text block;
  - per frame: `rows` row indices (`u32`), cursor x and y (`u16`), visible, shape and blink (`u8` each);
  - palette changes as `(frame, 256 + 2 colours)`.
- Runs merge neighbouring cells with the same colours and flags when every cell in them is narrow printable ASCII; every other character is a run of its own, so its width is held to its cells.
- Rows that Ghostty marks clean reuse their previous index without encoding.
- Hooked into prepared files:
  - `PreparedKind::Terminal { fps: u32 }` in `crates/compile/src/asset_ref.rs` (path segment `terminal/<fps>`);
  - `opsis-prepare`'s `serve.rs` calls `opsis_terminal::prepare` for it;
  - cached under `.opsis/cache/prepared/` like the other kinds.

### Executors (`executors.rs`)

Registered in `crates/desktop/src/run.rs` next to `capture_executors` (`terminal_executors(&deps)`):

| Node type                | Executor                                                                                                                                                                                                                                                                                     |
| ------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `capture.recordTerminal` | runs the session's steps, writes `take.cast` and `take.log.json`, returns a `RecordingValue` with `terminal` set                                                                                                                                                                             |
| `terminal.playTape`      | parses the tape (with `Source` resolved), runs it like Record terminal take; stamped with the tape file and every sourced file (`Executor::stamped`), so editing a sourced tape runs it again; opens the live window first when the file is missing or the node is run on its own (`forced`) |
| `terminal.castFile`      | reads the `.cast`, checks it, writes `take.log.json` from its `i` and `m` events (keys and chapters), returns the recording                                                                                                                                                                  |
| `output.tape`            | writes the session's steps as a `.tape` (claims the file)                                                                                                                                                                                                                                    |
| `output.cast`            | copies a terminal take's `.cast` to the chosen file (claims the file)                                                                                                                                                                                                                        |

Sessions run in the run's child temp folder.

### Live window (`live.rs`)

For Play tape's live recording:

- **Rust side:** `opsis-terminal` runs a session and streams frames as the binary row format (one frame per change, at most 30 a second) over a `Channel` to a window at route `#/terminal/<id>`, opened by `opsis-desktop` like the streamed views (`open_terminal_window`).
- **The window:**
  - draws with the same `TerminalScreen` component the film uses;
  - sends keys and pasted text back with `terminal_input`;
  - shows Save and Cancel. Save writes the tape: the typed text grouped into `Type` lines, keys as key commands, and pauses over 500 ms as `Sleep`.
- **Shell:** the window's session uses the node's shell choice.

## Types and compile (`opsis-types`, `opsis-compile`)

### Types

- `PortType::TerminalSession` (`crates/types/src/workflow.rs`), with no conversions into or out of it (`crates/compile/src/ports.rs`).
- `TerminalSessionValue` (`crates/types/src/values.rs`): `{ shell: ShellChoice, folder: String, cols: u16, rows: u16, env: Vec<EnvVar>, typing_ms: u32, steps: Vec<TerminalStep> }`.
- `TerminalStep { id, node_id, label, action: TerminalAction, wait_after_ms: u32 }`.
- `TerminalAction` enum: `Run { command, wait: RunWait, limit_ms }`, `Type { text, speed_ms }`, `Keys { keys, repeat }`, `WaitText { pattern, scope, limit_ms }`, `Sleep { ms }`, `StartRecording`, `Hide`, `Show`, `Chapter { title }`, `Clear`, `Scroll { lines }`, `Copy { text }`, `Paste`, `Env { name, value }`, `Require { program }`, `Screenshot`.
- `RecordingValue` gains `terminal: Option<TerminalTake>`: `{ cast: String (project ref), cols: u16, rows: u16, look: TerminalLook }`.
- `TerminalLook { theme: TerminalTheme, font_size: u32, line_height: f64, letter_spacing: f64, padding: u32, cursor_blink: bool, title: String }`.
- `TerminalTheme { name, fg, bg, cursor, palette: [String; 16] }`.
- `RecordingProps` (`crates/types/src/recording.rs`) gains `terminal: Option<TerminalProps>`: `{ frames: String (prepared ref, hosted), cols, rows, look }`.
- All exported with specta (`bun run types:export`).

### Nodes (`crates/compile/src/nodes/terminal.rs`)

Category `Terminal` (new `Category` variant, label "Terminal"). Every node gets a description, a card and a look (a stroke glyph from the six sets, checked by `crates/compile/tests/unit/looks.rs`; `bun run glyphs:bundle` after).

1. **Open terminal** (`terminal.open`): no input → `TerminalSession`.
   - Shell (Clean, My shell setup; default Clean).
   - Folder (default the project folder).
   - Columns (80, 20–300) and Rows (24, 5–100).
   - Typing speed (50 ms, 0–1000).
   - Environment variables (rows of name and value).
   - Card: shell, size, folder.
2. **Run command** (`terminal.run`): `TerminalSession` → `TerminalSession`.
   - Command (text, variables).
   - Wait for (Prompt, Quiet output; default Prompt).
   - Time limit (60 s, advanced).
   - Wait after (0.5 s).
   - Card: the command.
3. **Type** (`terminal.type`): `TerminalSession` → `TerminalSession`.
   - Text.
   - Typing speed (empty = the session's).
   - Wait after.
4. **Press keys** (`terminal.keys`): `TerminalSession` → `TerminalSession`.
   - Keys (`keys` widget).
   - Repeat (1–100).
   - Wait after.
5. **Start recording** (`terminal.startRecording`): `TerminalSession` → `TerminalSession`.
6. **Chapter** (`terminal.chapter`): `TerminalSession` → `TerminalSession`.
   - Title.
7. **Clear screen** (`terminal.clear`): `TerminalSession` → `TerminalSession`.
8. **Record terminal take** (`capture.recordTerminal`): `TerminalSession` → `recording`, executes.
   - Name.
   - Theme (a searchable picker over the 621 terminal themes, showing each theme's colours; default VHS's default).
   - Font size (20).
   - Line height (1.2).
   - Padding (24 px).
   - Window title (empty = the program's title).
   - Card: length, steps, size.
9. **Play tape** (`terminal.playTape`): no input → `recording`, executes.
   - Tape file (`asset`, accept `.tape`).
   - Record it live when missing (on).
   - Card: length, steps, ignored commands.
10. **Cast file** (`terminal.castFile`): no input → `recording`, executes.
    - Cast file (`asset`, accept `.cast`).
    - Shorten pauses over (empty = the file's own limit).
    - Theme (empty = the file's).
    - Card: length, size, events.
11. **Export tape** (`output.tape`): `TerminalSession` → file, executes, claims the file.
    - File name.
12. **Export cast** (`output.cast`): `recording` → file, executes, claims the file.
    - File name.
    - Warns when the recording isn't a terminal take.

Shared rules:

- Every action node (2–7) appends its own `TerminalStep` in `compile()` and passes its input through when skipped, like the browser nodes.
- Validation warns:
  - when a cursor-based style (Cursor, Click ripple, Spotlight, Highlight box, Tooltip, Callout arrow, Follow cursor, Motion blur) is connected to a terminal take;
  - when Export cast gets a browser take.
- Copy (`crates/compile/tests/unit/nodes_c.rs`): no tool names ("asciinema" and "VHS" never appear; the files are "cast files" and "tape files").

### Compile and the film

- `compile` turns a terminal `RecordingValue` into `RecordingProps.terminal` with `frames = prepared:terminal/<fps>/project/<cast path>`. `hosted` gives it its `.opsis/prepared/...` address.
- `footage_uses` (`crates/compile/src/footage.rs`) skips recordings with `terminal`.
- `used_font_names` (`crates/compile/src/fonts/mod.rs`) adds the terminal font stack's five families when any recording has `terminal`, so a film with a terminal take loads them first.
- `flatten_spec` treats terminal takes like other recordings (action marks, hits for Action sounds, chapters from Chapter actions).
- Terminal themes: VHS's `themes.json` (621 named themes, MIT), copied by a `bun run terminal:themes` script pinned to a VHS commit into `crates/compile/src/terminal/themes.generated.json` (a generated file in the `AGENTS.md` table), read by `terminal_themes()` and sent in the catalogue. The default theme is VHS's default. `Set Theme "<name>"` looks the name up exactly; `Set Theme { … }` (JSON) is used as it is; an unknown name falls back to the default with a warning on the card.

## Film page (`packages/video`)

- `packages/video/src/lib/terminal/frames.ts`:
  - decodes the frames file (`DataView`) into row tables and per-frame indices;
  - loads through `useHeldLoad` so a frame is never captured before the file has loaded;
  - is a pure function of the frame number.
- `packages/video/src/lib/terminal/theme.ts`: resolves tagged colours with the take's theme or the take's palette changes; inverse, faint and invisible here.
- `packages/video/src/lib/terminal/sprites.ts`: the sprite port.
  - `spriteFor(codePoint, cellWidth, cellHeight, weight) -> SVG markup | null`.
  - Ranges: box drawing `U+2500–257F`; blocks `U+2580–259F`; Braille `U+2800–28FF`; Powerline and Powerline Extra `U+E0B0–E0D4`; Symbols for Legacy Computing `U+1FB00–1FBFF`; its supplement `U+1CC00–1CEBF`; geometric shapes `U+25A0–25FF` as Ghostty draws them; branch drawing `U+F5D0–F60D`.
  - Geometry from Ghostty's `src/font/sprite/draw/*.zig` (box 929 lines, block 177, Braille 148, Powerline 400, legacy 1,413 and 633, geometric 198, branch 504), with stroke widths from the cell size the way Ghostty's `Metrics` derives them.
  - Property tests: every sprite stays inside its cell; lines that meet an edge meet it at the centre with the stroke width, so neighbours join.
- `packages/video/src/core/terminal-screen.tsx` (`TerminalScreen`):
  - a design box of `cols × cellWidth` by `rows × cellHeight` plus padding, where `cellWidth = fontSize × 0.6` (JetBrains Mono's advance) and `cellHeight = round(fontSize × lineHeight)`;
  - one absolutely placed element per run: text runs as spans with fixed width and height, sprite runs as SVG;
  - the cursor in its shape, blinking by the frame when the take says it blinks;
  - the background in the theme's colour;
  - `font-family` is the fixed stack, in this order: JetBrains Mono, Symbols Nerd Font Mono, Noto Sans Symbols 2, Noto Sans SC, Noto Color Emoji.
- `packages/video/src/core/recording-clip.tsx`: when `props.terminal` is set, draws `TerminalScreen` where it would draw `FootageFrame`; everything around it (camera, zoom, window frame, backdrop, keystroke chips) stays as it is. The window frame's title shows the take's title.
- No animation outside `useFrame()`, no CSS transitions, no colour literals beyond the themes' tables (`packages/video/src/colors.test.ts` lists none new).
- `PICTURE_VERSION` and `PREPARED_VERSION` stay as they are (nothing is released).

## Fonts

- `scripts/fonts-bundle.ts` gains the terminal stack. It fetches and checks by sha256 (pinned in the script), writing into `public/fonts/bundled/terminal/`:
  - JetBrains Mono 2.304 from `https://github.com/JetBrains/JetBrainsMono/releases/download/v2.304/JetBrainsMono-2.304.zip`: `fonts/variable/JetBrainsMono[wght].ttf` (303 KB) and `JetBrainsMono-Italic[wght].ttf` (309 KB), as TTF; `OFL.txt` kept;
  - Symbols Nerd Font Mono 3.4.0 from `https://deps.files.ghostty.org/NerdFontsSymbolsOnly-3.4.0.tar.gz`: `SymbolsNerdFontMono-Regular.ttf` (2.5 MB), as TTF; `LICENSE` (MIT) kept;
  - Noto Sans Symbols 2 from `@fontsource/noto-sans-symbols-2` (its subsets, 432 KB of WOFF2);
  - Noto Sans SC 400 and 700 from `@fontsource/noto-sans-sc` (3.7 MB of WOFF2 per weight, 102 files each with their `unicode-range`);
  - Noto Color Emoji from `@fontsource/noto-color-emoji` (9.4 MB of WOFF2, 12 files).
- The bundled list (`crates/compile/src/fonts/bundled.generated.json`) gets the five families, so `FontProvider` registers their faces. They don't appear in the font pickers.
- `bun add -d @fontsource/noto-sans-symbols-2 @fontsource/noto-sans-sc @fontsource/noto-color-emoji`.
- `public/fonts/bundled/terminal/**` joins the generated-files table, regenerated with `bun run fonts:bundle`.

## Editor

- Run panel: a terminal take's files are `take.cast` and `take.log.json`; the take plays in the Run panel through `TerminalScreen` over its prepared frames at the project frame rate, with the media player's controls.
- Live PiP during Record terminal take: the take streams frames like the live window; `LivePip` draws them with `TerminalScreen` (`watchLive` gets a terminal stream kind).
- The live terminal window (Play tape) as above; its bounds are saved like the other pop-outs; it closes with the main window.
- The Timeline shows terminal take bars with their action marks and chapters and no filmstrip.
- Inspector: the `.tape` and `.cast` file fields use the `asset` widget; Play tape's card lists ignored tape commands; the theme field shows the theme's colours.

## Repository changes (done as part of the work, not now)

- `Cargo.toml` workspace members: `crates/vt`, `crates/terminal`. `cargo add portable-pty -p opsis-terminal`; `cargo add thiserror -p opsis-vt`; `cargo add --dev proptest -p opsis-vt -p opsis-terminal`.
- `scripts/comments.ts` checks `.zig` files line by line (the no-comments rule; `//!` and `///` too).
- `bun run format:fix` runs `zig fmt crates/vt/zig`; `bun run lint:fix` runs `zig build` with `-Doptimize=Debug` in `crates/vt/zig` as a compile check.
- CI:
  - `mlugg/setup-zig` with `version: 0.17.0` on every platform job;
  - caches Zig's global cache;
  - runs `zig build --fetch` before Cargo.
- `AGENTS.md`:
  - Zig in the tooling paragraph;
  - the two crates in the crates paragraph;
  - the pins (Ghostty tarball, Zig 0.17) next to the specta pins;
  - the terminal fonts and `themes.generated.json` in the generated-files table;
  - a "Terminal takes" invariant paragraph (the engine is Ghostty's through `opsis-vt` only, only `opsis-vt` allows `unsafe`, takes are `.cast` v3, the font stack is fixed, sprites override cell-filling ranges).
- `THIRD_PARTY_NOTICES.md`: Ghostty (MIT, the wrapper's dependency and the sprite port), asciinema (GPL-3.0-or-later, the copied `asciicast` module), VHS (MIT, the tape grammar and `themes.json`), JetBrains Mono (OFL-1.1), Nerd Fonts Symbols Only (MIT), Noto Sans Symbols 2, Noto Sans SC and Noto Color Emoji (OFL-1.1), `portable-pty` (MIT).
- `examples/`: one example project with a terminal take (`bun run examples`).

## Tests

- `opsis-vt`: the interop properties above.
- `opsis-terminal`:
  - `.cast`: asciinema's own tests; write → read round trip of generated events (times within a microsecond); reading v1 and v2 files;
  - `.tape`: VHS's examples parse; parse → write → parse is the identity; diagnostics name the right line;
  - steps: tape → steps → tape is the identity for the commands that map both ways;
  - keys: every chord maps to the bytes xterm sends, in both cursor modes;
  - prepare:
    - frames from a fixed `.cast` against a golden file in `fixtures/terminal/` (`opsis-golden`);
    - the frame count is `ceil(visible length × fps) + 1`;
    - hidden sections take no frames;
    - a render hold shows no half-drawn frame;
  - session (ignored, needs a shell): a clean shell's prompt marks arrive; Run command waits for done; replies reach the program (`tput cols` prints the columns).
- `opsis-compile`:
  - the nodes' descriptions and copy;
  - `x-show-if` values;
  - a terminal take compiles to `RecordingProps.terminal` with a prepared ref;
  - `footage_uses` skips it;
  - the fonts are in `used_font_names`.
- `packages/video`:
  - frames decoding round trip against the Rust writer's golden file;
  - sprite properties;
  - `TerminalScreen` renders a fixed frame (happy-dom, structure only).
- Render (ignored, needs the dev engine and the film page): a terminal take renders through the frame driver; stills compared with `fixtures/terminal/stills`.
- End to end: a spec builds Open terminal → Run command (`echo hello`) → Record terminal take → Export video and checks the export exists.

## Order of work

1. `opsis-vt`: Zig wrapper, `build.rs`, safe API, interop tests; SIMD on confirmed on Linux, macOS and Windows in CI.
2. `opsis-terminal` core: shells and prompt integration, session, keys, driver, take, `.cast` module, prepare and the frames format, with their tests.
3. Types, nodes and compile: port type, values, the 12 node definitions, cards, validation, the prepared kind, footage and font hooks; `bun run types:export`.
4. Film page: frames decoder, theme, sprites (box, blocks, Braille, Powerline first, then legacy, geometric and branch), `TerminalScreen`, the recording clip switch.
5. Fonts: bundling script, generated files, font table.
6. `.tape`: lexer, parser, writer, mapping; Play tape and Export tape.
7. Cast file and Export cast.
8. Editor: Run panel playback, live PiP, the live terminal window for Play tape.
9. Repository changes: workspace, tooling, CI, `AGENTS.md`, notices, example.

## Sources

- Ghostty: https://github.com/ghostty-org/ghostty, the v1.4.0 lib-vt tarball (`https://tip.files.ghostty.org/83fb95624b0439ee660c2679b43b38c8b63769cd/libghostty-vt-source.tar.gz`), `include/ghostty/vt/render.h`, `terminal.h`, `snapshot.h`, `src/terminal/c/terminal.zig`, `src/terminal/render.zig`, `src/build/GhosttyZig.zig`, `src/build/GhosttyLibVt.zig`, `src/build/zig.zig`, `src/font/sprite/`, `src/font/embedded.zig`.
- asciinema 3.2.0 crate (`src/asciicast*`), asciicast v3 spec: https://docs.asciinema.org/manual/asciicast/v3/
- VHS: https://github.com/charmbracelet/vhs (`lexer/`, `parser/`, `token/`, README command reference).
- portable-pty 0.9.0 (`src/win/psuedocon.rs`), CreatePseudoConsole: https://learn.microsoft.com/en-us/windows/console/createpseudoconsole
- JetBrains Mono 2.304: https://github.com/JetBrains/JetBrainsMono/releases/tag/v2.304
- Nerd Fonts Symbols Only 3.4.0: `https://deps.files.ghostty.org/NerdFontsSymbolsOnly-3.4.0.tar.gz`
- Fontsource: `@fontsource/noto-sans-symbols-2` 5.3.0, `@fontsource/noto-sans-sc` 5.3.1, `@fontsource/noto-color-emoji` 5.3.2.
- Prior art read: Betamax (https://github.com/joshka/betamax), foley (https://github.com/GH-Jaider/foley), wterm (https://github.com/vercel-labs/wterm), ghostty-web (https://github.com/coder/ghostty-web), ttyd (https://github.com/tsl0922/ttyd), libghostty-rs (https://github.com/uzaaft/libghostty-rs).
