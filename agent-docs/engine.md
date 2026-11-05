# The engine: browser, FFmpeg and child programs

The one recording browser every capture, view and render launches, the bundled FFmpeg, how media is probed, and the temp folders and process tracking of every child program. Paths are relative to `crates/` unless they name another folder. Packaging the FFmpeg sidecar is in [packaging.md](packaging.md); see also [capture.md](capture.md) and [AGENTS.md](../AGENTS.md).

## Code map

| Concern            | Where                                                                                                                              |
| ------------------ | ---------------------------------------------------------------------------------------------------------------------------------- |
| Browser pins       | `engine/src/downloads.generated.json` (`bun run engine:downloads`)                                                                 |
| Installer          | `engine/src/install.rs` (into `<data folder>/engine`)                                                                              |
| Browser check      | `desktop/src/engine.rs` → `verify_on_start`, `checked_engine_files`, `engine_checking`; `capture/src/browser.rs` → `check_browser` |
| Engine files       | `engine_files`, `EngineFiles`, `EngineMissing`                                                                                     |
| Seed stamps        | `scripts/lib/browser-stamp.ts`                                                                                                     |
| Bundled FFmpeg     | `engine/src/ffmpeg.rs` → `bundled_ffmpeg`, `FfmpegMissing`; pins `scripts/lib/ffmpeg.generated.json`                               |
| FFmpeg contract    | `media/src/capabilities.rs` → `check_ffmpeg`, `REQUIRED_*`                                                                         |
| Probing            | `probe` (`opsis-probe`) behind `MediaTools::probe`; parity test `probe/tests/ffprobe.rs`                                           |
| Child temp folders | `engine/src/temp.rs` → `clear_stale_child_temps`; `AppPaths::views_temp`                                                           |
| Child processes    | `engine/src/children.rs` → `remember_child`, `end_children_of_gone_apps`                                                           |

## Rules

### One browser

- **Capture, the streamed views and film renders launch the same Chrome Headless Shell**: Google's Chrome for Testing build, the version pinned for the platform by sha256 and size. The app ships no browser.
- **Settings store it as a path (`settings.engine.browser`, with its version, sha256, size and modified time), and nothing else locates it.** Settings → Engine either downloads it into `<data folder>/engine` and saves that path, or saves a browser the user chooses.
- **Either way the full check runs before the path is saved**: the browser answers `--version` and opens a page over CDP when started headless (`check_browser`). A failed choice leaves the saved browser as it was.
- **At start the saved browser's size and modified time are compared (`verify_on_start`)**; a changed one is hashed, and only a different hash runs the full check again, so a moved or replaced file shows as a problem in Settings.
- **While a check runs, runs and the streamed views wait for it (`checked_engine_files`)**, and a run says so in its log; a browser that can't be used says why (`EngineMissing`: not set up, still being checked, or failed its check).
- **The check app's and the end-to-end seeds save the browser with its full stamp (`browser-stamp.ts`)**, so they start with no check.
- **A browser of another version than the pinned one is allowed with a warning.** Engine status is complete when the browser is ready.
- **Runs and views take the browser that passed (`engine_files`).**
- **Never fall back to Playwright's browsers, the system Chrome or the webview for capture or render.**

### FFmpeg

- **FFmpeg is not part of the engine: the app ships it as the `opsis-ffmpeg` sidecar**, the ffmpeg-static b6.1.1 program pinned per target, and users can't choose their own.
- **`EngineFiles.ffmpeg` is `bundled_ffmpeg()`**, so analysis and mixing need no setup. It returns `OPSIS_FFMPEG` in debug builds when set, and otherwise `opsis-ffmpeg` beside `current_exe()`; `FfmpegMissing` when neither is there.
- **Nothing checks FFmpeg at run time; `check_ffmpeg` is the contract a replacement program must meet**, run as a test on the staged program (`bun run test:rust`) and on every CI build runner.
  - It requires every encoder, decoder, format and filter Opsis uses (`REQUIRED_*`; a test fails when a golden filter graph uses one the list lacks), and a configuration line with `--enable-gpl --enable-version3` and without `--enable-nonfree`.
- **FFmpeg runs and probes sit behind the `MediaTools` trait so tests can fake them.**

### Probing

- **Nothing reads media through ffprobe: `opsis-probe` reads a file's length, container, picture (codec, size, frame rate, pixel format) and sound (codec, sample rate, channels) from its own bytes**, the way ffprobe reports them, for every probe the app makes.
- **Its parity test (`crates/probe/tests/ffprobe.rs`, ignored) compares it with ffprobe from the same ffmpeg-static release as the bundled FFmpeg, in `~/.cache/opsis-ffmpeg`** (`bun run ffprobe:stage`, pinned beside FFmpeg in `scripts/lib/ffmpeg.generated.json`), on samples it makes with the staged FFmpeg in `~/.cache/opsis-probe-samples`.

### Child programs

- **Every child program gets a temp folder under `<data folder>/tmp`, named `<app pid>-<kind>-<id>`, passed as `TMPDIR`, `TEMP` and `TMP`**, so whatever the browser or FFmpeg puts in the system temp folder lands there.
  - Each run gets its own (`run`, which film renders use too); the streamed views and their browsers share one per app launch (`shell-views`, `AppPaths::views_temp`).
  - They are made when the app starts or the run begins and deleted when it ends; at start `clear_stale_child_temps` deletes folders whose app process is gone.
- **A child started in a folder nobody made fails**, so `e2e/src/specs/views.spec.ts` opens the picker on a fresh data folder.
- **Browsers don't end with the app on their own, so every browser `launch` starts is remembered by pid (`remember_child`) until it closes**, and the app ends every one still open when it exits (`RunEvent::Exit`).
- **At start `end_children_of_gone_apps` ends any process whose command line names a profile in a temp folder of an app that is gone**, before those folders are cleared. Chrome rewrites its process title into one string, so the search reads the whole command line, never one argument.

## Deliberate limits

- **The browser hash takes about 0.13 s in a release build and about 2.4 s in a debug build** (`tauri dev`, tests), where `sha2` isn't optimized; that is left as it is.
