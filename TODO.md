# TODO

What's left outside the release plan: the places in the codebase that don't read as if Opsis had been written for Tauri and Rust from the start, in the order to fix them; bugs; ideas left for later; and behaviour never checked. Each item is removed when it is done. Release work (size, builds, signing, updates) is in `plans/release.md`.

## Written for Tauri and Rust

## Bugs

Wrong behaviour found in the session transcripts and confirmed in the current code.

## Unconfirmed bugs

Seen once during a session and never reproduced or explained.

## To be implemented

Ideas you approved but left for later.

## Briefs and deferred

Ideas that were discussed but never finalized or planned.

- [ ] A clip-level Reverse that plays a recording backwards through its time mapping, with one encode and no frame buffer.
- [ ] A lossless format option for Export video, for files that feed later nodes.
- [ ] A keep-quality encode option for the file nodes, in place of the fixed H.264 at quality 18 and `yuv420p`.
- [ ] A lossless checkpoint node that renders a clip's finished picture to a file the rest of the graph plays.
- [ ] Recording the page's own sound in a take.
- [ ] Removing `specs` from `CompileResult` (`crates/compile/src/compile.rs`), now that export nodes build their film from their `clip` input.
- [ ] An executor stamp hook that replaces the sign-in-specific `secret_params` and `sign_in_stamp` (`crates/project/src/stamps.rs`) in node cache keys.
- [ ] A node claims hook that replaces `sign_in_clashes` and `export_clashes` in `crates/compile/src/graph.rs`.
- [ ] A session value that carries its current address, so the picker and the reload warning stop looking for Go to page.
- [ ] Node looks on `NodeDef` (`crates/compile/src/node.rs`), so `NODE_LOOKS` in `apps/ui/src/nodes/looks.ts` comes from the catalogue instead of a list kept by hand.
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
- [ ] Exporting long films in chunks in fresh browsers and joining them, to avoid the GPU backend's memory leak.
- [ ] A Run panel that lists every node, nesting compile-only nodes under their export and actions under their capture.
- [ ] "Not in this run" and "Turned off" states in the Run panel.
- [ ] A smaller FFmpeg build (shared, combined or trimmed), which conflicts with keeping both ffmpeg-static binaries.
- [ ] Replacing `../../` and deeper imports in `apps/ui/src` with `@/` (40 files) and banning them in lint.
- [ ] Answering the API's `/status` from Rust's own state instead of asking the editor window (`read_project` in `crates/api/src/routes.rs`).
- [ ] `bun run elements:render` covering Opsis's own core elements too.
- [ ] Reopening the panels that were popped out when the app starts.
- [ ] A Keystroke chips option to leave out plain keys such as Esc, Enter and Tab.
- [ ] A `CONTRIBUTING.md`.
- [ ] A `CODE_OF_CONDUCT.md`.
- [ ] A `SECURITY.md`.
- [ ] Issue and pull request templates.
- [ ] A changelog.
- [ ] A "Why it's built this way" README section from PLAN.md's old decisions table.
- [ ] A project skill that launches the dev app headless with a short profile path for checks.

## Unverified

Behaviour that was never checked and isn't in the release test pass in `plans/release.md` (section 8).

- [ ] Resizing our undecorated window by dragging its edge under Wayland.
- [ ] Double-clicking the app bar to maximize under Wayland.
- [ ] Restoring saved window positions (`visible_bounds`, `crates/desktop/src/windows.rs`) on X11, Windows or macOS.
- [ ] Why HEVC files don't play on Linux: in the Preview window (the headless shell's decoders) and in the Run panel (WebKitGTK through GStreamer).
- [ ] Audio scrubbing on the Timeline.
- [ ] The live recording view in a popped-out Run window.
- [ ] Export chapters in a video player's chapter menu.
