# Changelog

Notable changes to Opsis. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions will follow [Semantic Versioning](https://semver.org/) from the first release.

## [Unreleased]

Nothing has been released yet. This is what `main` holds today.

### Added

- A desktop app (Tauri 2 and Rust, with the editor in the system webview) with a node editor that records a real website and turns the recording into a finished video.
- Browser actions that drive a real website in one take (18 actions, sign-in saved once and reused, a Start recording step, a live view of the recording browser in the editor and in its own Live window, and a replay with a step-by-step trace).
- Camera and cursor nodes (auto zoom, manual zooms, pans, follow cursor, speed ramps, window and device frames, cursors, click ripples, keystroke chips, spotlights, blurred areas), keyframes edited in the Inspector and on the Timeline.
- 138 graphic elements (29 built in, 109 from the Remocn, snapcn and RemotionUI registries), 3D elements and models, 25 transitions, 16 bundled font families and a font library.
- Audio with a Mixer (strips, buses with ducking, a master with loudness targets), media and image nodes, and colour grade nodes.
- Exports in 16:9, 9:16, 1:1 and 4:5 at once, with chapters, stills in social sizes, and Export as code.
- A Timeline, a Mixer and a Run panel that each open in their own window, and a Preview window that streams the exact export picture from the headless browser.
- A Local API (off by default) on a local socket for scripts and agents.
- End-to-end tests that drive the real app in a virtual display.

### Changed

- Rewritten from the Electron app to Tauri and Rust: the project document, undo, autosave, compile, runs, shared window state, asset addresses and fonts all live in Rust, and the editor calls the generated commands directly.
- Scripts run with Bun.
