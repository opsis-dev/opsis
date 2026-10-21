# Colour grade

A second way to change how a finished video looks, for when Picture look isn't enough. Small grade nodes build a list of grading steps; one **Apply grade** node runs the whole list over a video or image file in a single FFmpeg pass and writes a new file.

Status: plan only. Nothing here is built yet.

## Why

**Picture look** (`effect.look`, Effects) stays exactly as it is: the fast, live path. It runs `@remotion/effects` shaders on the picture components of a clip (a recording's page footage, a video, an image), so it previews instantly. On a recording it changes the page footage only, not the window frame, backdrop, cursor or marks, because the effects attach to the page `<Video>`, the only effect-capable component in a recording clip.

**Apply grade** is the explicit fallback. It takes a file (an export, an imported video or image) and grades the whole picture as it is in that file. Nothing changes until it runs, and its result is a new file that goes on like any other (Use as clip, Convert, Export video…).

The steps are separate nodes passed forward as a list, as recording styles are, and not separate file → file nodes: every file node is a full decode and encode, so three file nodes in a row would encode the film three times, take three times as long, leave three files and lose quality on every pass. One list and one pass encodes once, and the order of the steps is the order of the chain, which matters (a look file before or after a contrast change gives different results).

## Conventions

These restate rules from `AGENTS.md`; when the two disagree, `AGENTS.md` wins.

- No comments; kebab-case files; nodes never generate Remotion code; executors are keyed by node type and every `executes: true` node has one (tested).
- User-facing copy names no tool or format ("FFmpeg", "LUT", "CRF", "eq", "curves"…): plain words such as "Look file", "Colour for shadows". Sentence case, durations in seconds, technical settings under `advanced`.
- Every node has a `description`, a `card()` of up to four rows with icons from `NODE_ICON_NAMES`, and a look in `NODE_LOOKS` (stroke-only glyph, run `bun run glyphs:bundle`). Two nodes share a glyph only if they move differently.
- Nothing loads from the internet. Look files are project files (`asset` widget), and their size and modified time reach the cache key through `fileStamps`.
- No version bumps before release (`ENGINE_VERSION`, `PICTURE_VERSION`, `MIX_VERSION`); clear test caches instead.
- Checks: `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`, the no-comments check, and the app run described in **Checks**.

## The grade resource

- **Port type** `grade`, added to `PORT_TYPES` (`src/shared/schema/workflow.ts`) with `PORT_WORDS.grade = "grade"`. No conversions in `CONVERT` (`src/shared/nodes/ports.ts`): only grade nodes make a grade.
- **Value** in `src/shared/schema/values.ts`, next to `recordingStyleValueSchema`:

  ```ts
  const gradeValueSchema = z.object({
    type: z.literal("grade"),
    entries: z.array(styleEntrySchema),
  });
  ```

  Added to `compileValueSchema`. Entries are `{ nodeId, type, params }`, the same shape as style entries.

- **Port colour** `PORT_COLORS.grade` in `src/renderer/editor/ports.ts`: the new category's colour.
- **Base** in `src/shared/nodes/define.ts`: `gradeModifier = { ...nodeBase("Grade", "compile"), inputs: { grade: opt("grade", "Grade") }, outputs: { grade: port("grade", "Grade") }, compile: appendGrade }`, where `appendGrade` appends `entryOf(ctx)` exactly like `appendStyle`. A skipped grade node passes its input through, so it never enters the list.

## The new category

- `NODE_CATEGORIES` gains `"Grade"` (after `"Effects"`), `CATEGORY_LABELS.Grade = "Colour grade"`.
- `CATEGORY_STYLES.Grade` in `src/renderer/editor/ports.ts`: its own colour (for example `#fb7185`, distinct from Effects and Output) and icon `palette`.
- Anything else keyed by `NodeCategory` (the type check lists them: the timeline bar tones in `src/renderer/editor/timeline/video-bar.tsx`, the collapsed node list's category rail) gets an entry.
- The six grade nodes live in a new `src/shared/nodes/grade.ts`, registered in `src/shared/nodes/registry.ts`.

## Grade nodes

Every grade node is `grade? → grade`. Each setting that is at its neutral value adds no filter, so a node left at its defaults changes nothing. Each card lists the settings that change the picture, like Picture look's card ("Warm · 3 changes"); a node whose settings are all neutral shows "No change" muted.

The FFmpeg column is what `Apply grade` writes for that setting (see **Filter chain**). Ranges are the node's; the mapping turns them into the filter's own units.

### 1. Colour and light — `grade.colour`

| Setting                                    | Range, default                                                                                | FFmpeg                                                                                                                                             |
| ------------------------------------------ | --------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| Brightness                                 | −1…1, 0                                                                                       | `eq=brightness=B`                                                                                                                                  |
| Contrast                                   | 0.5…1.5 ×, 1                                                                                  | `eq=contrast=C`                                                                                                                                    |
| Colour (saturation)                        | 0…2 ×, 1                                                                                      | `eq=saturation=S`                                                                                                                                  |
| Gamma                                      | 0.5…2, 1                                                                                      | `eq=gamma=G`                                                                                                                                       |
| Exposure                                   | −2…2 stops, 0                                                                                 | `exposure=exposure=E`                                                                                                                              |
| Warmth                                     | −1…1, 0                                                                                       | `colortemperature=temperature=T` with T = 6500 − warmth·3500 (warm is a lower colour temperature, so the picture is pushed towards amber), `mix=1` |
| Vibrance                                   | −1…1, 0                                                                                       | `vibrance=intensity=V`                                                                                                                             |
| Hue shift                                  | −180…180 °, 0                                                                                 | `hue=h=H`                                                                                                                                          |
| Colour for shadows / midtones / highlights | colour + amount 0…1 each, amount 0                                                            | `colorbalance=rs:gs:bs:rm:gm:bm:rh:gh:bh`, each offset = (channel − mean of the colour's three channels) × amount, clamped to −1…1                 |
| Tone curve                                 | None, Vintage, Cross-process, Lighter, Darker, More contrast, Strong contrast, Negative; None | `curves=preset=vintage\|cross_process\|lighter\|darker\|medium_contrast\|strong_contrast\|negative`                                                |

One `eq` filter carries all four of its settings. Order inside the node: `exposure`, `eq`, `colortemperature`, `vibrance`, `hue`, `colorbalance`, `curves`.

Presets on Brightness (`presets`, label "Look"), the same names and values as Picture look's (Natural, Warm, Cool, Punchy, Soft, Black and white, Old film without grain and dark edges, which belong to Texture).

### 2. Look file — `grade.lookFile`

| Setting   | Range, default                    | FFmpeg                                                                                                                       |
| --------- | --------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| Look file | `asset`, accept `.cube,.3dl,.png` | `.cube`/`.3dl`: `lut3d=file='<path>':interp=tetrahedral`; `.png` (a look image): `haldclut` with the image as a second input |
| Strength  | 0…1, 1                            | below 1: `split[a][b];[b]<look>[g];[g][a]blend=all_mode=normal:all_opacity=S`                                                |

- The help explains in plain words: "A colour look saved from another video editor, or downloaded. Files ending in .cube or .3dl, or a look image."
- A missing file is a card warning and fails Apply grade with "Choose a look file for “<node name>”."
- A look image goes in as `-loop 1 -i <png>` and `haldclut=repeatlast=1` reads it, so the image is not consumed after the first frame. The input index is assigned by the filter chain builder (see **Filter chain**).
- Paths are quoted for the filter graph (`'`, `\`, `:` escaped), never interpolated raw. Unit-test the escaping.
- The look file is read at run time only, never copied into the output.

### 3. Black and white and tint — `grade.mono`

| Setting         | Values, default                               | FFmpeg                                                                                                               |
| --------------- | --------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| Kind            | Black and white, Sepia, Tint; Black and white | below                                                                                                                |
| Black and white | —                                             | `monochrome=cb=0:cr=0:size=1`                                                                                        |
| Sepia           | —                                             | `colorchannelmixer=.393:.769:.189:0:.349:.686:.168:0:.272:.534:.131`                                                 |
| Tint colour     | colour, `#d8a15b`, shown for Tint             | `colorize=hue=<hue of the colour>:saturation=<its saturation>:lightness=0`                                           |
| Strength        | 0…1, 1                                        | `monochrome`: `size` is fixed and strength blends as in Look file; `colorize`: `mix=S`; sepia: blend as in Look file |

### 4. Texture — `grade.texture`

| Setting           | Range, default                                                                        | FFmpeg                                                    |
| ----------------- | ------------------------------------------------------------------------------------- | --------------------------------------------------------- |
| Dark edges        | 0…1, 0                                                                                | `vignette=angle=A:x0=w*X:y0=h*Y` with A = amount·π/2      |
| Dark edges centre | `spot` widget over the input file's first frame when there is one, default the middle | `x0`, `y0` as fractions of width and height               |
| Film grain        | 0…1, 0                                                                                | `noise=alls=round(grain·40):allf=t+u` (moves every frame) |
| Sharpen           | 0…1, 0                                                                                | `cas=strength=S`                                          |
| Soften            | 0…1, 0                                                                                | `gblur=sigma=soften·8·(width/1920)`                       |

Order: `cas`, `gblur`, `vignette`, `noise` (grain last, so it isn't softened or darkened away).

### 5. Lens — `grade.lens`

| Setting         | Range, default | FFmpeg                                                                               |
| --------------- | -------------- | ------------------------------------------------------------------------------------ |
| Colour fringing | 0…1, 0         | `rgbashift=rh=P:bh=-P` with P = round(fringe·8·(width/1920)) px                      |
| Lens bulge      | −1…1, 0        | `lenscorrection=k1=K·0.5:k2=0:i=bilinear` (positive bulges out, negative pinches in) |
| Motion trails   | 0…1, 0         | `lagfun=decay=0.5+trails·0.48`                                                       |

Motion trails and film grain do nothing useful on a still; for an image input they are left out and the card says "Motion trails need a video".

### 6. Auto fix — `grade.autoFix`

| Setting            | Default                      | FFmpeg                                                                             |
| ------------------ | ---------------------------- | ---------------------------------------------------------------------------------- |
| White balance      | off                          | `format=gbrpf32le,grayworld,format=yuv420p` (the filter works in planar float RGB) |
| Stretch the levels | off, with strength 0…1 (0.5) | `normalize=smoothing=24:independence=0:strength=S`                                 |
| Remove banding     | off                          | `deband` (its defaults)                                                            |

`advanced`: Stretch the levels' smoothing frames.

## Apply grade — `media.applyGrade`

- Category **Media**, title **Apply grade**, `executes: true`, phase `media`, built with the file nodes' `op()` in `src/shared/nodes/media.ts`.
- Inputs: `media: port("media", "File")`, `grade: port("grade", "Grade")`. Output: `media: port("media", "File")`. `compile` is `passMedia`, like the other file nodes.
- Description: "Applies a colour grade to a whole video or image file: everything in the picture, window and backdrop included." + `NEW_FILE`.
- Settings (the same fields as Convert, so an exported film can be graded at the quality it was made):
  - `codec`: `h264` (default), `h265`, `vp9`, `prores` — `FORMAT_NAMES` labels.
  - `crf`: `quality(18)`, shown for `h264`, `h265`, `vp9`.
  - `preset`: `PRESET_LABELS`, `advanced`, shown for `h264`, `h265`.
  - An image input always writes a PNG; the format settings are hidden with `showIfFile`.
- Card: the number of steps and their node titles ("3 steps: Colour and light, Look file, Texture"), the output format and quality, the length from the result. Warnings: no grade connected, or a grade whose steps are all neutral ("The grade changes nothing").
- Executor in `src/worker/executors/media.ts` (`defineExecutor(applyGradeNode, …)`), registered with the others:
  1. `requireMedia` the input; probe it (`ctx.ffmpeg.probe`) for width, height and whether it has sound.
  2. Build the filter chain from the grade entries (pure, below).
  3. Run FFmpeg: `-i <input>` plus each look image as `-loop 1 -i <png>`, `-filter_complex <chain>`, `-map [out]`, `-map 0:a? -c:a copy` (sound is never touched), then the codec's video arguments from `CODECS` **without** its `-vf EVEN_FRAME` (the even-size `scale` goes at the end of the filter chain instead, since `-vf` and `-filter_complex` can't be combined), `FASTSTART` for MP4, `-pix_fmt yuv420p` for H.264/H.265/VP9 and `yuv422p10le` for ProRes. Progress through `onProgress` with the input's length.
  4. `mediaResult(ctx, file, "Graded file")`; the file is `graded.<ext>` in the node's cache folder.
- Cache key: the usual node hash (type, params, upstream hashes), so changing any grade node's settings or order runs it again. Look files reach the key through `fileStamps` on the Look file node.

## Filter chain

A pure module `src/worker/media/grade.ts`:

- `gradeFilters(entries, { width, height, still }, lookPath)` → `{ chain: string, extraInputs: string[] }`. Each entry type has a pure `filtersFor(params, size)` returning its filters (empty when neutral); the chain joins them with `,` from `[0:v]` to `[out]`, inserting `split`/`blend` labels for strengths and numbered inputs (`[1:v]`, `[2:v]`…) for look images. Ends with `scale=trunc(iw/2)*2:trunc(ih/2)*2` for video.
- Tested by string in `grade.test.ts`, like `src/shared/mix/graph.ts`: each setting's filter at chosen values, neutral settings giving nothing, the order inside a node, the order across nodes, strength blends, look image input numbering, path escaping, an image input dropping temporal filters, and an all-neutral grade giving a plain copy chain.
- Card and filter code share the neutral checks (`isNeutral(params)` per entry type) so the card's "changes nothing" warning and the empty chain always agree.

## Files to touch

- `src/shared/schema/workflow.ts` — `PORT_TYPES`, `PORT_WORDS`.
- `src/shared/schema/values.ts` — `gradeValueSchema`.
- `src/shared/nodes/define.ts` — `NODE_CATEGORIES`, `CATEGORY_LABELS`, `gradeModifier`, `appendGrade`.
- `src/shared/nodes/grade.ts` (new) — the six grade nodes and their neutral checks; `grade.test.ts`.
- `src/shared/nodes/media.ts` — `applyGradeNode`; `media.test.ts` for its card.
- `src/shared/nodes/registry.ts` — register the grade nodes.
- `src/shared/nodes/looks.ts` — one look per new node; `bun run glyphs:bundle`.
- `src/shared/nodes/icon-name.ts` — any new card icons.
- `src/renderer/editor/ports.ts` — `CATEGORY_STYLES.Grade`, `PORT_COLORS.grade`, plus every other `NodeCategory` table the type check names.
- `src/worker/media/grade.ts` (new) + `grade.test.ts`.
- `src/worker/executors/media.ts` — the executor; `executors.test.ts` already fails if it's missing.
- `src/shared/nodes/effects.ts` — one sentence in Picture look's description or help pointing to Apply grade for grading a recording's whole picture.
- `AGENTS.md` — the grade list next to "Recording style is a list too", and `grade` in "Nodes pass resources forward".
- `src/shared/templates/` — no template changes.

## Checks

1. Unit tests above, then the full suite, type check, lint, format, no-comments check.
2. In the app, on the browser take with a plain white backdrop used for the Picture look stills (`~/.cache/opsis-remotion-extras/check`): Export video → Apply grade with each grade node alone at a strong setting, plus one chain of three nodes, then Export images from Use as clip of each result. Each still must show the change over the whole picture: backdrop, window frame, page, cursor and marks.
3. A `.cube` look file and a look image, at strength 1 and 0.5.
4. An image input (PNG out) and a video with sound (the sound must come through untouched; compare with `ffprobe`).
5. Changing one grade node runs only Apply grade again; changing nothing runs nothing.
6. Every output format plays in the Preview through Use as clip, except ProRes, which the Preview can't decode (the card says so for ProRes, as Convert's does).

## Risks

- **Range mapping.** The filters' own ranges differ (for example `eq` contrast is −1000…1000 with 1 neutral, `colortemperature` is in kelvin). The mappings above are the first guess; check them visually against Picture look at the same settings and adjust the factors, not the setting ranges.
- **Colour space.** Most filters run in YUV; `grayworld` needs planar float RGB, so it converts and back. Each conversion is lossless in float but adds time; only White balance pays it.
- **Speed.** One pass is usually several times faster than real time at 1080p; `lenscorrection` and `gblur` with a large sigma are the slow ones. Measure a 1-minute 1080p film with every node on and report it.
- **Re-encoding.** The input is usually an already encoded export, so Apply grade is a second encode. At the default quality the loss is hard to see; ProRes keeps it lowest but doesn't preview.
