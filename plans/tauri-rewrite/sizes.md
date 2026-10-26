# Sizes

What Opsis ships today and after the move, measured on Linux x64 (2026-10-25). Sizes are unpacked bytes on disk unless noted; installers are compressed and much smaller.

## What it adds

A measured basis for the size goal (from about 800–900 MB to about 600 MB) and a list of what is still unmeasured.

## Fit with Opsis

### Today

| Part                                                              | Size     | How measured                                          |
| ----------------------------------------------------------------- | -------- | ----------------------------------------------------- |
| Electron 45 (`node_modules/electron/dist`)                        | 286.8 MB | `du -sb`                                              |
| Chrome Headless Shell (dev engine, `TESTED_VERSION` 157.0.8080.0) | 247.0 MB | `du -sb ~/.config/Opsis/engine/chrome-headless-shell` |
| FFmpeg (`ffmpeg` 79.8 MB + `ffprobe` 79.7 MB + licence)           | 159.5 MB | `du -sb`                                              |
| Prebuilt Remotion bundle                                          | 67 MB    | disk usage of a fresh `bundleRemotionProject` output  |
| Remotion compositor (`@remotion/compositor-linux-x64-gnu`)        | 27.3 MB  | `du -sb`                                              |
| `playwright-core`                                                 | 13.7 MB  | `du -sb`                                              |
| Renderer build (`.vite`)                                          | 5.1 MB   | `du -sb`                                              |
| `public/` (fonts, backdrops, sounds, placeholders, Draco)         | 7.0 MB   | `du -sb`                                              |

### After

| Part                                           | Size       | Note                                                                                                                                                                                           |
| ---------------------------------------------- | ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Tauri app (Rust binary + UI build + `public/`) | 93.4 MB    | measured in phase 6 (2026-10-26, Linux x64): the release binary with the editor build and `public/` embedded, 69.8 MB stripped; no size work yet (`opt-level`, LTO, `strip` come with phase 8) |
| Bun 1.4.3                                      | 89.0 MB    | Node 26.12 would be 150.3 MB, 116.9 MB stripped                                                                                                                                                |
| Remotion compositor                            | 27.3 MB    | unchanged, used by `renderMedia`                                                                                                                                                               |
| `@remotion/renderer` and the sidecar's own JS  | about 3 MB | `@remotion/renderer` is 3.0 MB                                                                                                                                                                 |
| Prebuilt Remotion bundle                       | 67 MB      | unchanged; shrinks once `asset-preparation.md` takes the font and 3D parsers out                                                                                                               |
| Chrome Headless Shell                          | 247.0 MB   | unchanged                                                                                                                                                                                      |
| FFmpeg                                         | 159.5 MB   | unchanged (out of scope)                                                                                                                                                                       |

Gone: Electron (286.8 MB) and `playwright-core` (13.7 MB). Added: Bun (89.0 MB) and the Rust binary. Without the Tauri binary, the listed parts come to about 593 MB; with it unstripped, about 686 MB.

## Work

1. **Measure each build.** Record the Tauri release binary, the deb, rpm, NSIS and macOS bundle sizes per platform in this file when phase 8 first runs in CI.
2. **Sidecar as one file.** Measure `bun build --compile` of the sidecar (`sidecar.md`, Work step 5) against Bun plus a JS folder.

## Risks

- **The bundle is large.** 67 MB includes `public/` copied in and every lazily loaded element chunk; it is shipped once, but check what dominates it before trimming.
- **Two Chromium builds are gone, one stays.** Electron's Chromium leaves; the headless shell stays because capture and renders need a real Chromium.

## Sources

- `du -sb` and `stat` on this machine, 2026-10-25 (Node `v26.12.0` binary, Bun `1.4.3`, the dev engine in `~/.config/Opsis/engine`).
- The bundle was built with `bundleRemotionProject` into `~/.cache/opsis-bun-test/bundle` (2026-10-25).
