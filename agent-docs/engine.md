# The engine: browser, FFmpeg and child programs

The one recording browser every capture, view and render launches, the bundled FFmpeg, how media is probed, and the temp folders and process tracking of every child program. Paths are relative to `crates/` unless they name another folder. Packaging the FFmpeg sidecar is in [packaging.md](packaging.md); see also [capture.md](capture.md) and [AGENTS.md](../AGENTS.md).

## Code map

| Concern            | Where                                                                                                                                                                                                                     |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Browser pins       | `engine/src/downloads.generated.json` (`bun run engine:downloads`), read by `engine/src/downloads.rs`                                                                                                                     |
| Installer          | `engine/src/install.rs` (into `<data folder>/engine`, `fetch_verified`, `CalmProgress`); the dev and e2e installer `engine/src/bin/install-engine.rs`                                                                     |
| Settings → Engine  | `qt/cpp/engine_card.cc` → `EngineCard` (also on the setup screen); the `EngineView` bridge struct; `session/src/engine.rs` → `install_engine`, `choose_browser`, `get_engine_status`                                      |
| Browser check      | `session/src/engine.rs` → `verify_on_start`, `checked_engine_files`, `engine_checking`; `capture/src/browser.rs` → `check_browser`                                                                                        |
| Engine files       | `session/src/engine.rs` → `engine_files`; `engine/src/tools.rs` → `EngineFiles`, `browser_warning`; `engine/src/paths.rs` → `EngineMissing`                                                                               |
| Seed stamps        | `scripts/lib/browser-stamp.ts`                                                                                                                                                                                            |
| Bundled FFmpeg     | `engine/src/ffmpeg.rs` → `bundled_ffmpeg`, `FfmpegMissing`; pins `scripts/lib/ffmpeg.generated.json`                                                                                                                      |
| FFmpeg contract    | `media/src/capabilities.rs` → `check_ffmpeg`, `REQUIRED_*`                                                                                                                                                                |
| Probing            | `probe` (`opsis-probe`) behind `MediaTools::probe`; parity test `probe/tests/ffprobe.rs`                                                                                                                                  |
| Child temp folders | `engine/src/temp.rs` → `ChildKind`, `clear_stale_child_temps`, `temp_env`; `AppPaths::views_temp`                                                                                                                         |
| Child processes    | `engine/src/children.rs` → `remember_child`, `forget_child`, `end_remembered_children`, `end_children_of_gone_apps`; `session/src/session.rs` → `Session::end`; the start sweep in `ui/src/main.rs` → `tidy_earlier_runs` |

## Rules

### One browser

- **Capture, the streamed views and film renders launch the same Chrome Headless Shell**: Google's Chrome for Testing build, the version pinned for the platform by sha256 and size. The app ships no browser.
- **Settings store it as a path (`settings.engine.browser`, with its version, sha256, size and modified time), and nothing else locates it.** Settings → Engine either downloads it into `<data folder>/engine` and saves that path, or saves a browser the user chooses.
- **The download is checked before anything is replaced**: it goes to a `.download-*` file, a wrong sha256 or size deletes it and fails (`fetch_verified`), and only then is it unpacked into a staging folder, the program looked for there and made executable (`unpack_shell`), and the old browser folder replaced (`install.rs`). Whether it runs is the full check's question, below.
- **Either way the full check runs before the path is saved**: the browser answers `--version` and opens a page over CDP when started headless (`check_browser`). A failed choice leaves the saved browser as it was.
- **At start the saved browser's size and modified time are compared (`verify_on_start`)**; a changed one is hashed, and only a different hash runs the full check again, so a moved or replaced file shows as a problem in Settings.
- **Runs and the streamed views take only a browser that passed (`engine_files`), and while a check runs they wait for it (`checked_engine_files`)**, a run saying so in its log; a browser that can't be used says why (`EngineMissing`: not set up, still being checked, or failed its check).
- **The check app's and the end-to-end seeds save the browser with its full stamp (`browser-stamp.ts`)**, so they start with no check.
- **A browser of another version than the pinned one is allowed with a warning** (`browser_warning`).
- **Engine status is complete when the browser is ready.**
- **Never fall back to Playwright's browsers or the system Chrome for capture or render.**

### FFmpeg

- **FFmpeg is not part of the engine: the app ships it as the `opsis-ffmpeg` sidecar**, the ffmpeg-static program pinned per target in `scripts/lib/ffmpeg.generated.json`, and users can't choose their own.
- **`EngineFiles.ffmpeg` is the program `bundled_ffmpeg()` finds, or none (`.ok()`)**, so analysis and mixing need no setup. How it finds the program, and `OPSIS_FFMPEG` in debug builds, is in [packaging.md](packaging.md).
  - With none, every FFmpeg run through `MediaTools` fails with `FfmpegMissing` when it starts (`Ffmpeg::command`, `media/src/ffmpeg.rs`), and a file player shows no picture.
- **Nothing checks FFmpeg at run time; `check_ffmpeg` is the contract a replacement program must meet**, run as a test on the staged program (`bun run test:rust`) and on every CI build runner.
  - It requires every encoder, decoder, format and filter Opsis uses (`REQUIRED_*`; a test fails when a golden filter graph uses one the list lacks), and a configuration line with `--enable-gpl --enable-version3` and without `--enable-nonfree`.
- **FFmpeg runs and probes sit behind the `MediaTools` trait so tests can fake them.**

### Probing

- **Nothing reads media through ffprobe: `opsis-probe` reads a file's length, container, picture (codec, size, frame rate, pixel format) and sound (codec, sample rate, channels) from its own bytes**, the way ffprobe reports them, for every probe the app makes.
- **Its parity test (`probe/tests/ffprobe.rs`, ignored) compares it with ffprobe from the same ffmpeg-static release as the bundled FFmpeg, in `~/.cache/opsis-dev/ffmpeg`** (`bun run ffprobe:stage`, pinned beside FFmpeg in `scripts/lib/ffmpeg.generated.json`), on samples it makes with the staged FFmpeg in `~/.cache/opsis-dev/probe-samples`.

### Child programs

- **Every child program gets a temp folder under `<data folder>/tmp`, named `<app pid>-<kind>-<id>` (`child_temp_name`, which keeps the first 8 characters of the id), passed as `TMPDIR`, `TEMP` and `TMP`**, so whatever the browser or FFmpeg puts in the system temp folder lands there.
  - Each run gets its own (`run`, which film renders use too); each open file player decodes through FFmpeg in its own (`analysis`, `ui/src/player.rs`); the streamed views, their browsers, the browser check, the sign-in view and the terminal painter share one per app launch (`shell-views`, `AppPaths::views_temp`).
  - They are made when the app starts (`AppState::new`) or the run begins (`make_child_temp`) and deleted when it ends; at start `clear_stale_child_temps` deletes every folder in `<data folder>/tmp` whose name doesn't start with the pid of a running app.
  - Each launcher sets the three variables itself: `capture/src/browser.rs`, `terminal/src/shell.rs` and the footage decoders (`footage/src/decoder.rs`, `footage/src/terminal.rs`, which a render's footage and a file player's picture use) do, so a new launcher has to set them too.
  - An FFmpeg run through `MediaTools` (`media/src/ffmpeg.rs`: `run`, `pipe`; `probe` starts no program) gets them only through `Ffmpeg::with_temp`, which only tests call, so the media nodes, the mix, analysis and a file player's sound (`decode_stem`) run without them (TODO.md).
- **A child started in a folder nobody made fails**, so the folders above are made before anything starts in them, and the `runs` e2e scenario opens the browser picker on a fresh data folder.
- **Browsers and terminal shells don't end with the app on their own, so each one the app starts is remembered by pid (`remember_child`) until it closes (`forget_child`)**, and `Session::end` ends every one still open when the app exits (`end_remembered_children`).
- **At start `end_children_of_gone_apps` ends any process whose command line names a profile in a temp folder of an app that is gone**, before those folders are cleared; that finds browsers only, so a shell left by an app that crashed keeps running. Chrome rewrites its process title into one string, so the search reads the whole command line, never one argument.

## Deliberate limits

- **The browser hash is slow in a debug build** (`bun start`, tests), where `sha2` isn't optimized; that is left as it is.
