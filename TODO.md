# TODO

What's left outside the release plan: the places in the codebase that don't read as if Opsis had been written for Tauri and Rust from the start, in the order to fix them; bugs; ideas left for later; and behaviour never checked. Each item is removed when it is done. Release work (size, builds, signing, updates) is in `plans/release.md`.

## Written for Tauri and Rust

## Bugs

Wrong behaviour found in the session transcripts and confirmed in the current code.

- [ ] The render GPU backend leaks memory over a long export: export long films in chunks, each in a fresh browser, and join them.
- [ ] HEVC (H.265) files don't play in the Preview window or in exports: the headless shell has no HEVC decoder (Chromium decodes HEVC only through platform hardware, so `canPlayType` is empty and loading fails with `DEMUXER_ERROR_NO_SUPPORTED_STREAMS`). The Run panel plays them (WebKitGTK through GStreamer's `avdec_h265`).

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
- [ ] A smaller FFmpeg build (shared, combined or trimmed), which conflicts with keeping both ffmpeg-static binaries.
- [ ] Reopening the panels that were popped out when the app starts.
- [ ] A Keystroke chips option to leave out plain keys such as Esc, Enter and Tab.

## Briefs and deferred

Ideas that were discussed but never finalized or planned.

## Unverified

Behaviour that was never checked and isn't in the release test pass in `plans/release.md` (section 8).

- [ ] Restoring saved window positions (`visible_bounds`, `crates/desktop/src/windows.rs`) on Windows and macOS. X11 was checked in a virtual display without a window manager; under Wayland the compositor places windows, so positions are never restored there.
- [ ] Audio scrubbing on the Timeline.
