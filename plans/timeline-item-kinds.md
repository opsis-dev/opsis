# Timeline items by kind

Every timeline item stored a `component` name and a loose `props` record. Three built-in components have known prop shapes, but TypeScript couldn't tell them apart from any other element, so code that read them had to check and convert at runtime. This plan gives every item a `kind` (`group`, `recording` or `element`) so the type system knows a group's or a recording's props wherever an item is read, and the spec schema validates them where specs enter.

Status: built on 2026-10-21, not committed yet. Every step in **Work** is done and its checks pass; **Built** lists what came out differently from the first draft.

## How to read this plan

- One change, built in the order of **Work**; each step type-checks and passes the tests before the next starts.
- Sections: **The problem**, **Fit with Opsis** (Already in place / Missing; every file, function and count named was checked in the repo on 2026-10-21, before the change), **Design**, **Work**, **Built**, **Risks**, **Sources**.

## Conventions

These restate rules from `AGENTS.md`; when the two disagree, `AGENTS.md` wins.

- No comments anywhere. File names in kebab-case.
- `src/shared` and `src/video` stay runtime-neutral: zod schemas only, no Node imports.
- **No old formats.** Nothing has been released, so there is no migration or fallback for specs in the old shape. Caches that depend on the spec get new keys and fill again.
- Before writing a helper, search for one that already does the job; after moving an export, check it still has users outside its file.
- **Checks.** `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`, the no-comments check from `AGENTS.md`, and the parity check in **Work** item 7. App checks open Opsis windows on the user's screen: ask before running them. Don't commit.

## The problem

`timelineItemSchema` (`src/shared/schema/timeline.ts`) was `elementRefSchema` (`component: z.string()`, `props: z.record(z.string(), z.unknown())`) extended with the placement fields (`id`, `from`, `durationInFrames`, `enter`, `exit`, `nodeId`, `handles`, `slot`, `envelope`, `camera`, `design`). Three components have known props:

| Component          | Props type                                                        | Written by                                    |
| ------------------ | ----------------------------------------------------------------- | --------------------------------------------- |
| `core/group`       | `GroupProps` (`tracks`, `remap`, `crop`, `wrapper`, `sequencer`…) | `groupItem` (`compile/clip.ts`)               |
| `core/recording`   | `RecordingProps` (about 25 fields, including the action log)      | `itemClip(…, "core/recording", …)`, Auto zoom |
| `core/placeholder` | `PlaceholderProps` (`title`, `subtitle`)                          | `itemClip(…, "core/placeholder", …)`          |

**Writing was typed.** Groups went through `groupItem`, the rest through `itemClip<C>(nodeId, component, props: PropsFor<C>, …)`, where `PropsFor` mapped the three names to their props and every other name to `Record<string, unknown>`. There were 7 `groupItem` and 11 `itemClip` calls across 8 files; recordings were written once, in `src/shared/nodes/camera.ts`, placeholders in `media.ts`, `capture.ts` and `camera.ts`.

**Reading wasn't.** An item read back had `props: Record<string, unknown>` whatever its component. The 13 calls to `groupOf` and `recordingOf` (in `compile/clip.ts`, `timeline/flatten.ts`, `timeline/thumbnails.ts`, `nodes/effects.ts`, `video/demo-timeline.tsx` and `video/fonts/font-provider.tsx`) turned it back into `GroupProps` or `RecordingProps` by parsing it with a schema at runtime.

**Why checking the name doesn't help.** Making the item a union on `component` (`"core/group" | "core/recording" | string`) doesn't narrow: `string` includes `"core/group"`, so `item.component === "core/group"` still leaves the "any element" case in the type. Narrowing needs a field whose values don't overlap, which is what `kind` is.

The same gap showed in code that rewrites items generically:

- `holdLastFrame` (`compile/clip.ts`) is only ever given group items (from `placeClip`, via `groupItem`), but took a `TimelineItem` and wrote `remap` into `props` without the types knowing it's a group.
- `mapItems` rebuilt a group as `{ ...mapped, props: { ...group, tracks } }`, `patchRecordings` as `{ ...item, props: patch(props) }`, and Picture look's `withLook` (`nodes/effects.ts`) as `{ ...item, props: { ...recording, look } }`; all relied on the runtime check to know what they held.
- `RecordingClip` was drawn through the element registry (`CORE_ELEMENTS` in `src/video/registry/index.ts`), so its props passed through the `component()` cast in `src/video/registry/framing.tsx` like any installed element.

## Fit with Opsis

Already in place:

- `groupPropsSchema` and `sequencerPropsSchema` (`compile/clip.ts`) and `recordingPropsSchema` (`compile/recording.ts`). `GroupProps`, `SequencerProps` and `RecordingProps` are derived from them. They reuse `rectSchema` and `viewportSchema` (`schema/actions.ts`), `remapSchema` (`compile/remap.ts`), `actionLogSchema`, `elementRefSchema` and `timelineTrackSchema`.
- `parseOncePer(schema)` (`src/shared/memo.ts`) memoises a parse per object; it serves the guards and the editor's envelope reader.
- Specs are built in-process by compile and never come back from an executor: no executor returns a `clip` or `timeline` value. `timelineSpecSchema` was attached to whole specs in two places: as Remotion's composition `schema` in `src/video/root.tsx`, and in the generated `export-root.tsx` of an exported project (`EXPORT_ROOT_TSX` in `src/worker/executors/export-project.ts`), which parses its `timeline.json` with it and passes no schema to its `<Composition>`.
- Cache keys hash the spec with `stableJson` (sorted keys): `pictureKey` (`src/worker/render/picture.ts`), the card-still hashes (`src/main/stills.ts`), `lookupKey` for mix plans. A new item shape gives them new keys with no version bump.
- `last-run.json` and node caches hold executor results only, so they don't contain timeline items. `examples/` holds workflows, not specs. An exported project carries its own copy of the sources and its own `timeline.json`.
- Parse cost, measured on the worker test project (a 52.7 s take, 26 actions, 1,140 cursor samples): 0.31 ms to parse its recording props, 0.06 ms for the whole 39 KB spec with props unchecked.

Missing:

- A `kind` field and per-kind item schemas.
- Typed item writers per kind, and readers that narrow instead of parse.
- A renderer path that draws groups and recordings by kind instead of by registry name.

## Design

**Item shape.** One base with the placement fields, and three kinds:

```ts
type TimelineItem = ItemBase &
  (
    | { kind: "group"; props: GroupProps }
    | { kind: "recording"; props: RecordingProps }
    | { kind: "element"; component: string; props: Record<string, unknown> }
  );
```

- Groups and recordings lose `component`: their kind says what they are, and the renderer no longer looks them up by name.
- Placeholders stay elements (`component: "core/placeholder"`). Only the `Placeholder` element reads their props, through the registry like every element, so a kind of their own would type nothing new. `itemClip` keeps their props typed.
- Videos stay elements too (`component: "core/video"`). `core/video` is also an element users place, with settings of its own (`src/shared/elements/core.ts`), drawn through the registry; `videoOf` keeps reading its sound fields with `parseOncePer`, and it accepts anything with a `kind`, so the Timeline's audio bars can read a video bar the same way.

**Schemas.** `timelineItemSchema` becomes `z.discriminatedUnion("kind", [...])`, each member `itemBaseSchema.extend({ kind: z.literal(…), props: … })`, the element member also taking `component`. `groupPropsSchema.tracks` refers back to `timelineTrackSchema`, so the group schema is recursive: it is written with a zod 4 getter with one return annotation (`get tracks(): z.ZodArray<typeof timelineTrackSchema>`), which is all inference needs.

- To keep `src/shared/schema` free of `src/shared/compile` imports, `groupPropsSchema`, `sequencerPropsSchema` and `remapSchema` move to `schema/timeline.ts` (with `Remap`; `compile/remap.ts` keeps the remap maths), and `recordingPropsSchema` with the annotation, cursor, ripple, keystroke, blur-area and picture-look schemas, `CAMERA_STYLES` and `HAND_DRAWN_LOOKS` moves to a new `schema/recording.ts`. The defaults (`DEFAULT_CURSOR_LOOK`, `DEFAULT_RIPPLE`, `DEFAULT_KEYCAP`, `defaultRecordingProps`), `PREVIEW_FILE_WIDTH` and the helpers stay in `compile/recording.ts`. Importers import from where a name lives; nothing is re-exported.
- `schema/timeline.ts` exports `ItemBase`, `GroupItem`, `RecordingItem` and `ElementItem`.
- The Remotion root passes no `schema` to `<Composition>` (see **Risks**).

**Writers.** `groupItem(placement, props)` returns a `GroupItem`; the new `recordingClip(nodeId, props, frames, extra)` writes recordings; `itemClip` becomes element-only (`ElementPropsFor<C>` types `core/placeholder` and leaves every other name a record), and `PropsFor`/`BuiltInProps` go away. Both clip writers share one private `contentClip`.

**Readers.**

- `groupOf(item)` becomes `item.kind === "group" ? item.props : null`, the same for `recordingOf`; code that rebuilds an item narrows on `item.kind` directly. `readGroup`/`readRecording` go; `parseOncePer` stays for `videoOf` and the envelope reader.
- `holdLastFrame`, `placeClip` and `withExit` take and return group items.
- `mapItems` rebuilds groups through `groupItem`; `walkItems` and `patchRecordings` narrow on `kind`; Picture look's `withLook` narrows to recordings and elements.
- `itemHasSound` and `soundNodeId` read through `videoOf`, which only answers for elements; the volume change in `applyMixLevels` and the mute in `withMasterStem` (`src/shared/mix/levels.ts`) narrow to elements first.
- `inputTake` and `usedFontNames` check `kind` before reading a component or `fontFamily`.

**Renderer** (`src/video/demo-timeline.tsx`):

- `ItemContent` switches on `kind`: group → `Group`, recording → `RecordingClip` with its typed props, element → `Element`.
- `PREMOUNTED` goes; a Sequence premounts when `kind !== "element"`.
- `"core/recording"` leaves `CORE_ELEMENTS` and `CoreProps` in `src/video/registry/index.ts`, so recordings no longer pass through the `component()` cast.
- The element path ran props through `resolveProps`, which turned `project:`/`app:` refs into URLs. `RecordingClip` already resolved `src` and `proxySrc` with `useAsset`; it now resolves `backdrop` too (a backdrop can be a project image), so nothing a recording draws loses its resolved address.
- The Sequence `name` prop shows the kind's name for groups and recordings.

**Timeline bars** (`src/shared/timeline/flatten.ts`):

- `VideoBar` gets `kind` (`recording` or `element`), and `component` only on element bars.
- `visitItem` narrows on `kind` and hands `leafBar` the narrowed item; `thumbnailOf` (`timeline/thumbnails.ts`) takes a recording or element item.
- Tests that found a bar by `component === "core/recording"` use `kind`.

## Work

1. **Move the prop schemas** into `schema/recording.ts` and `schema/timeline.ts` as in **Design**. Update imports; no export without outside users (`HIGHLIGHT_LOOKS` and `pictureLookSchema` became module-private) and no second definition of a moved name. Checks pass with no behaviour change.
2. **Item union.** Add `itemBaseSchema` and the three members. `TimelineItem` is inferred. Type errors now list every writer and reader to change; fix them in steps 3–5 without casts. Time `bun run typecheck` before and after.
3. **Writers.** Rework `groupItem` and `itemClip`, add `recordingClip`, and update the calls and the tests that build items by hand (`clip.test.ts`, `font-provider.test.ts`, `duck.test.ts`, `picture.test.ts`). Tests that read an element's props go through a new `elementOf` fixture (`src/shared/test-fixtures.ts`), which fails on any other kind.
4. **Readers and rewrites** as in **Design → Readers**.
5. **Renderer and bars** as in **Design → Renderer** and **Timeline bars**.
6. **Tests.**
   - Schema (`src/shared/schema/timeline.test.ts`): a group item with recording props, a recording with a string `log`, an element without `component`, an item with `component` and no `kind`, and a bad recording inside a group are all rejected; a valid group parses unchanged.
   - Compile (`clip.test.ts`): every built-in item compiles to its kind, and `groupOf`/`recordingOf` return the item's own props.
   - Flatten: the recording bar is found by `kind: "recording"` (`compile.test.ts`).
   - The round-trip tests for timeline handles stay green.
7. **Parity check.** Compile every project in `examples/` and the saved test projects (with their `last-run.json` results, and with a made-up take for the examples) with the code before step 2 and after step 5. Map the old items to the new shape (`core/group` → group, `core/recording` → recording, else element) and compare with sorted keys: no differences allowed.
8. **Docs and checks.** `AGENTS.md` describes item kinds and why the Remotion root has no `schema`. Run the checks in **Conventions**. With the user's go-ahead, run the app checks that preview, show the Timeline and export, since card stills and pictures render once more under their new keys.

## Built

- **Parity:** 14 compiles (6 examples and the grade test project, each with and without results) match the old output once mapped, with groups, recordings (six of the compiles hold one) and elements in each.
- **Type check:** 19.4 s before, 20.0 s after.
- **Tests:** 1,264 pass.
- **App:** on a copy of the grade test project with a project image as its backdrop, the editor previews, the Timeline shows the recording bar with its filmstrip and action mark, card stills draw, and both exports render (1920×1080, 6.07 s), the backdrop image included.
- **The Remotion root's `schema` had to go.** The first export after step 5 failed with `TypeError: Converting circular structure to JSON`. Remotion's render page runs `JSON.stringify` over the whole composition list (`getCanSerializeDefaultProps` in `@remotion/studio`'s render entry) before every render and still, and that list includes the composition's `schema`. A recursive zod schema is a cyclic object graph; `z.lazy` is no way out, since it turns cyclic after its first parse. Remotion never validates props with that schema (no `safeParse` in `remotion`, `@remotion/renderer`, `@remotion/studio` or `@remotion/player`): it only feeds the Studio's props editor, which Opsis doesn't use. So `src/video/root.tsx` passes none, and specs are still checked where they come in from outside: the exported project's root parses its `timeline.json`.

## Risks

- **Recursive inference.** zod 4 inferred the recursive union with one return annotation on the `tracks` getter and no casts; the type check grew by about 3%.
- **Remotion's composition schema.** The first draft assumed Remotion validates input props with it before rendering. It doesn't, and passing a recursive schema breaks every render (see **Built**). Never put the spec schema back on `<Composition>`.
- **Exported projects.** The generated `export-root.tsx` parses `timeline.json` with the same schema, so an exported project validates its groups and recordings too. Projects exported before the change keep their own copy of the old sources and still work.
- **Item spreads.** `{ ...item, props: … }` on a union keeps the kind only when the new props match it; TypeScript reports each mismatch, which is the point. Fix them by narrowing, never by widening the props type or casting.
- **A recording written as an element.** `itemClip(…, "core/recording", …)` still type-checks (element props are a record) but would draw nothing, since the registry no longer has `core/recording`. Recordings go through `recordingClip`.
- **Cache refills.** Every export renders its picture once and card stills redraw once after the change, because their keys hash the spec. Expected, and in line with **No old formats**.

## Sources

- `src/shared/schema/timeline.ts`: `elementRefSchema`, `groupPropsSchema`, `itemBaseSchema`, `timelineItemSchema`, `timelineTrackSchema`, `timelineSpecSchema`; `src/shared/schema/recording.ts`: `recordingPropsSchema`, `CAMERA_STYLES`, `AnnotationSpec`, `KeystrokePosition`.
- `src/shared/compile/clip.ts`: `groupOf`, `recordingOf`, `videoOf`, `itemClip`, `recordingClip`, `contentClip`, `groupItem`, `holdLastFrame`, `mapItems`, `walkItems`, `patchRecordings`, `itemHasSound`, `inputTake`.
- `src/shared/compile/recording.ts`: defaults and helpers; `src/shared/compile/remap.ts`: remap maths.
- `src/shared/memo.ts`: `parseOncePer`.
- `src/shared/mix/levels.ts`: `applyMixLevels` and `withMasterStem`; `src/shared/nodes/effects.ts`: `withLook`; `src/shared/nodes/camera.ts`: Auto zoom's `recordingClip` call.
- `src/shared/timeline/flatten.ts` (`VideoBar`, `leafBar`, `visitItem`), `src/shared/timeline/thumbnails.ts` (`thumbnailOf`), `src/shared/timeline/audio-bars.ts` (`clipSoundBar`).
- `src/video/demo-timeline.tsx` (`ItemContent`, `Element`, `Group`), `src/video/core/recording-clip.tsx` (`RecordingClip`), `src/video/registry/index.ts` (`CORE_ELEMENTS`, `CoreProps`), `src/video/registry/framing.tsx` (`component`), `src/video/root.tsx`.
- `src/worker/executors/export-project.ts` (`EXPORT_ROOT_TSX`), `src/worker/render/picture.ts` (`pictureKey`), `src/main/stills.ts`.
- `node_modules/@remotion/studio/dist/renderEntry.js` (`getCanSerializeDefaultProps`, `getUnevaluatedComps`).
