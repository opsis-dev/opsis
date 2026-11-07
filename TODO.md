# TODO

What's left that no plan covers: the places in the codebase that don't read as if Opsis had been written for Tauri and Rust from the start, in the order to fix them; bugs; ideas left for later; and behaviour never checked. Each item is removed when it is done.

## Written for Tauri and Rust

## Bugs

Wrong behaviour found in the session transcripts and confirmed in the current code.

- [ ] HEVC (H.265) files don't play in exports: the headless shell has no HEVC decoder (Chromium decodes HEVC only through platform hardware, so `canPlayType` is empty and loading fails with `DEMUXER_ERROR_NO_SUPPORTED_STREAMS`). The Run panel plays them (WebKitGTK through GStreamer's `avdec_h265`).

## Unconfirmed bugs

Seen once during a session and never reproduced or explained.

## To be implemented

Ideas you approved but left for later.

- [ ] A clip-level Reverse that plays a recording backwards through its time mapping, with one encode and no frame buffer.
- [ ] A lossless format option for Export video, for files that feed later nodes.
- [ ] A keep-quality encode option for the file nodes, in place of the fixed H.264 at quality 18 and `yuv420p`.
- [ ] A lossless checkpoint node that renders a clip's finished picture to a file the rest of the graph plays.
- [ ] Recording the page's own sound in a take.
- [ ] An Open terminal session source.
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
- [ ] A smaller FFmpeg build (shared, combined or trimmed) in place of the ffmpeg-static binary.
- [ ] Reopening the panels that were popped out when the app starts.
- [ ] A Keystroke chips option to leave out plain keys such as Esc, Enter and Tab.
- [ ] Typed values across the native editor's bridge in place of its JSON strings (`crates/qt/src/bridge.rs`: `settings_json`, `book_json`, `values_json`, `value_json`, `new_item_json`, `extra_json`, and the `_json` arguments of `set_params`, `edit_keyframes`, `edit_mix`, `eq_curve`, `edit_timeline`, `edit_markers`, `edit_gradient`). Edits become one typed call per operation (`mix_set_gain`, `timeline_trim`…) that Rust turns into its edit enums; values become cxx shared structs, with a `has_` flag for an optional value, a kind enum plus fields for an enum with data and a list of pairs for a map, converted from `opsis-types` with round-trip property tests. Done one domain at a time (Mixer, Timeline, Inspector), with no hand-written C++ generator.

## Briefs and deferred

Ideas that were discussed but never finalized or planned.

- [ ] An `#[element(…)]` macro that writes the 138 element `definition()` functions from attributes on their settings structs, the third part of the macros research after `#[settings]` and `#[choice]`.
- [ ] A way to see the film before exporting, in place of the removed Preview (its window, streamed page and Fast/Exact copies). Undecided.
- [ ] Measuring how fast a low-resolution draft export renders (for example 480p), and whether one can hold more than 20 frames a second.
- [ ] Skipping static frames in a render: compile marks each frame where nothing can change (every element past its animation end and before its exit, no camera move, no video, no recording, no shader time), and such a frame reuses the previous picture without a seek or capture. Drawing is most of a render's CPU (Chrome's GPU process), so drawing fewer frames is where a render gets faster.
- [ ] Scaled renders drawn one frame at a time: begin-frame screenshots ignore an emulated scale, so a film with a scale other than 1 takes a screenshot per frame (`launch_film` in `crates/capture/src/film.rs`). Setting the scale at launch (`--force-device-scale-factor`) would let it use begin-frame capture.
- [ ] More image operations in the footage chain (`crates/footage`): crop, rotate and flip, adjust, blur and sharpen, spotlight, magnify. Each needs its own design, since a crop changes the footage's size, which the camera and window frame depend on.
- [ ] Allowing CSS animations and the Web Animations API in the film. The frame driver owns the page clock (the smooth-take machinery holds it and steps every CSS and web animation), so they could be drawn frame-exactly; `AGENTS.md` bans them until this is decided.
- [ ] Shipping the ffmpeg-static binary inside the deb and rpm instead of downloading it in Settings → Engine. This would reverse shipping no FFmpeg of ours and bring FFmpeg's GPLv3 source offer into the release.
- [ ] Settings the audit left out because the code under them can't take them yet:
  - the transitions our presentations draw with fixed values: the clock wipe's direction and start angle, the iris's centre and direction, a fade through a colour and the lens zoom's centre;
  - which side a tooltip or callout sits on (needs placement that keeps the box on screen, `packages/video/src/camera/annotations.tsx`);
  - each transition's own settings in a Sequence's cuts (a nested form per cut from the chosen transition's schema);
  - the agent-run panel's width and text size (they feed its whole layout and length model).

## Unverified

Behaviour that was never checked.

- [ ] Restoring saved window positions (`visible_bounds`, `crates/desktop/src/windows.rs`) on Windows and macOS. X11 was checked in a virtual display without a window manager; under Wayland the compositor places windows, so positions are never restored there.
- [ ] Audio scrubbing on the Timeline.
- [ ] Whether long exports leak memory with `angle` drawing. Remotion documents a Chrome memory leak with `angle` on long renders (fixed once on macOS in Chrome 133, remotion #4918); it was never tried in Opsis. If it shows, the documented workaround is exporting in chunks, each in a fresh browser, and joining them.
