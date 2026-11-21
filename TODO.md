# TODO

What's left that no plan covers: the places in the codebase that don't read as if Opsis had been written for Rust and Qt from the start, in the order to fix them; bugs; ideas left for later; and behaviour never checked. Each item is removed when it is done.

## Unconfirmed bugs

Seen once during a session and never reproduced or explained.

- [ ] The View menu didn't open once in the `runs` e2e scenario, at `run-panel-plays-the-export` (`crates/qt/cpp/e2e_ports.cc`), right after `changelog-exports`: the click reached the menu bar and the menu wasn't visible within `kMenuOpenMs` (`crates/qt/cpp/e2e_steps.cc`). It passed in the two runs after it (2026-11-08). `Scenario::ClickMenu` now names the open popup, the modal dialog and the active window when a menu doesn't open, so the next failure says what was in the way.
- [ ] The native editor froze once in the `runs` e2e scenario, in the action of `typing-waits-to-mark-changed` (`crates/qt/cpp/e2e_ports.cc`), and logged nothing until the 15-minute limit. That action starts a run with `session.run_node`, whose host side (`Ui::run_node`, `crates/ui/src/ui.rs`) blocks the GUI thread in `runtime.block_on` until the run has started. It passed in the 13 runs after it, all under gdb. To catch it: `OPSIS_QT_E2E_HANG_MS=240000 bun run e2e:qt runs` runs the app under gdb and writes every thread's backtrace into `target/e2e-qt/runs.log` if it's still going after that time.

## To be implemented

Ideas you approved but left for later.

- [ ] A clip-level Reverse that plays a recording backwards through its time mapping, with one encode and no frame buffer (the Reverse node, `media.reverse`, only reverses a whole file).
  - The time mapping already takes times that run backwards (`remap_frame`, `crates/scene/src/keyed.rs`), but the footage decoder restarts a second before any frame earlier than the few it keeps (`needs_restart`, `KEPT_FRAMES`, `crates/footage/src/footage.rs`), so playing backwards needs decoding in batches; the clip's sound needs reversing too.
- [ ] A keep-quality encode option for the file nodes, in place of the fixed H.264 at quality 18 and `yuv420p` they encode with (`EDIT_CRF`, `crates/media/src/codecs.rs`).
  - The lossless encoders it could use are in place: Export video's lossless formats encode with FFV1 and lossless RGB H.264 (`film_pixel_format`, `crates/media/src/codecs.rs`), and the bundled FFmpeg is checked for them (`REQUIRED_ENCODERS`).
- [ ] A lossless checkpoint node that renders a clip's finished picture to a file the rest of the graph plays.
  - Export video (MP4 or MKV lossless) → Media file → Use as clip may already be it, since Export video passes its file on; then this is a template rather than a node.
- [ ] Recording the page's own sound in a take.
  - Chrome's `Page.startScreenRecording` (Chrome 153 on; the pinned engine is 157) records the tab's sound as Opus beside AV1 video in the headless shell, but only sound that really plays: muted, or with no sound server, the track is silent. A take would need a virtual sound output on each system so nothing plays aloud, and only real-timing takes can use it, since a smooth take holds the page clock.
- [ ] An App window session source.
- [ ] A Moments node that marks clicks and typing by hand on an imported video.
- [ ] A Resize window browser action.
- [ ] An Open a new tab browser action.
- [ ] Lid-angle keyframes on the 3D screen's laptop, so its lid can open and close on a playing clip.
- [ ] A Run panel that lists every node, nesting compile-only nodes under their export and actions under their capture.
- [ ] "Not in this run" and "Turned off" states in the Run panel.
- [ ] A smaller FFmpeg build (shared, combined or trimmed) in place of the ffmpeg-static binary; `plans/ffmpeg.md` section 2 plans one trimmed build of Opsis's own.
- [ ] Typed values across the native editor's bridge in place of its JSON strings (`crates/qt/src/bridge.rs`: `settings_json`, `book_json`, `values_json`, `value_json`, `new_item_json`, `extra_json`, and the `_json` arguments of `set_params`, `edit_keyframes`, `edit_mix`, `eq_curve`, `edit_timeline`, `edit_markers`, `edit_gradient`). Edits become one typed call per operation (`mix_set_gain`, `timeline_trim`…) that Rust turns into its edit enums; values become cxx shared structs, with a `has_` flag for an optional value, a kind enum plus fields for an enum with data and a list of pairs for a map, converted from `opsis-types` with round-trip property tests. Done one domain at a time (Mixer, Timeline, Inspector), with no hand-written C++ generator.

## Briefs and deferred

Ideas that were discussed but never finalized or planned.

- [ ] An `#[element(…)]` macro that writes the 138 element `definition()` functions from attributes on their settings structs, the third part of the macros research after `#[settings]` and `#[choice]`.
- [ ] A way to see the film before exporting, in place of the removed Preview (its window, streamed page and Fast/Exact copies). Undecided.
- [ ] Measuring how fast a low-resolution draft export renders (for example 480p), and whether one can hold more than 20 frames a second.
- [ ] Skipping the scene build for static frames: compile marks each frame where nothing can change (every element past its animation end and before its exit, no camera move, no video, no recording, no shader time), and such a frame reuses the previous picture without building its scene. The renderer already reuses the last picture when a frame's scene hashes the same (`Film::scaled_frame`, `crates/film/src/lib.rs`), skipping layout and paint, but it still builds and hashes every scene.
- [ ] More image operations in the footage chain (`crates/footage`): crop, rotate and flip, adjust, blur and sharpen, spotlight, magnify. Each needs its own design, since a crop changes the footage's size, which the camera and window frame depend on.
- [ ] Settings the audit left out because the code under them can't take them yet:
  - the transitions our presentations draw with fixed values: the clock wipe's direction and start angle, the iris's centre and direction, a fade through a colour and the lens zoom's centre;
  - which side a tooltip or callout sits on (needs placement that keeps the box on screen, `crates/compile/src/elements/camera/annotations.rs`);
  - the agent-run panel's width and text size (they feed its whole layout and length model).
- [ ] Keeping the assistant's scripts out of the Script panel. The panel shows only the runs started with its own Run button, and a Local API run (`run_lua` through `opsis-mcp`, named `request.lua`) answers its caller without touching the panel's output or status.
  - Both start through `run_script` (`crates/session/src/script.rs`), which clears the one `Scripts.output`, writes every printed line into it and stores the outcome in the one `Scripts.last`, whoever started the run; the panel's `view` (`crates/ui/src/script_panel.rs`) draws that state without checking which script it came from. So with `scripts/script.lua` open, an assistant script that failed and rolled back shows "Failed. Its changes to the project were undone." beside Run and its error in the output (seen 2026-11-22).
  - `run_script` already knows the source (`ScriptSource::Text` from `request_source` against `ScriptSource::File`) and the agent's `session` tag.
  - One script still runs at a time across both, so the panel's Run answers "A script is already running." while an assistant script runs.
- [ ] Driving the recording browser over a pipe instead of a DevTools port. While a take, a sign-in or a streamed view has the browser open, it listens for DevTools on a random `127.0.0.1` port with no password, so any program on the computer, any user's, can find the port, connect, and read the restored sign-in's cookies, read the page or act in it.
  - `launch_with` (`crates/capture/src/browser.rs`) sets no port, so `chromiumoxide` adds `--remote-debugging-port=0` (`chromiumoxide-0.9.1/src/browser/config.rs`, default `port: 0`), and `chromiumoxide` 0.9 connects only over a port.
  - Websites can't reach it: Chrome takes DevTools requests only for a `localhost` or IP host and refuses WebSocket origins `--remote-allow-origins` doesn't name. A program already running as the same user can usually unlock the keyring and the sealed sign-ins anyway, so the exposure is other users and programs that can make local connections without reading the user's files.
  - `--remote-debugging-pipe`, which Playwright itself uses, opens no port; it needs a CDP client that speaks over the pipe's file descriptors (a change to `chromiumoxide`, or another client).
  - The agent's browser (`PUT /project/browser`, `crates/session/src/agent_browser.rs`) uses the same port, open for longer and handed out on purpose; the pipe, or Opsis relaying the connection at a random single-use path, would tighten both.

## Unverified

Behaviour that was never checked.

- [ ] Restoring saved window positions (`visible_bounds`, `crates/session/src/windows.rs`) on Windows and macOS. X11 was checked in a virtual display without a window manager; under Wayland the compositor places windows, so positions are never restored there.
- [ ] Hearing audio scrubbing on the Timeline: the e2e step `timeline-scrub-ruler` checks only where the playhead lands.
- [ ] Whether long exports leak memory with `angle` drawing. Remotion documents a Chrome memory leak with `angle` on long renders (fixed once on macOS in Chrome 133, remotion #4918); it was never tried in Opsis. If it shows, the documented workaround is exporting in chunks, each in a fresh browser, and joining them.
