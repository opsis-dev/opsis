# Timeline items by kind

Every timeline item stores a `component` name and a loose `props` record. Three built-in components have known prop shapes, but TypeScript can't tell them apart from any other element, so code that reads them has to check and convert at runtime. This plan gives every item a `kind` (`group`, `recording` or `element`) so the type system knows a group's or a recording's props wherever an item is read, and the spec schema validates them where specs enter.

Status: plan only. The runtime checks this replaces are in place: `groupOf` and `recordingOf` validate props against `groupPropsSchema` and `recordingPropsSchema`, once per props object.

## How to read this plan

- One change, built in the order of **Work**; each step type-checks and passes the tests before the next starts.
- Sections: **The problem**, **Fit with Demoflow** (Already in place / Missing; every file, function and count named was checked in the repo on 2026-10-19), **Design**, **Work**, **Risks**, **Sources**.

## Conventions

These restate rules from `AGENTS.md`; when the two disagree, `AGENTS.md` wins.

- No comments anywhere. File names in kebab-case.
- `src/shared` and `src/video` stay runtime-neutral: zod schemas only, no Node imports.
- **No old formats.** Nothing has been released, so there is no migration or fallback for specs in the old shape. Caches that depend on the spec get new keys and fill again.
- Before writing a helper, search for one that already does the job; after moving an export, check it still has users outside its file.
- **Checks.** `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`, the no-comments check from `AGENTS.md`, and the parity check in **Work** item 7. App checks open Demoflow windows on the user's screen: ask before running them. Don't commit.

## The problem

`timelineItemSchema` (`src/shared/schema/timeline.ts`) is `elementRefSchema` (`component: z.string()`, `props: z.record(z.string(), z.unknown())`) extended with the placement fields. Three components have known props:

| Component          | Props type                                                        | Written by                      |
| ------------------ | ----------------------------------------------------------------- | ------------------------------- |
| `core/group`       | `GroupProps` (`tracks`, `remap`, `crop`, `wrapper`, `sequencer`…) | `groupItem` (`compile/clip.ts`) |
| `core/recording`   | `RecordingProps` (about 20 fields, including the action log)      | `recordingClip`                 |
| `core/placeholder` | `PlaceholderProps` (`title`, `subtitle`)                          | `placeholderClip`               |

**Writing is typed.** All three writers go through `itemClip<C>(nodeId, component, props: PropsFor<C>, …)`, where `PropsFor` maps the three names to their props and every other name to `Record<string, unknown>`. `itemClip`, `groupItem`, `placeholderClip` and `recordingClip` are called 20 times across 8 files.

**Reading isn't.** An item read back has `props: Record<string, unknown>` whatever its component. The 11 calls to `groupOf` and `recordingOf` (in `compile/clip.ts`, `timeline/flatten.ts`, `timeline/thumbnails.ts`, `video/demo-timeline.tsx` and `video/fonts/font-provider.tsx`) turn it back into `GroupProps` or `RecordingProps` by parsing it with a schema at runtime.

**Why checking the name doesn't help.** Making the item a union on `component` (`"core/group" | "core/recording" | string`) doesn't narrow: `string` includes `"core/group"`, so `item.component === "core/group"` still leaves the "any element" case in the type. Narrowing needs a field whose values don't overlap, which is what `kind` is.

The same gap shows in code that rewrites items generically:

- `holdLastFrame` (`compile/clip.ts`) is only ever given group items (from `placeClip`, via `groupItem`), but takes a `TimelineItem` and writes `remap` into `props` without the types knowing it's a group.
- `mapItems` rebuilds a group as `{ ...mapped, props: { ...group, tracks } }`, and `patchRecordings` as `{ ...item, props: patch(props) }`; both rely on the runtime check to know what they hold.
- `RecordingClip` is drawn through the element registry (`CORE_ELEMENTS` in `src/video/registry/index.ts`), so its props pass through the `component()` cast in `src/video/registry/framing.tsx` like any installed element.

## Fit with Demoflow

Already in place:

- `groupPropsSchema` and `sequencerPropsSchema` (`compile/clip.ts`) and `recordingPropsSchema` (`compile/recording.ts`). `GroupProps`, `SequencerProps` and `RecordingProps` are derived from them. They reuse `rectSchema` and `viewportSchema` (`schema/actions.ts`), `remapSchema` (`compile/remap.ts`), `actionLogSchema`, `elementRefSchema` and `timelineTrackSchema`.
- `parseOncePer(schema)` (`src/shared/memo.ts`) memoises a parse per object; it serves the guards and the editor's envelope reader.
- Specs are built in-process by compile and never come back from an executor: no executor returns a `clip` or `timeline` value. `timelineSpecSchema` runs on whole specs in two places, Remotion's composition `schema` in `src/video/root.tsx` and the generated `export-root.tsx` of an exported project (`EXPORT_ROOT_TSX` in `src/worker/executors/export-project.ts`).
- Cache keys hash the spec with `stableJson` (sorted keys): `pictureKey` (`src/worker/render/picture.ts`), the card-still hashes (`src/main/stills.ts`), `lookupKey` for mix plans. A new item shape gives them new keys with no version bump.
- `last-run.json` and node caches hold executor results only, so they don't contain timeline items. `examples/` holds workflows, not specs. An exported project carries its own copy of the sources and its own `timeline.json`.
- Parse cost, measured on the worker test project (a 52.7 s take, 26 actions, 1,140 cursor samples): 0.31 ms to parse its recording props, 0.06 ms for the whole 39 KB spec with props unchecked.

Missing:

- A `kind` field and per-kind item schemas.
- Typed item writers per kind, and readers that narrow instead of parse.
- A renderer path that draws groups and recordings by kind instead of by registry name.

## Design

**Item shape.** One base with the placement fields (`id`, `from`, `durationInFrames`, `enter`, `exit`, `nodeId`, `handles`, `slot`, `envelope`), and three kinds:

```ts
type TimelineItem = ItemBase &
  (
    | { kind: "group"; props: GroupProps }
    | { kind: "recording"; props: RecordingProps }
    | { kind: "element"; component: string; props: Record<string, unknown> }
  );
```

- Groups and recordings lose `component`: their kind says what they are, and the renderer no longer looks them up by name.
- Placeholders stay elements (`component: "core/placeholder"`). Only the `Placeholder` element reads their props, through the registry like every element, so a kind of their own would type nothing new. `placeholderClip` keeps its typed parameter.

**Schemas.** `timelineItemSchema` becomes `z.discriminatedUnion("kind", [...])`, each member `itemBaseSchema.extend({ kind: z.literal(…), props: … })`, the element member also taking `component`. `groupPropsSchema.tracks` refers back to `timelineTrackSchema`, so the group schema is recursive: write it with a zod 4 getter (`get tracks() { return z.array(timelineTrackSchema); }`) and annotate where inference needs it.

- To keep `src/shared/schema` free of `src/shared/compile` imports, `groupPropsSchema` and `sequencerPropsSchema` move to `schema/timeline.ts`, and `recordingPropsSchema` with its annotation and keystroke schemas moves to a new `schema/recording.ts`. `compile/clip.ts` and `compile/recording.ts` import the types from there; nothing is re-exported.
- `remapSchema` moves with the group schema or is imported by it; `compile/remap.ts` keeps the remap maths.

**Writers.** `groupItem(placement, props)` returns the group kind; `recordingClip` and `placeholderClip` build their items directly; `itemClip` becomes element-only (`component: string`, `props: Record<string, unknown>`), and `PropsFor`/`BuiltInProps` go away.

**Readers.**

- `groupOf(item)` becomes `item.kind === "group" ? item.props : null`, the same for `recordingOf`; or callers narrow on `item.kind` directly and the two helpers go. `parseOncePer` stays for the envelope reader.
- `holdLastFrame` takes a group item.
- `mapItems` rebuilds groups through `groupItem`.
- `patchRecordings` maps recording items only.
- `itemHasSound`, `soundNodeId`, and the volume change in `applyMixLevels` and the mute in `withMasterStem` (`src/shared/mix/levels.ts`) narrow to elements first, since only `core/video` elements carry sound.

**Renderer** (`src/video/demo-timeline.tsx`):

- `ItemContent` switches on `kind`: group → `Group`, recording → `RecordingClip` with its typed props, element → `ElementItem`.
- `PREMOUNTED` (`"core/group"`, `"core/recording"`) becomes a check on `kind !== "element"`.
- `"core/recording"` leaves `CORE_ELEMENTS` and `CoreProps` in `src/video/registry/index.ts`, so recordings no longer pass through the `component()` cast.
- The Sequence `name` prop shows the kind's name for groups and recordings.

**Timeline bars** (`src/shared/timeline/flatten.ts`):

- `VideoBar` gets `kind`, and `component` only on element bars.
- `thumbnailOf` (`timeline/thumbnails.ts`) narrows on `kind` before its `switch (item.component)`.
- Tests that find a bar by `component === "core/recording"` use `kind`.

## Work

1. **Move the prop schemas.** Create `src/shared/schema/recording.ts` with `recordingPropsSchema`, the annotation spec and keystroke schemas, `CAMERA_STYLES` and their types. Move `groupPropsSchema` and `sequencerPropsSchema` into `schema/timeline.ts` with the recursive `tracks` getter. Update imports; the guard must show no export without outside users and no second definition of any moved name. Checks pass with no behaviour change.
2. **Item union.** Add `itemBaseSchema` and the three members. `TimelineItem` is inferred. Type errors now list every writer and reader to change; fix them in steps 3–5 without casts. Time `bun run typecheck` before and after (about 16 s today) and note the change.
3. **Writers.** Rework `groupItem`, `recordingClip`, `placeholderClip` and `itemClip` as in **Design**; update the 20 calls and the tests that build items by hand (`clip.test.ts`, `font-provider.test.ts`, `flatten` and `compile` tests).
4. **Readers and rewrites.** Narrow on `kind` in the 11 `groupOf`/`recordingOf` calls, `holdLastFrame`, `mapItems`, `patchRecordings`, `walkItems`, `itemHasSound`, `soundNodeId` and the two item rewrites in `mix/levels.ts`. Delete `readGroup`/`readRecording` and the props parsing they did; keep `parseOncePer` for envelopes.
5. **Renderer and bars.** `ItemContent`, `PREMOUNTED`, the registry entries, `VideoBar.kind`, `thumbnailOf`, and the Timeline code that reads bar components.
6. **Tests.**
   - Schema: a group item with recording props, a recording with a string `log`, and an element without `component` are all rejected.
   - Compile: every built-in item compiles to its kind.
   - Flatten: recording bars carry `kind: "recording"`.
   - Keep the existing round-trip tests for timeline handles green.
7. **Parity check.** Compile every project in `examples/` and the saved test projects (with their `last-run.json` results) with the code before step 2 and after step 5. Map the old items to the new shape (`core/group` → group, `core/recording` → recording, else element) and compare with `stableJson`: no differences allowed.
8. **Docs and checks.** Update `AGENTS.md` where it describes specs or `groupOf` (search for `core/group`, `groupOf`, `recordingOf`, `TimelineItem`). Run the checks in **Conventions**. With the user's go-ahead, run the app checks that preview, record and export, since card stills and pictures render once more under their new keys.

## Risks

- **Recursive inference.** zod 4 infers recursive objects through getters, but a recursive schema inside a discriminated union can need explicit `z.ZodType` annotations, and can slow the type check. Measure in step 2; if inference fails, annotate the group schema's type rather than casting.
- **Remotion's composition schema.** `root.tsx` passes `timelineSpecSchema` as the composition `schema`, and Remotion validates input props with it before rendering. With props validated, every render start parses its recordings once (0.31 ms for the measured take). The Studio props editor may not draw a recursive union; Demoflow doesn't use the Studio.
- **Exported projects.** The generated `export-root.tsx` parses `timeline.json` with the same schema, so an exported project validates its recordings too. Projects exported before the change keep their own copy of the old sources and still work.
- **Item spreads.** `{ ...item, props: … }` on a union keeps the kind only when the new props match it; TypeScript reports each mismatch, which is the point. Fix them by narrowing, never by widening the props type or casting.
- **Cache refills.** Every export renders its picture once and card stills redraw once after the change, because their keys hash the spec. Expected, and in line with **No old formats**.

## Sources

- `src/shared/schema/timeline.ts`: `elementRefSchema`, `timelineItemSchema`, `timelineTrackSchema`, `timelineSpecSchema`.
- `src/shared/compile/clip.ts`: `groupPropsSchema`, `sequencerPropsSchema`, `groupOf`, `recordingOf`, `itemClip`, `groupItem`, `placeholderClip`, `recordingClip`, `holdLastFrame`, `mapItems`, `walkItems`, `patchRecordings`, `itemHasSound`.
- `src/shared/compile/recording.ts`: `recordingPropsSchema`, `CAMERA_STYLES`, `AnnotationSpec`, `KeystrokePosition`.
- `src/shared/memo.ts`: `parseOncePer`.
- `src/shared/mix/levels.ts`: `applyMixLevels` and `withMasterStem`.
- `src/shared/timeline/flatten.ts` (`VideoBar`, `leafBar`, `visitItem`) and `src/shared/timeline/thumbnails.ts` (`thumbnailOf`).
- `src/video/demo-timeline.tsx` (`ItemContent`, `PREMOUNTED`, `Group`), `src/video/registry/index.ts` (`CORE_ELEMENTS`, `CoreProps`), `src/video/registry/framing.tsx` (`component`), `src/video/root.tsx`.
- `src/worker/executors/export-project.ts` (`EXPORT_ROOT_TSX`), `src/worker/render/picture.ts` (`pictureKey`), `src/main/stills.ts`.
