# Image and sound nodes

Twenty-two nodes for stills and sound: nine that work on images (annotate, frame, magnify, grid, before and after, compare, outline, tilt, auto levels), seven that clean or change a sound file (clean speech, shorten pauses, clicks and clipping, hum, pitch, voice effects, sound picture) and six that make sound (transition sounds, generated sounds, risers, room tone, tones, music fitted to the film).

Status: plan only. Nothing here is built yet. Opsis is licensed under the GPL (version 3 or later), so GPL code may be used as well as MIT, BSD, ISC, Zlib, MPL-2.0 and Apache-2.0 code, except inside what is combined with Remotion (`packages/video`, `apps/preview`, `apps/sidecar`), where the Remotion permission in `LICENSE` covers only Opsis's own code. None of these nodes needs GPL code there.

## How to read this plan

- Each section is one node and can be built on its own, except where a section names another it needs (sections 4 and 5 use the growing inputs that Sequence and Layers already have; sections 18 and 19 share section 17's sound maker).
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

| Node                          | What users get                                                      | Built with                             | Licence of what's added    | Size   |
| ----------------------------- | ------------------------------------------------------------------- | -------------------------------------- | -------------------------- | ------ |
| 1. Annotate                   | Arrows, boxes, ovals, numbered steps and text on a screenshot       | `imageproc` drawing, `ab_glyph` text   | `ab_glyph` Apache-2.0      | medium |
| 2. Frame screenshot           | Rounded corners, a soft shadow and padding on a backdrop            | `image`, `imageproc` blur              | none new                   | small  |
| 3. Magnify                    | An enlarged circle or box of one area, joined to the spot by a line | `image`, `imageproc` drawing           | none new                   | small  |
| 4. Grid                       | Any number of images in a row, column or grid                       | `imageproc::compose`                   | none new                   | small  |
| 5. Before and after           | Two pictures side by side or split by a divider                     | `image`                                | none new                   | small  |
| 6. Compare screenshots        | What changed between two captures, boxed                            | `imageproc` threshold, labelling, hash | none new                   | medium |
| 7. Outline                    | A sticker outline around a cutout                                   | `imageproc::morphology`                | none new                   | small  |
| 8. Tilt                       | A screenshot on a tilted plane                                      | `imageproc` projective warp            | none new                   | small  |
| 9. Auto levels                | A dull capture's contrast stretched                                 | `imageproc::contrast`                  | none new                   | small  |
| 10. Clean speech              | Noise, harsh "s" sounds and rumble removed, level evened            | `nnnoiseless`, FFmpeg                  | `nnnoiseless` BSD-3-Clause | medium |
| 11. Shorten pauses            | Long silences in a voice file tightened                             | FFmpeg                                 | none new                   | small  |
| 12. Fix clicks and clipping   | Clicks removed and clipped peaks rebuilt                            | FFmpeg                                 | none new                   | small  |
| 13. Remove hum                | Mains hum and its harmonics removed                                 | FFmpeg                                 | none new                   | small  |
| 14. Change pitch              | Pitch raised or lowered, length kept                                | FFmpeg                                 | none new                   | small  |
| 15. Voice effects             | Telephone, radio, room and echo                                     | FFmpeg, built-in room recordings       | room recordings CC0        | medium |
| 16. Sound picture             | A sound's waveform or spectrum as an image                          | FFmpeg                                 | none new                   | small  |
| 17. Generated sound           | Whooshes, risers, pops, dings and swells made from settings         | `fundsp`                               | `fundsp` MIT OR Apache-2.0 | medium |
| 18. Transition sounds         | A whoosh at every transition of a clip                              | section 17, flattened transitions      | none new                   | medium |
| 19. Riser to a mark           | A build-up that ends exactly on an action or marker                 | section 17                             | none new                   | small  |
| 20. Room tone                 | A quiet noise bed under a clip                                      | FFmpeg or `fundsp`                     | none new                   | small  |
| 21. Tones and countdown beeps | Tones, beeps and countdowns                                         | FFmpeg or `fundsp`                     | none new                   | small  |
| 22. Fit music to the film     | A song stretched or looped to end with the film                     | FFmpeg                                 | none new                   | medium |

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
3. **Text.** `ab_glyph` (a new dependency; `imageproc::drawing::draw_text` takes an `ab_glyph` font) loads the chosen project or bundled font file; WOFF2 is unwrapped with `wuff` first, as the 3D text preparer does.
4. **Card.** "3 marks · 2 steps"; a warning when a mark's text is empty.
5. **Tests.** Pixel tests of each mark kind on a blank image against golden PNGs in `fixtures/image/`.

### Licence

`ab_glyph`: Apache-2.0, compatible with the GPL version 3.

### Risks

- Fonts with variable axes draw at their default instance unless the weight is applied; draw at the chosen weight or limit the font field to static faces.

## 2. Frame screenshot

### What it adds

A screenshot with rounded corners and a soft drop shadow, padded on a solid colour, a gradient or one of the built-in backdrops, ready for a post. Xnapper does this automatically; Better Shot offers shadow, corner and background controls.

### In place

- Backdrops in `public/backdrops` (`Backdrop` in `crates/compile/src/backdrops.rs`), the `gradient` and `backdrop` widgets.
- Place image puts one picture on another.

### Work

1. **Settings.** Background (`backdrop`, colour or `gradient`), padding (percent of the shorter side), corner radius, shadow size, softness and opacity, output shape (keep, 16:9, 1:1, 4:5, 9:16).
2. **Operation.** Mask the picture's corners with anti-aliased quarter circles; draw the shadow as a blurred, offset alpha mask (`gaussian_blur_f32`); render the gradient by its stops; composite with `image::imageops::overlay`.
3. **Card.** The background and the output size.
4. **Tests.** Output size per shape; a corner pixel is the background; the shadow darkens below the picture.

### Licence

Nothing new; the backdrops are already credited.

## 3. Magnify

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

## 4. Grid

### What it adds

Any number of images joined in a row, a column or a grid with gaps and a background, for "three steps" posts and changelog collages.

### In place

- Growing inputs (`PortFamily`, `crates/compile/src/node.rs`): Join files already takes any number of files.

### Work

1. **Ports.** A file family (`in1`, `in2`, … with one empty input after the last), two required.
2. **Settings.** Layout (row, column, grid with a column count), gap, padding, background colour, how pictures of different sizes fit (same height, same width, fit in equal cells).
3. **Operation.** Pure layout function (cells and scales, shared with the card) and `imageproc::compose`/`image::imageops::overlay`.
4. **Card.** "4 images · 2 × 2 · 1920 × 1080 px".
5. **Tests.** Layout sizes for each mode; pixels of each cell come from its input.

### Licence

Nothing new.

## 5. Before and after

### What it adds

Two pictures side by side, stacked, or one picture split by a divider at a chosen position with "Before" and "After" labels, for redesign and changelog posts.

### Work

1. **Ports.** Two files, **Before** and **After**.
2. **Settings.** Layout (side by side, stacked, split), split position and angle, divider colour and width, labels on or off and their text, font and colour.
3. **Operation.** Fit both to one size, compose, draw the divider and labels (section 1's text drawing).
4. **Card.** The layout and the size.
5. **Tests.** Split position maps to the right columns.

### Licence

Nothing new.

## 6. Compare screenshots

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

## 7. Outline

### What it adds

A coloured outline around the visible part of a transparent picture, a sticker look for cutouts and logos.

### Work

1. **Settings.** Width, colour, softness, shadow on or off.
2. **Operation.** Grow the alpha mask with `morphology::dilate` (or `distance_transform` for wide outlines), colour it, put the picture on top.
3. **Card.** "8 px white outline".
4. **Tests.** The outline appears only where the original was transparent.

### Risks

- A picture with no transparency gets an outline only around the frame; warn on the card when the input has no transparency.

### Licence

Nothing new.

## 8. Tilt

### What it adds

A screenshot placed on a tilted plane with perspective, for a mock-up still without a 3D render.

### Work

1. **Settings.** Turn, tilt and lean (degrees), background, shadow.
2. **Operation.** Corner positions from the angles (pure, tested), `geometric_transformations::Projection::from_control_points` and `warp_into` with bilinear sampling.
3. **Card.** The angles and the size.
4. **Tests.** Corners land where the function says.

### Licence

Nothing new.

## 9. Auto levels

### What it adds

A dull or washed-out capture's contrast and levels stretched, the still counterpart of the grade's Auto fix.

### Work

1. **Settings.** Strength, clip percent of darkest and lightest pixels, per channel or together.
2. **Operation.** Histograms (`stats::histogram`), `contrast::stretch_contrast` per channel or on lightness; `equalize_histogram` as a stronger choice.
3. **Card.** "Levels stretched".
4. **Tests.** Output histogram spans the range.

### Licence

Nothing new.

## 10. Clean speech

### What it adds

A voice recording cleaned in one step: background noise removed, rumble cut, harsh "s" sounds softened and the level evened, like Descript's Studio Sound or Auphonic's single pass. For a narration recorded at a desk.

### In place

- Even out volume (`media.loudnorm`) and the Mixer's EQ, compressor and limiter work on the whole channel, not on noise.
- FFmpeg has `afftdn` (spectral noise reduction), `highpass`, `deesser`, `speechnorm` and `dynaudnorm`.

### Work

1. **Settings.** Noise removal strength, de-ess amount, level evening on or off, method under `x-advanced` (learned noise removal or spectral).
2. **Noise.** Learned method: decode to 48 kHz mono 16-bit with FFmpeg, run `nnnoiseless` (RNNoise in Rust, its trained weights built into the crate, so nothing to download) frame by frame, write with `hound`. Spectral method: `afftdn`.
3. **The rest.** One FFmpeg pass: `highpass=f=80`, `deesser`, `speechnorm` (or `dynaudnorm`), copying stereo back when the input was stereo.
4. **Card.** The steps on, and the noise level before and after (measured with `astats`).
5. **Tests.** Argument builders by string; a fixture of a tone in noise comes out with less noise power (measured by `astats`).

### Licence

`nnnoiseless`: BSD-3-Clause, compatible with the GPL.

### Risks

- The learned method works on 48 kHz mono; music under the voice is treated as noise. Say on the card it's for speech.

## 11. Shorten pauses

### What it adds

Long silences in a voice file shortened to a set length, so a slow narration tightens without cutting words, like Cleanvoice's silence remover.

### Work

1. **Settings.** Silence threshold, longest pause to keep, shortest pause to touch.
2. **Operation.** `silencedetect` lists the pauses; pure code turns them into keep ranges (each pause trimmed to the longest allowed, centred); FFmpeg `atrim`/`asetpts`/`concat` joins the ranges with 10 ms crossfades.
3. **Card.** "Saved 12.4 s from 9 pauses".
4. **Tests.** Keep ranges from a silence list; a fixture with known gaps comes out the expected length.

### Risks

- A voice already placed against a take loses sync; the description says it's for a voice file before it's placed.

### Licence

Nothing new.

## 12. Fix clicks and clipping

### What it adds

Clicks and pops removed and clipped peaks rebuilt in a recording.

### Work

1. **Settings.** Clicks on or off, clipping on or off, strength.
2. **Operation.** `adeclick` and `adeclip` in one pass.
3. **Card.** The fixes on.
4. **Tests.** Arguments by string; a sine with an inserted spike loses the spike.

### Licence

Nothing new.

## 13. Remove hum

### What it adds

Mains hum at 50 or 60 Hz and its harmonics removed from a recording.

### Work

1. **Settings.** Mains frequency (50, 60, find it), harmonics to remove, notch width.
2. **Operation.** A chain of narrow `equalizer` cuts at the frequency and its multiples; "find it" measures the energy near 50 and 60 Hz with a short `astats`/FFT pass first.
3. **Card.** "50 Hz and 4 harmonics".
4. **Tests.** Filter string; a 50 Hz tone plus voice loses the tone.

### Licence

Nothing new.

## 14. Change pitch

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

## 15. Voice effects

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

## 16. Sound picture

### What it adds

A sound's waveform or spectrum drawn as an image, for a podcast post or a still that shows the voice.

### Work

1. **Settings.** Kind (waveform, spectrum), size, colours (`color`), background.
2. **Operation.** `showwavespic` or `showspectrumpic` to a PNG; the output is an `image` so the image nodes take it.
3. **Card.** The kind and size.
4. **Tests.** Arguments by string; the PNG has the asked size.

### Licence

Nothing new.

## 17. Generated sound

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

## 18. Transition sounds

### What it adds

A whoosh at every transition in a clip, timed to each transition's middle and scaled to its length, as one Mixer channel, the way Action sounds puts a click at every click.

### In place

- Transition marks come from `flatten_spec` (`TransitionMark` with `from`, `to`, `kind`, `crates/compile/src/timeline/flatten.rs`).
- Action sounds (`audio.actionSounds`) shows how a node turns a clip's moments into hits on one channel, drawn as one Timeline bar.

### Work

1. **Node.** `audio.transitionSounds`: input `clip`, output `sound`; settings: sound (built-in or section 17's whoosh), volume, vary each hit, skip plain cuts.
2. **Compile.** Flatten the clip, one hit per transition mark at its middle, length from the transition's; hits flagged like Action sounds' so the Timeline shows one bar.
3. **Card.** "6 transitions · Whoosh".
4. **Tests.** Hits land at the marks' middles through a trim and a speed change.

### Licence

Nothing new.

## 19. Riser to a mark

### What it adds

A build-up sound that ends exactly on a chosen action, marker or the end of a clip, for reveals and launches.

### Work

1. **Settings.** Ends on (action, marker, clip end), length before it, preset (section 17's riser, swell), with a short hit at the end on or off.
2. **Compile.** Find the target frame (actions through `anchor_sounds`, markers by id), start the sound its length before; render via section 17.
3. **Card.** "Rises 3 s into “Launch”".
4. **Tests.** The sound ends on the target frame.

### Licence

Nothing new.

## 20. Room tone

### What it adds

A quiet bed of soft noise under a clip, so cuts between takes and silent stretches don't drop to dead silence.

### Work

1. **Settings.** Colour of noise (pink, brown, white), level, fade in and out.
2. **Operation.** `anoisesrc` rendered once to a looping WAV (or `fundsp` noise), placed as a looped sound the length of the clip on a Music-like channel that never keys the ducking.
3. **Card.** "Pink noise · −45 dB".
4. **Tests.** The level matches the setting.

### Licence

Nothing new.

## 21. Tones and countdown beeps

### What it adds

A tone, a beep or a countdown (three short beeps and a long one) at a time or on an action, for timers and "go" moments.

### Work

1. **Settings.** Kind (tone, beep, countdown), pitch, length, count.
2. **Operation.** `sine` (or `fundsp`) rendered to WAV; countdown spacing one second unless set.
3. **Card.** "Countdown · 3 beeps".
4. **Tests.** Beep count and spacing from the rendered samples.

### Licence

Nothing new.

## 22. Fit music to the film

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
