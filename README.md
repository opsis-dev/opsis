# Opsis

Record a real website and turn it into a polished product demo.

Opsis is a desktop app with a node editor. Browser nodes drive a real website (clicks, typing, scrolling) and record it in one take; camera, cursor, text, motion-graphics, transition, audio and media nodes turn the recording into a finished video, exported in several shapes at once. Everything runs on your computer.

> **Early preview.** Built and tested on Linux, where the deb, rpm, AppImage and pacman packages install and run. Nothing has been built on Windows or macOS yet, and there are no signed installers or auto-updates. Expect rough edges.

Built with Rust, [Qt](https://www.qt.io) Widgets and [KDDockWidgets](https://github.com/KDAB/KDDockWidgets) for the editor, React for the film page, and FFmpeg. See [`plans/`](./plans) for work that's planned but not built.

## What it does

- **Record:** 18 browser actions (go to page, click, type, hover, drag onto an element, by a distance or along a path, scroll, choose option, upload file, press keys, wait, check text, fake a response, freeze time, hide clutter, page scripts, chapters, HTML overlays…), a Sign in step that saves the login once and reuses it in every take, a Start recording step so setup runs off camera, a live view of the recording browser, and a replay with a step-by-step trace of every take.
- **Camera and cursor:** a camera that zooms to every action, manual zooms, pans, follow the cursor, speed up quiet parts, speed ramps, crop, laptop and phone frames, macOS, Windows and Linux window frames (light or dark, with the page address), studio backdrops, motion blur; cursor styles (a simple arrow, or a Mac cursor that turns into a hand over links and a text cursor over fields), click ripples, keystroke chips, spotlights and highlight boxes, clean or hand-drawn; blurred areas that hide private details and stay on the page while the camera moves. Cursor, marks, backdrop and frame settings form a style that one take or several share.
- **Graphics:** 138 elements (text effects, overlays, charts, UI scenes, device mockups, backgrounds, effects), plus Lottie animations (`.json` and `.lottie`), a mosaic effect for hiding private details, title cards with a hand-drawn mark on chosen words, and a picture look (colour, dark edges, film grain).
- **3D:** a 3D logo, card and text in any project font, 3D models from your project with their own animations, a clip shown on a 3D laptop, monitor, phone or floating screen that ends facing the camera, and floating shapes as a background; camera keyframes set in the Inspector and dragged on the timeline.
- **Transitions:** 25 plus a light-leak cut. A Sequence can use a different transition, length and hold before any of its clips.
- **Text and fonts:** 16 bundled font families; a font library you fill from font files or ZIPs, fonts installed on your computer, or free fonts downloaded online (only when you ask). Projects set fonts for text, headings and code, and copy the fonts they use so they open anywhere.
- **Audio:** music with fades and ducking under sound effects (3 built-in CC0 songs or your own files), music fitted to the film (stretched a little, or looped with a crossfade), sound effects (7 built-in CC0 sounds or your own files) at a time or on a browser action, click and key sounds at every action of a take, a whoosh at every transition, a riser that ends on an action, a marker or the clip's end, generated sounds (whoosh, riser, pop, ding, tap, swell, click) with their variations, tones and countdown beeps, room tone, any number of sounds under a clip, captions timed from a script.
- **Media:** trim, join, speed (up to 64×), reverse, resize, convert, GIF, extract audio, even out volume, grab a frame, web-ready; for sound: clean speech (noise, rumble, harsh "s" sounds, level), fix clicks and clipping, remove hum, shorten pauses, change pitch, voice effects (telephone, radio, rooms, echo) and a picture of the waveform or spectrum. Media file nodes read their file without a run, and nodes warn before a run when they get the wrong kind of file.
- **Images:** resize, crop (an area you draw or margins), rotate and flip, straighten, tilt in 3D, adjust colour, auto levels, blur and sharpen, place a logo or watermark, redact, spotlight, trim empty edges, make a colour transparent, outline, a grid of images, a framed screenshot on a backdrop, before and after, magnify, compare two screenshots, annotate with arrows, boxes, ovals, numbered steps and text, and save as PNG, JPEG, WebP or AVIF. Phone photos come out the right way up and in the right colours.
- **Export:** 16:9, 9:16, 1:1 and 4:5 in one go. Other shapes keep the whole window and zoom in closer on the action, with captions and lower thirds kept clear of the edges. Video files carry the film's chapters (from Chapter actions) for players that list them, with an optional chapter list to paste into a YouTube description. Also still images in social sizes.
- **Timeline:** the film as bars with filmstrips, waveforms, action marks, chapters, markers and music beats; move, trim, split, ripple, roll, slip, speed and keyframes, each written back to the nodes (or added as a named node) as one undo step; snapping, J/K/L, in and out points.
- **Mixer:** a strip per sound with trim, low cut, 4-band EQ, compressor, pan, fader, mute and solo, grouped under its bus; buses with low cut, EQ, compressor, presets for the bus or every sound on it, and sidechain ducking that can ignore short sounds; a master with EQ, compressor, limiter and loudness targets (−14, −16, −23 LUFS); meters and loudness readouts; type exact values on any knob; Ctrl-click or Shift-click strip names to apply a preset, paste or reset to several sounds at once. Exports play exactly the rendered mix, and a mix change only adds the new sound to the already rendered picture.
- **Editor:** 252 node types, 7 templates, undo/redo, groups, auto layout. Timeline, Mixer and Run are tabs, and each can open in its own window; the Timeline plays the film's sound on its own clock and marks the nodes under its playhead. The canvas has a mouse mode and a touchpad mode (two fingers move, pinch zooms) and a minimap you can hide.
- **Projects:** new projects go to `~/OpsisProjects` (changeable in Settings), and the New project dialog shows the folder it will create. If the project file changes on disk while it's open (a text editor, a script, version control), the editor loads it, or asks first when you have unsaved edits; it never saves over a change it hasn't seen.
- **Settings:** its own window with pages for General, Projects, Editor, Engine, Recording, Fonts, API, Licences (every licence Opsis and its fonts come with) and About.
- **Local API (off by default):** scripts and AI agents on your computer can read, change and run the open project over plain HTTP on a local socket (a named pipe on Windows) that only your user account can open. The socket is `api.sock` in the app's data folder; set `OPSIS_API_SOCKET` to another full path (at most 100 bytes) if that path is too long. `GET /openapi.json` describes it; `GET /nodes/{type}` gives each node's ports and settings. Every request is one undo step in the editor.

## Getting started

Requirements: [Bun](https://bun.sh), Node.js 22 or later, a Rust toolchain, Zig 0.17, and [uv](https://docs.astral.sh/uv/) for `bun run qt:install`, which puts Qt 6.12 and KDDockWidgets in `~/.cache/opsis-dev/qt`. On Linux, also the sound library and Qt's X11 libraries (on Ubuntu: `libasound2-dev libxcb-cursor0 libxkbcommon-x11-0 libxcb-icccm4 libxcb-image0 libxcb-keysyms1 libxcb-render-util0 libxcb-xinerama0 libxcb-randr0 libxcb-shape0 libxcb-xkb1`).

```sh
bun install
bun run qt:install
bun run film:bundle
bun run terminal:bundle
bun start
```

`bun run film:bundle` builds the film page that exports draw, and `bun run terminal:bundle` the page that draws live terminal pictures; run them again after changing `packages/video` or `packages/runtime`. `bun start` downloads the pinned FFmpeg into `target/ffmpeg/` the first time and points the app at it (`OPSIS_FFMPEG`); a release build looks for the same program beside its own. The first time, open **Settings → Engine** and either **Download engine**, which puts the recording browser (Chrome Headless Shell) in the app's data folder, or **Choose file…** for a browser you already have. Opsis checks it before using it, and again at start if the file changed. Editing and mixing work without it; recording and exporting need it. Then **New project → Sample: to-do app → Run**.

Ready-made copies of every template are in [`examples/`](./examples) (**Open project…**).

## Building

| Command              | What it does                                                                                                                                                                               |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `bun run make`       | Stage everything the app ships, build the release binary against the static Qt and package it into `~/.cache/opsis-dev/build` (the deb by default; `-- --formats deb,rpm,appimage,pacman`) |
| `CARGO_BUILD_JOBS=8` | Limit how many cores the Rust build uses                                                                                                                                                   |

The app ships FFmpeg (the pinned ffmpeg-static build, as the `opsis-ffmpeg` program beside the app's own) but never the recording browser; users download it or choose their own from Settings. On Linux the release binary and its packages build in a plain `debian:trixie` container. CI runs the checks and the end-to-end tests on Linux on every push ([`.github/workflows/build.yml`](./.github/workflows/build.yml)), and builds the installers for Linux, Windows and macOS on `v*` tags and by hand ([`.github/workflows/release.yml`](./.github/workflows/release.yml)).

## Scripts

| Script                    | What it does                                                                                      |
| ------------------------- | ------------------------------------------------------------------------------------------------- |
| `bun start`               | Run the app in development                                                                        |
| `bun run typecheck`       | `tsc --noEmit` everywhere                                                                         |
| `bun run lint:fix`        | oxlint, `cargo clippy`, clang-tidy and the comment check                                          |
| `bun run format:fix`      | oxfmt, `cargo fmt`, `zig fmt` and clang-format                                                    |
| `bun run test`            | vitest and `cargo test` (with the staged FFmpeg)                                                  |
| `bun run e2e:qt`          | The editor driven through its scenarios in a virtual display (needs `xvfb`; `sway` for `wayland`) |
| `bun run types:export`    | Write the film's TypeScript types from the Rust code                                              |
| `bun run film:bundle`     | Build the film page that exports draw                                                             |
| `bun run terminal:bundle` | Build the page that draws live terminal pictures                                                  |
| `bun run fonts:bundle`    | Copy the bundled fonts from their Fontsource packages                                             |
| `bun run brand:icons`     | Render `brand/logo.svg` to PNGs                                                                   |
| `bun run qt:install`      | Fetch Qt 6.12 and KDDockWidgets into `~/.cache/opsis-dev/qt`                                      |
| `bun run qt:static`       | Build the static Qt without ICU that release builds link, into `~/.cache/opsis-dev/qt-static`     |
| `bun run qt:notices`      | Write `QT_NOTICES.generated.md` from the static Qt's bill of materials                            |
| `bun run package:check`   | Install each Linux package in a plain `debian:trixie` container and run it                        |
| `bun run examples`        | Rewrite `examples/*/project.opsis.json` from the templates                                        |
| `bun run ffmpeg:stage`    | Download the pinned FFmpeg for this computer, if missing, and print its path                      |

## Architecture

```
Editor (Qt 6 Widgets + KDDockWidgets, C++ views over a cxx bridge)
   canvas · node list · inspector · timeline · mixer        ── views and calls (cxx) ──
                                                                         │
Rust: session · projects · app database · fonts · Local API
   compile (graph → TimelineSpec) · runner (topo-sort → sha256 cache → phases → events)
   capture over CDP · media (FFmpeg) · prepared 3D files · image nodes · streamed views
        │ CDP
   Chrome Headless Shell: recording, the film page frame by frame, sign-in, element picker
```

- **Nodes never generate film code.** They produce files and compile to a `TimelineSpec`; the film page's one `DemoTimeline` draws any spec, and Rust captures it frame by frame into FFmpeg.
- **Nodes pass resources forward:** a browser session (Open browser and its steps), a recording, a recording style, clips, files, images, sounds and captions. Browser nodes add their steps to the session, and the Record node replays them in one continuous take. Timing comes from the recorded frames' own timestamps, so cursor moves, clicks and camera zooms line up with the picture.
- **Caching:** every node is hashed from its type, settings and upstream hashes; results are cached in `.opsis/cache/<hash>`, so only changed branches run again.

```
crates/      Rust: types, compile, store, project, fonts, engine, traces, runner, media, render,
             capture, stream, prepare, image, api, session, qt (the editor's views), ui (the app)

apps/film    the film page the renderer draws

packages/video  the film: DemoTimeline, elements, parts, lib, fonts, camera, 3D
packages/runtime the film's page runtime: clock, sequences, holds, maths
scripts/     fonts, icons, engine downloads, packaging stage
```

Contributor rules (no code comments, kebab-case files, pinned versions, what must not regress) are in [AGENTS.md](./AGENTS.md).

## Why it's built this way

| Decision          | Choice                                                                                                      | Why                                                                                                                                   |
| ----------------- | ----------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| App shape         | One native app: Rust for everything that touches the system and owns the state, the editor in Qt Widgets    | A typed boundary, native widgets and docking, and one place (Rust) that owns the project, runs, settings and the state windows share. |
| Types and logic   | Rust is the source; the editor's views are the bridge's structs, the film's types are generated (specta)    | Views, calls and the timeline spec can't drift apart, and the editor keeps no copy of logic Rust already has.                         |
| Canvas            | Our own canvas on `QGraphicsView`, with layout and edge lanes worked out in Rust                            | Cards, ports, edges and the minimap drawn natively, and the same maths the Local API and tests use.                                   |
| Workflow format   | A versioned JSON graph, compiled in Rust to one `TimelineSpec`                                              | The graph is for people; the spec is the single input the video renders from.                                                         |
| Video engine      | React with Opsis's own small runtime, drawn frame by frame by Rust                                          | One page, `DemoTimeline`, renders every spec; Rust seeks it, captures each frame and encodes with FFmpeg, with no Node at run time.   |
| Browser           | One Chrome Headless Shell for recording, renders, sign-in and the element picker, driven over CDP from Rust | The same engine everywhere; Playwright's page scripts and keyboard are reused without its Node runtime.                               |
| Animations        | Frame-driven only: every element and transition                                                             | Every frame has to be a pure function of the frame number; CSS transitions and tween libraries don't survive a render.                |
| GSAP              | Not used                                                                                                    | The runtime's `interpolate`, `spring` and easing cover it, without GSAP's licence clause on visual builders.                          |
| FFmpeg            | The ffmpeg-static build, pinned by sha256 and shipped beside the app as a separate program                  | Exports work without setting anything up, and every run starts the same known build.                                                  |
| Camera and cursor | Built on [programatic-demo](https://github.com/ashrafchowdury/programatic-demo) (MIT)                       | It already solved the click log, auto-zoom camera, vector cursor and motion blur; its notice is kept.                                 |
| Offline           | Nothing uses the internet unless you ask                                                                    | Engine and font downloads sit behind a button, and a film never loads anything remote.                                                |
| Licensing         | GPLv3 or later                                                                                              | Free software, with no other licence needed to export.                                                                                |

## Licences

Opsis is free software under the [GNU General Public License version 3 or later](./LICENSE). Videos you make with Opsis are yours: the licence covers the program, not what it makes.

Bundled and vendored third-party code, fonts, sounds, FFmpeg and the recording browser keep their own licences; see [THIRD_PARTY_NOTICES.md](./THIRD_PARTY_NOTICES.md).
