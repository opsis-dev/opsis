# TODO

What's left that no plan covers: the places in the codebase that don't read as if Opsis had been written for Rust and Qt from the start, in the order to fix them; bugs; ideas left for later; and behaviour never checked. Each item is removed when it is done.

## Written for Rust and Qt

- [ ] Code nothing calls or reads: the `x-accept` and `x-length-key` keywords the settings emit and nothing reads.

## Bugs

Wrong behaviour found in the session transcripts and confirmed in the current code.

- [ ] A clip length field shows "Usual length" even with a clip connected: `blank` (`crates/ui/src/inspector.rs`) looks for the setting's own key (`clipDuration`) in `InspectorContext.connected_inputs`, which holds the connected port names (`clip`), so `x-unconnected-zero-label` always wins.
- [ ] `x-zero-label` on a file (`asset`) or select field is never shown: the asset field always reads "Choose a file" and a select shows its `none_label`, so labels such as Text mask's "Gradient" and Terminal look's "The take's own" never appear.
- [ ] Card rows and the Inspector disagree on `x-show-if` with `"*"`: the Inspector treats `"*"` as any non-empty value (`ANY_VALUE`, `crates/ui/src/inspector.rs`), while `shown` in `crates/compile/src/nodes/elements.rs` compares it as text.
- [ ] `documents_every_route_it_serves` (`crates/api/tests/routes.rs`) leaves out `/openapi.json`, `/status`, `/nodes` and `/nodes/{type}`, so those routes could lose their OpenAPI entry without a failing test.

## Unconfirmed bugs

Seen once during a session and never reproduced or explained.

- [ ] The View menu didn't open once in the `runs` e2e scenario, at `run-panel-plays-the-export` (`crates/qt/cpp/e2e_ports.cc`), right after `changelog-exports`: the click reached the menu bar and the menu wasn't visible within `kMenuOpenMs` (`crates/qt/cpp/e2e_steps.cc`). It passed in the two runs after it (2026-11-08). `Scenario::ClickMenu` now names the open popup, the modal dialog and the active window when a menu doesn't open, so the next failure says what was in the way.
- [ ] The native editor froze once in the `runs` e2e scenario, in the action of `typing-waits-to-mark-changed` (`crates/qt/cpp/e2e_ports.cc`), and logged nothing until the 15-minute limit. That action starts a run with `session.run_node`, whose host side (`Ui::run_node`, `crates/ui/src/ui.rs`) blocks the GUI thread in `runtime.block_on` until the run has started. It passed in the 13 runs after it, all under gdb. To catch it: `OPSIS_QT_E2E_HANG_MS=240000 bun run e2e:qt runs` runs the app under gdb and writes every thread's backtrace into `target/e2e-qt/runs.log` if it's still going after that time.

## To be implemented

Ideas you approved but left for later.

- [ ] A clip-level Reverse that plays a recording backwards through its time mapping, with one encode and no frame buffer (the Reverse node, `media.reverse`, only reverses a whole file).
- [ ] A lossless format option for Export video, for files that feed later nodes (its ProRes is ProRes HQ, which isn't lossless).
- [ ] A keep-quality encode option for the file nodes, in place of the fixed H.264 at quality 18 and `yuv420p` they encode with (`EDIT_CRF`, `crates/media/src/codecs.rs`).
- [ ] A lossless checkpoint node that renders a clip's finished picture to a file the rest of the graph plays.
- [ ] Recording the page's own sound in a take.
- [ ] An App window session source.
- [ ] A Moments node that marks clicks and typing by hand on an imported video.
- [ ] A Resize window browser action.
- [ ] A Switch theme browser action.
- [ ] A Change language browser action.
- [ ] A Go offline browser action.
- [ ] A Sign out browser action.
- [ ] An Open a new tab browser action.
- [ ] Record take and Screenshot passing the session on, so one chain can make several takes or stills.
- [ ] A Chapter node on the Timeline that marks the start of any clip.
- [ ] Lid-angle keyframes on the 3D screen's laptop, so its lid can open and close on a playing clip.
- [ ] KTX2 textures in 3D models through three.js's bundled basis transcoder.
- [ ] A Run panel that lists every node, nesting compile-only nodes under their export and actions under their capture.
- [ ] "Not in this run" and "Turned off" states in the Run panel.
- [ ] A smaller FFmpeg build (shared, combined or trimmed) in place of the ffmpeg-static binary; `plans/ffmpeg.md` section 3 plans one trimmed build of Opsis's own.
- [ ] Reopening the panels that were popped out when the app starts.
- [ ] A Keystroke chips option to leave out plain keys such as Esc, Enter and Tab.
- [ ] Typed values across the native editor's bridge in place of its JSON strings (`crates/qt/src/bridge.rs`: `settings_json`, `book_json`, `values_json`, `value_json`, `new_item_json`, `extra_json`, and the `_json` arguments of `set_params`, `edit_keyframes`, `edit_mix`, `eq_curve`, `edit_timeline`, `edit_markers`, `edit_gradient`). Edits become one typed call per operation (`mix_set_gain`, `timeline_trim`…) that Rust turns into its edit enums; values become cxx shared structs, with a `has_` flag for an optional value, a kind enum plus fields for an enum with data and a list of pairs for a map, converted from `opsis-types` with round-trip property tests. Done one domain at a time (Mixer, Timeline, Inspector), with no hand-written C++ generator.

## Briefs and deferred

Ideas that were discussed but never finalized or planned.

- [ ] An `#[element(…)]` macro that writes the 138 element `definition()` functions from attributes on their settings structs, the third part of the macros research after `#[settings]` and `#[choice]`.
- [ ] A way to see the film before exporting, in place of the removed Preview (its window, streamed page and Fast/Exact copies). Undecided.
- [ ] Measuring how fast a low-resolution draft export renders (for example 480p), and whether one can hold more than 20 frames a second.
- [ ] Skipping static frames in a render: compile marks each frame where nothing can change (every element past its animation end and before its exit, no camera move, no video, no recording, no shader time), and such a frame reuses the previous picture without a seek or capture. Drawing is most of a render's CPU (Chrome's GPU process), so drawing fewer frames is where a render gets faster.
- [ ] Scaled renders drawn one frame at a time: begin-frame screenshots ignore an emulated scale, so a film with a scale other than 1 takes a screenshot per frame (`launch_film` in `crates/capture/src/film.rs`). Setting the scale at launch (`--force-device-scale-factor`) would let it use begin-frame capture. Moot if `plans/native-renderer.md` replaces the frame driver.
- [ ] More image operations in the footage chain (`crates/footage`): crop, rotate and flip, adjust, blur and sharpen, spotlight, magnify. Each needs its own design, since a crop changes the footage's size, which the camera and window frame depend on.
- [ ] Allowing CSS animations and the Web Animations API in the film. The frame driver owns the page clock (the smooth-take machinery holds it and steps every CSS and web animation), so they could be drawn frame-exactly; `AGENTS.md`'s film rules (every animation a pure function of `useFrame()`, no CSS transitions) rule them out until this is decided, and `plans/motion-library.md` allows a paused CSS animation only where a keyframe can't be rewritten.
- [ ] Settings the audit left out because the code under them can't take them yet:
  - the transitions our presentations draw with fixed values: the clock wipe's direction and start angle, the iris's centre and direction, a fade through a colour and the lens zoom's centre;
  - which side a tooltip or callout sits on (needs placement that keeps the box on screen, `packages/video/src/camera/annotations.tsx`);
  - the agent-run panel's width and text size (they feed its whole layout and length model).

## Unverified

Behaviour that was never checked.

- [ ] Restoring saved window positions (`visible_bounds`, `crates/session/src/windows.rs`) on Windows and macOS. X11 was checked in a virtual display without a window manager; under Wayland the compositor places windows, so positions are never restored there.
- [ ] Hearing audio scrubbing on the Timeline: the e2e step `timeline-scrub-ruler` checks only where the playhead lands.
- [ ] Whether long exports leak memory with `angle` drawing. Remotion documents a Chrome memory leak with `angle` on long renders (fixed once on macOS in Chrome 133, remotion #4918); it was never tried in Opsis. If it shows, the documented workaround is exporting in chunks, each in a fresh browser, and joining them.
