# Image and sound nodes

Nine nodes for stills and sound: three that work on images (annotate, magnify, compare), two that change a sound file (pitch, voice effects) and four that make sound (transition sounds, generated sounds, risers, music fitted to the film).

Status: plan only. Nothing here is built yet. Opsis is licensed under the GPL (version 3 or later), so GPL code may be used as well as MIT, BSD, ISC, Zlib, MPL-2.0 and Apache-2.0 code, except inside what is combined with Remotion (`packages/video`, `apps/preview`, `apps/sidecar`), where the Remotion permission in `LICENSE` covers only Opsis's own code. None of these nodes needs GPL code there.

## How to read this plan

- Each section is one node and can be built on its own, except where a section names another it needs (sections 7 and 8 share section 6's sound maker).
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

| Node                     | What users get                                                      | Built with                             | Licence of what's added    | Size   |
| ------------------------ | ------------------------------------------------------------------- | -------------------------------------- | -------------------------- | ------ |
| 1. Annotate              | Arrows, boxes, ovals, numbered steps and text on a screenshot       | `imageproc` drawing, `ab_glyph` text   | none new                   | medium |
| 2. Magnify               | An enlarged circle or box of one area, joined to the spot by a line | `image`, `imageproc` drawing           | none new                   | small  |
| 3. Compare screenshots   | What changed between two captures, boxed                            | `imageproc` threshold, labelling, hash | none new                   | medium |
| 4. Change pitch          | Pitch raised or lowered, length kept                                | FFmpeg                                 | none new                   | small  |
| 5. Voice effects         | Telephone, radio, room and echo                                     | FFmpeg, built-in room recordings       | room recordings CC0        | medium |
| 6. Generated sound       | Whooshes, risers, pops, dings and swells made from settings         | `fundsp`                               | `fundsp` MIT OR Apache-2.0 | medium |
| 7. Transition sounds     | A whoosh at every transition of a clip                              | section 6, flattened transitions       | none new                   | medium |
| 8. Riser to a mark       | A build-up that ends exactly on an action or marker                 | section 6                              | none new                   | small  |
| 9. Fit music to the film | A song stretched or looped to end with the film                     | FFmpeg                                 | none new                   | medium |

Left out: content-aware resize (`imageproc::seam_carving` is called rough by its own documentation), speech made from text and captions from speech (`plans/voice-and-captions.md` covers them).

## 1. Annotate

### What it adds

Arrows, rectangles, ovals, numbered step badges and text drawn on a screenshot, each in the project's colours and fonts, so a still explains itself. Screenshot tools lead with this (Xnapper's text, arrows, rectangles and ovals; Better Shot's numbered labels).

### In place

- The `boxes` widget draws areas over an image (Crop image's **Area to keep**).
- The video's annotation nodes (Tooltip, Callout arrow, Highlight box) draw on recordings, not on image files.
- Project fonts are files in `assets/fonts/<id>/`, read by `opsis-fonts` with `skrifa`.

### Work

1. **Settings.** A list of marks (rows): kind (arrow, rectangle, oval, step number, text), its area or two ends (`boxes`), colour (`color`, project swatches), line width, text, font (`font` widget), text size. Step numbers count up in list order.
2. **Drawing.** `imageproc::drawing`: thick lines and arrow heads as filled polygons, hollow rectangles and ellipses drawn several times for width, filled circles for steps; anti-aliased with `draw_antialiased_line_segment` and `draw_polygon` at 2× then scaled down, like the sizes in place.
3. **Text.** Drawn as Before and after draws its labels: the face is picked like 3D text picks it (`family_for`, `face_for`), loaded at the chosen weight with WOFF and WOFF2 unwrapped (`load_font`), measured and drawn with `ab_glyph` (`measure`, `draw_text` in `crates/image/src/text.rs`), and the node keys on the project fonts (`uses_project_fonts`).
4. **Card.** "3 marks · 2 steps"; a warning when a mark's text is empty.
5. **Tests.** Pixel tests of each mark kind on a blank image against golden PNGs in `fixtures/image/`.

### Licence

Nothing new.

### Risks

- Fonts with variable axes draw at their default instance unless the weight is applied; draw at the chosen weight or limit the font field to static faces.

## 2. Magnify

### What it adds

One area of a still shown enlarged in a circle or rounded box with a border, placed beside it and joined to the spot by a line, so small text reads in a post.

### In place

- `boxes`; Resize image's resampling (`fast_image_resize`).

### Work

1. **Settings.** Area to enlarge (`boxes`), where to show it (`boxes`), shape (circle, rounded box), zoom, border colour and width, connecting line on or off.
2. **Operation.** Crop, resize with `fast_image_resize`, mask to the shape, draw the border and the line (`drawing`), overlay.
3. **Card.** "2× · circle".
4. **Tests.** The enlarged area's pixels match the source area scaled.

### Licence

Nothing new.

## 3. Compare screenshots

### What it adds

The areas that changed between two captures of the same page, boxed on the newer one, plus an image of just the differences, for "what's new" stills and for checking that a page didn't change.

### Work

1. **Ports.** **Before** and **After** files; outputs: the marked image and a `file info` with the changed-area count.
2. **Settings.** Sensitivity, smallest area to mark, box colour and width, padding around each change.
3. **Operation.** Align sizes, per-pixel difference, `contrast::threshold`, `morphology::dilate` to join nearby changes, `region_labelling::connected_components` to find areas, bounding boxes drawn with `drawing`. `image_hash` gives a quick "unchanged" answer.
4. **Card.** "5 changes" or "No change".
5. **Tests.** Two fixtures that differ in known rectangles give those rectangles.

### Risks

- Captures of different sizes or scroll positions differ everywhere; refuse different sizes with a clear message.

### Licence

Nothing new.

## 4. Change pitch

### What it adds

A sound's pitch raised or lowered in semitones while its length stays, to fit a song's key or make a playful voice.

### Work

1. **Settings.** Semitones, keep the voice character (formants) on or off.
2. **Operation.** `rubberband=pitch=…:formant=…`; without it in a user's FFmpeg, `asetrate` then `atempo` back to length (character not kept).
3. **Card.** "+3 semitones".
4. **Tests.** Arguments by string; output length equals input length within a frame.

### Licence

The pitch library is inside the FFmpeg the user downloads; nothing is added to Opsis.

### Risks

- `rubberband` is in the pinned FFmpeg build but not in every FFmpeg a user may choose; check for it like the other required filters, or make it optional with the fallback above.

## 5. Voice effects

### What it adds

A voice or sound made to sound like a telephone, a radio, a small room, a hall or an echo.

### Work

1. **Settings.** Effect (telephone, radio, small room, hall, echo), mix amount.
2. **Telephone and radio.** `highpass`, `lowpass`, a little `acrusher` and `compand`.
3. **Rooms.** `afir` with a short impulse response recording per room, shipped as WAV files in `public/ir`, each CC0 and checked on its page.
4. **Echo.** `aecho` with delay and decay from the settings.
5. **Card.** The effect and mix.
6. **Tests.** Filter strings per effect; the room recordings exist and are listed with credits.

### Licence

The impulse responses must be CC0 or compatible and credited in `THIRD_PARTY_NOTICES.md`.

## 6. Generated sound

### What it adds

Whooshes, risers, pops, dings, clicks and swells made from a few settings (length, pitch, brightness, variation seed), so every film gets fresh, matching sounds with no file to find and no licence to check.

### In place

- Built-in sounds are seven CC0 files (`crates/compile/src/data/sounds.json`); Sound effect places one at a time or on an action.

### Work

1. **Engine.** `fundsp` graphs per preset: noise through a swept band-pass with an envelope (whoosh, riser), a short sine burst with pitch drop (pop), bell partials (ding), filtered noise click, slow pad swell. One pure function per preset, seeded.
2. **Node.** `audio.generate`: preset, length (seconds), pitch, brightness, seed; output `sound` like Sound effect, with time or **On an action** placement shared with Sound effect.
3. **Rendering.** The executor renders 48 kHz stereo WAV with `hound` into the node's folder; the cache key is the settings, so the same settings never render twice.
4. **Card.** The preset and length, with a play button in the Inspector after a run.
5. **Tests.** Same seed gives the same samples; length and peak level per preset.

### Licence

`fundsp`: MIT OR Apache-2.0.

### Risks

- Sound quality depends on the presets' tuning; ship few, good ones and listen to each before adding it.

## 7. Transition sounds

### What it adds

A whoosh at every transition in a clip, timed to each transition's middle and scaled to its length, as one Mixer channel, the way Action sounds puts a click at every click.

### In place

- Transition marks come from `flatten_spec` (`TransitionMark` with `from`, `to`, `kind`, `crates/compile/src/timeline/flatten.rs`).
- Action sounds (`audio.actionSounds`) shows how a node turns a clip's moments into hits on one channel, drawn as one Timeline bar.

### Work

1. **Node.** `audio.transitionSounds`: input `clip`, output `sound`; settings: sound (built-in or section 6's whoosh), volume, vary each hit, skip plain cuts.
2. **Compile.** Flatten the clip, one hit per transition mark at its middle, length from the transition's; hits flagged like Action sounds' so the Timeline shows one bar.
3. **Card.** "6 transitions · Whoosh".
4. **Tests.** Hits land at the marks' middles through a trim and a speed change.

### Licence

Nothing new.

## 8. Riser to a mark

### What it adds

A build-up sound that ends exactly on a chosen action, marker or the end of a clip, for reveals and launches.

### Work

1. **Settings.** Ends on (action, marker, clip end), length before it, preset (section 6's riser, swell), with a short hit at the end on or off.
2. **Compile.** Find the target frame (actions through `anchor_sounds`, markers by id), start the sound its length before; render via section 6.
3. **Card.** "Rises 3 s into “Launch”".
4. **Tests.** The sound ends on the target frame.

### Licence

Nothing new.

## 9. Fit music to the film

### What it adds

A song made to end with the film: stretched slightly in tempo, or looped at a chosen point with a crossfade and faded at the end, so the music doesn't cut off or stop early.

### In place

- Music (`audio.music`) loops a file from its start and fades at the end of the clip.
- Built-in music has a length (`music.json`).

### Work

1. **Settings.** Way (stretch, loop, both), largest stretch allowed (percent), loop from and to (seconds), crossfade length, ending fade.
2. **Operation.** Stretch with `rubberband=tempo=` (fallback `atempo`) when the needed change is within the limit; otherwise loop the chosen section with `acrossfade` until the length is reached, then fade. The target length comes from the connected clip.
3. **Node.** Input `clip` (for the length) and `media`; output a new file that Music plays.
4. **Card.** "Stretched 4%" or "Looped twice".
5. **Tests.** Output length equals the clip's within a frame for each way.

### Licence

Nothing new.

## Sources

- [imageproc 0.27 modules](https://docs.rs/imageproc/0.27.0/imageproc/)
- [Xnapper: annotate screenshot](https://xnapper.com/blog/annotate-screenshot), [Xnapper review](https://daveswift.com/xnapper/), [CleanShot X vs Xnapper](https://thesweetbits.com/cleanshot-vs-xnapper/), [Better Shot](https://github.com/KartikLabhshetwar/better-shot)
- [Descript clean audio tool](https://www.descript.com/tools/clean-audio-tool), [Best AI audio enhancement tools](https://dupple.com/learn/best-ai-audio-enhancement-tools), [Cleanvoice: podcast APIs](https://cleanvoice.ai/blog/best-podcast-api/)
- crates.io: [fundsp](https://crates.io/crates/fundsp), [nnnoiseless](https://crates.io/crates/nnnoiseless), [rubato](https://crates.io/crates/rubato)
- The audio filters listed were checked present in the pinned FFmpeg 7.0.2 static build (`ffmpeg -filters`): `afftdn`, `arnndn`, `anlmdn`, `adeclick`, `adeclip`, `silenceremove`, `silencedetect`, `deesser`, `speechnorm`, `dynaudnorm`, `rubberband`, `afir`, `aecho`, `acrusher`, `compand`, `acrossfade`, `showwavespic`, `showspectrumpic`, `sine`, `anoisesrc`, `aevalsrc`, `atempo`, `asetrate`, `equalizer`, `highpass`, `lowpass`.
