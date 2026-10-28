# Image and sound nodes

One node that makes sound: a riser that ends on a mark.

Status: plan only. Nothing here is built yet. Opsis is licensed under the GPL (version 3 or later), so GPL code may be used as well as MIT, BSD, ISC, Zlib, MPL-2.0 and Apache-2.0 code, except inside what is combined with Remotion (`packages/video`, `apps/preview`, `apps/sidecar`), where the Remotion permission in `LICENSE` covers only Opsis's own code. None of these nodes needs GPL code there.

## How to read this plan

- The section is one node. It uses Generated sound's sounds (`audio.generate`: its presets and variations, `sound_shape`, made by `crates/media/src/sound_maker.rs`).
- Each section has **What it adds**, **In place**, **Work** (numbered), **Licence** and, where there are any, **Risks**. Rules from `AGENTS.md` hold throughout; when the two disagree, `AGENTS.md` wins.
- Sizes are rough: small (a few files, one crate and its card or widget), medium (several crates or the editor and Rust together), large (a new way of working through a whole crate).

## Conventions

- **Image nodes** follow the seven in place (`crates/compile/src/nodes/image.rs`, executors in `crates/image/src/executors.rs`, operations in `crates/image/src/ops.rs`, loading in `crates/image/src/load.rs`): file ports in, an `image` out, every picture loaded upright and in sRGB, kept at 16 bits with transparency, PNG between nodes. The sizes a node makes are pure functions shared by the executor and the card. Their Inspector picture is the connected input, and areas are drawn on it with the `boxes` widget (fractions of the picture), as Crop image does.
- **Sound file nodes** follow Even out volume (`media.loudnorm`, `one_file_op` in `crates/compile/src/nodes/media.rs`, its executor in `crates/media/src/executors.rs`): a `media` in, a new file out ("Makes a new file"), FFmpeg arguments built by pure functions tested by string. Every filter a node uses joins `REQUIRED_FILTERS` (`crates/media/src/capabilities.rs`); a test fails on a filter the list lacks.
- **Sound making nodes** output a `sound` (`AudioValue`) that Add sound takes. Sounds made from settings are rendered once, when the node runs, into its folder (`hound`), never at preview or render time, so the preview and the export play the same file. They play on the node's own Mixer channel on the Effects bus unless the section says otherwise.
- New settings are `schemars` fields with their label, limits and `x-` keywords; the editor builds the form from them. Technical settings go under `x-advanced`.
- A new node has a `description`, a `card`, a look on its definition (`.look(…)`, then `bun run glyphs:bundle`) and passes `crates/compile/tests/unit/nodes_c.rs`.
- Copy names no library, tool, filter or format. Durations in seconds.
- A new built-in sound, impulse response or picture needs a checked free licence, an entry in `THIRD_PARTY_NOTICES.md` and a local file; nothing loads from the internet.
- New crates go in with `cargo add`; their licences go in `THIRD_PARTY_NOTICES.md` and Settings → Licences.
- Checks for every section: `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`, the no-comments check, then the app itself (`check-app`).

## Summary

| Node               | What users get                                      | Built with      | Licence of what's added | Size  |
| ------------------ | --------------------------------------------------- | --------------- | ----------------------- | ----- |
| 1. Riser to a mark | A build-up that ends exactly on an action or marker | Generated sound | none new                | small |

Left out: content-aware resize (`imageproc::seam_carving` is called rough by its own documentation), speech made from text and captions from speech (`plans/voice-and-captions.md` covers them).

## 1. Riser to a mark

### What it adds

A build-up sound that ends exactly on a chosen action, marker or the end of a clip, for reveals and launches.

### Work

1. **Settings.** Ends on (action, marker, clip end), length before it, preset (Generated sound's riser or swell, with its variation), with a short hit at the end on or off.
2. **Compile.** Find the target frame (actions through `anchor_sounds`, markers by id), start the sound its length before; render it like Generated sound.
3. **Card.** "Rises 3 s into “Launch”".
4. **Tests.** The sound ends on the target frame.

### Licence

Nothing new.

## Sources

- [imageproc 0.27 modules](https://docs.rs/imageproc/0.27.0/imageproc/)
- [Xnapper: annotate screenshot](https://xnapper.com/blog/annotate-screenshot), [Xnapper review](https://daveswift.com/xnapper/), [CleanShot X vs Xnapper](https://thesweetbits.com/cleanshot-vs-xnapper/), [Better Shot](https://github.com/KartikLabhshetwar/better-shot)
- [Descript clean audio tool](https://www.descript.com/tools/clean-audio-tool), [Best AI audio enhancement tools](https://dupple.com/learn/best-ai-audio-enhancement-tools), [Cleanvoice: podcast APIs](https://cleanvoice.ai/blog/best-podcast-api/)
- crates.io: [fundsp](https://crates.io/crates/fundsp), [nnnoiseless](https://crates.io/crates/nnnoiseless), [rubato](https://crates.io/crates/rubato)
- The audio filters listed were checked present in the pinned FFmpeg 7.0.2 static build (`ffmpeg -filters`): `afftdn`, `arnndn`, `anlmdn`, `adeclick`, `adeclip`, `silenceremove`, `silencedetect`, `deesser`, `speechnorm`, `dynaudnorm`, `rubberband`, `afir`, `aecho`, `acrusher`, `compand`, `acrossfade`, `showwavespic`, `showspectrumpic`, `sine`, `anoisesrc`, `aevalsrc`, `atempo`, `asetrate`, `equalizer`, `highpass`, `lowpass`.
