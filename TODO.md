# TODO

What's left outside the release plan: the places in the codebase that don't read as if Opsis had been written for Tauri and Rust from the start, in the order to fix them; bugs; ideas left for later; and behaviour never checked. Each item is removed when it is done. Release work (size, builds, signing, updates) is in `plans/release.md`.

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

## Briefs and deferred

Ideas that were discussed but never finalized or planned.

- [ ] Replacing the small Remotion packages the video uses with what they wrap or with our own code: `@remotion/noise`, `@remotion/rough-notation`, `@remotion/lottie` and `@remotion/three` with the libraries underneath (`simplex-noise`, `roughjs`, `lottie-web`, `@react-three/fiber`); our own motion blur, Mac cursors (drawn from Lucide's ISC shapes), light leaks (a shader) and effects (shaders, or baked by FFmpeg) in place of `@remotion/motion-blur`, `@remotion/mac-cursors`, `@remotion/light-leaks` and `@remotion/effects`; and `@remotion/paths` copied in. `plans/motion-library.md` plans a new element on `@remotion/rough-notation`, so the two have to agree.
- [ ] An `#[element(…)]` macro that writes the 138 element `definition()` functions from attributes on their settings structs, the third part of the macros research after `#[settings]` and `#[choice]`.
- [ ] A way to see the film before exporting, in place of the removed Preview (its window, streamed page and Fast/Exact copies). Undecided.
- [ ] Measuring how fast a low-resolution draft export renders (for example 480p), and whether one can hold more than 20 frames a second.
- [ ] Shipping the ffmpeg-static binary inside the deb and rpm instead of downloading it in Settings → Engine. This would reverse "no FFmpeg of ours inside" in `plans/release.md` and bring its GPLv3 source offer into the release.
- [ ] Settings the audit left out because the code under them can't take them yet:
  - the transitions Remotion draws: the clock wipe's direction and start angle, the iris's centre and direction, a fade through a colour and the lens zoom's centre;
  - the light leak's colours (`@remotion/light-leaks` has none);
  - which side a tooltip or callout sits on (needs placement that keeps the box on screen, `packages/video/src/camera/annotations.tsx`);
  - each transition's own settings in a Sequence's cuts (a nested form per cut from the chosen transition's schema);
  - the agent-run panel's width and text size (they feed its whole layout and length model).

## Unverified

Behaviour that was never checked and isn't in the release test pass in `plans/release.md` (section 8).

- [ ] Restoring saved window positions (`visible_bounds`, `crates/desktop/src/windows.rs`) on Windows and macOS. X11 was checked in a virtual display without a window manager; under Wayland the compositor places windows, so positions are never restored there.
- [ ] Audio scrubbing on the Timeline.
- [ ] Whether long exports leak memory with `angle` drawing. Remotion documents a Chrome memory leak with `angle` on long renders (fixed once on macOS in Chrome 133, remotion #4918); it was never tried in Opsis. If it shows, the documented workaround is exporting in chunks, each in a fresh browser, and joining them.
