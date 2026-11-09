# Changelog

Notable changes to Opsis. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions will follow [Semantic Versioning](https://semver.org/) from the first release.

## [Unreleased]

Nothing has been released yet. This is what `main` holds today.

### Added

- A desktop app (Rust, with a native Qt Widgets editor) with a node editor that records a real website and turns the recording into a finished video.
- Browser actions that drive a real website in one take (18 actions, sign-in saved once and reused, a Start recording step, a live view of the recording browser in the editor and in its own Live window, and a replay with a step-by-step trace).
- Terminal actions that drive a real shell in one take, VHS tapes played or recorded live, cast files used as takes, and sessions exported as tapes.
- Camera and cursor nodes (auto zoom, manual zooms, pans, follow cursor, speed ramps, window and device frames, cursors, click ripples, keystroke chips, spotlights, blurred areas), keyframes edited in the Inspector and on the Timeline.
- More than a hundred graphic elements, 3D elements and models, 25 transitions plus a light-leak cut, 16 bundled font families and a font library.
- Audio with a Mixer (strips, buses with ducking, a master with loudness targets), media and image nodes, and colour grade nodes.
- Exports in 16:9, 9:16, 1:1 and 4:5 at once, with chapters and stills in social sizes.
- A Timeline, a Mixer and a Run panel that each open in their own window.
- Settings on Basic and Advanced tabs in the Inspector, with every value a user would want to change as a setting: element, node, export, browser action, camera and recording-style settings.
- Tidy up layout lines edges up with the handles they join and gives edges entering one column their own lanes.
- The top bar says Unsaved until the latest change is saved.
- A Local API (off by default) on a local socket for scripts and agents.
- End-to-end scenarios that drive the real app in a virtual display and on headless Wayland with real input, and view checks on Qt's offscreen platform.

### Changed

- Rewritten from the Electron app to Rust with a native editor in Qt 6 Widgets and KDDockWidgets: the project document, undo, autosave, compile, runs, shared window state, asset addresses and fonts all live in Rust, and the C++ views call the session through a `cxx` bridge.
- Scripts run with Bun.
- Media files are read by Opsis itself, so ffprobe is no longer downloaded; the engine is the recording browser only, and FFmpeg ships with the app.
- The editor uses Inter and JetBrains Mono.

### Removed

- The Preview window, with its Fast and Exact quality and its shape switcher; a take no longer makes a small preview copy of its recording.

### Fixed

- Tidy up layout no longer does nothing on a flow where a Sequence, Layers or another node with numbered inputs uses a third input or more.
