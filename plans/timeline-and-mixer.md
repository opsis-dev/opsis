# Timeline and Mixer panels

Two new editor panels next to Preview and Run: a **Timeline** that shows and edits the compiled film as bars, and a **Mixer** that works like an audio console. Both are as capable as a desktop video editor and audio app, while the node graph stays the single source of truth: every edit in either panel writes back to node settings or to the project's mix, and undo covers it.

Status: built (all eight phases). Where the build differs from this plan, the notes under "As built" at the end say how.

## Decisions

| Question                | Answer                                                                                                                                                                       |
| ----------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Timeline editing        | Full: move, trim, split, ripple, slip, snap, keyframes. Every edit writes back to the graph.                                                                                 |
| Mixer depth             | Full console: volume, pan, EQ, compressor, buses, ducking by sidechain, master limiter, loudness target and meters. Exports apply exactly what the preview plays.            |
| Mute and solo           | Saved in the project and applied to exports.                                                                                                                                 |
| Placement               | Tabs in the bottom panel: Preview, Timeline, Mixer and Run. Every tab can open in its own window.                                                                            |
| Ducking in the meantime | The simple version on the Music node ships first (dip while other sounds play, by their length). The Mixer's sidechain ducking replaces it later and reads the old settings. |

## Layout

- The bottom panel holds four tabs: **Preview**, **Timeline**, **Mixer**, **Run**. Its size and open state persist through `useDefaultLayout`, and the active tab is remembered, owned by `src/renderer/editor/panels.ts` like the existing panels.
- View menu and top bar get one entry per panel. Shortcuts follow common editors: Ctrl/Cmd+J still toggles the whole row; Ctrl/Cmd+1…4 focus Preview, Timeline, Mixer, Run.
- The Timeline is the widest by default; the Mixer shows as many channel strips as fit and scrolls sideways.

### Moving panels into their own windows (later phase)

- Each panel header gets **Open in new window**. The panel leaves the row and opens in a `BrowserWindow` that loads the same renderer at `#/panel/<name>?project=<id>`; closing the window docks it back. Window bounds persist per panel.
- The editor window owns the project store. Other windows hold a mirror that follows it over a `BroadcastChannel` (same origin, same session), sending edits back as commands so undo stays in one place.
- The playhead and play/pause state sync through the same channel at frame rate, so a popped-out Timeline drives the Preview in another window.
- The live recording frames (the dedicated `MessagePort` from the worker) are re-transferred to whichever window shows Run.
- Works for a second monitor: the editor on one screen, Preview full size and Timeline on the other.

## Data: how the panels know what to edit

The compiled `TimelineSpec` already gives every item and sound its start, length and source `nodeId`. Edits also need to know **which setting** produced each number, so compile records it:

```ts
type ParamRef = { nodeId: string; param: string; unit: "ms" | "frames"; scale?: number };
type EditHandles = {
  move?: ParamRef;
  trimStart?: ParamRef;
  trimEnd?: ParamRef;
  speed?: ParamRef;
  volume?: ParamRef;
};
```

- `TimelineItem.handles` and `AudioItem.handles` are filled by each node's `compile()` (Delay → `move`, Trim → `trimStart`/`trimEnd`, Sound effect → `move` = `atMs`, Music → `move` = `startMs` and `trimStart` = `trimMs`, Sequence → order).
- An edit applies the delta to the referenced setting. When nothing owns the number (for example moving a clip that has no Delay before it), the edit inserts the right node (Delay, Trim) into the graph in one undo step and says so in a toast.
- A test compiles every node, applies each handle with a delta and recompiles, checking the bar moved by exactly that delta (round-trip test for every handle).

## Timeline panel

### Display

- **Ruler** in seconds and timecode (switchable), with zoom from the whole film down to single frames, and smooth scroll.
- **Video lanes**, one per track: a bar per item, labelled with the node's name and coloured by category; transitions draw as overlaps with their name.
- **Filmstrips** on recording and file bars: small frames from `recording.preview.mp4`, made by FFmpeg as a sprite sheet into the node's cache folder.
- **Action marks** on recording bars: every click, keystroke and scroll from the action log as ticks, with chapter names; hovering shows the action.
- **Audio lanes** below: waveforms for every sound, the volume line drawn over each, and the ducking dips visible on the music.
- **Markers lane**: chapters, user markers (M), and beats detected in the music.
- **Playhead** shared with Preview: drag to scrub (with audio scrub), click the ruler to jump.

### Editing

- Select one or many bars (click, shift-click, marquee); delete removes the owning node or edge, with a confirmation when it removes more than one node.
- **Move** a bar (writes Delay or the sound's start), **trim** either edge (writes Trim or the item's length), **split** at the playhead (S): the clip becomes two Trim nodes joined by the Sequence it was in.
- **Ripple** (shift while dragging: later items follow), **roll** (the cut between two bars moves), **slip** (the content moves inside a fixed bar).
- **Snapping** to the playhead, bar edges, markers, action marks and beats; hold Alt to turn it off.
- **Speed**: drag a bar's corner to change its speed (writes the Speed node, inserted if missing).
- **Keyframes**: volume and pan automation on audio lanes; opacity, position and scale automation on overlay bars. Keyframes are stored as an `envelope` setting (`{ frame, value, ease }[]`) on the owning node, shown in the Inspector too, and compile into frame-driven values, so they stay pure functions of the frame.
- Keyboard: J/K/L shuttle, ←/→ one frame, Shift+←/→ one second, I/O in and out points for a preview loop, Home/End, Ctrl/Cmd+= and − to zoom, Z to fit.
- Every edit is one undo step, merged while dragging.

### Selection follows the graph

Selecting a bar selects its node on the canvas and opens it in the Inspector; selecting a node highlights its bars. Hidden-in-a-group nodes reveal their group.

## Mixer panel

### Channels

- One **channel strip** per sound source: the Music node, each Sound effect, each clip's own sound (recordings with sound later, imported video files), grouped by node.
- **Buses**: Music, Effects, Clips by default; users can add, rename and route channels to buses. Every bus feeds the **Master**.
- Each strip, in signal order:
  1. **Trim** (input gain)
  2. **High-pass** filter
  3. **EQ**: 4 bands (low shelf, two peaks, high shelf) with a small curve display
  4. **Compressor**: threshold, ratio, attack, release, makeup, with a gain-reduction meter
  5. **Pan**
  6. **Fader** in dB
  7. **Mute** and **Solo** (saved, applied to exports)
- Buses add **ducking by sidechain**: "lower this bus when that bus plays", with amount, attack and release. This replaces the simple Music ducking and is the loudness-aware version.
- Master adds EQ, compressor, a **limiter** with a ceiling, and a **loudness target**: web (−14 LUFS), podcast (−16), broadcast (−23), or off.
- Presets per strip ("Voice", "Music bed", "Punchy effects") and copy/paste of strip settings.

### Meters

- Per strip: peak and RMS with peak hold, gain reduction for the compressor.
- Master: momentary, short-term and integrated loudness (LUFS), true peak, and loudness range.
- Meters read precomputed levels at the playhead (below), so they move during playback and scrubbing at no cost and show what the export will measure.

### Where the settings live

Mix settings are a project-level `mix` section of the workflow file, keyed by node id (channels) and bus id, migrated by `normalizeWorkflow`. Deleting a node removes its channel; undo restores it. Ducking settings from the simple Music node migrate into a bus sidechain.

## Preview equals export: rendered stems

Volume alone can be done by Remotion per frame, but EQ, compression, sidechain and limiting cannot. To keep preview and export identical, the worker renders the mix with FFmpeg:

1. Each channel is rendered to a **stem** (its sounds placed in time, processed by its strip) with an FFmpeg filter graph: `highpass`, `equalizer`, `acompressor`, `volume`, `pan`.
2. Buses mix their stems and apply ducking with `sidechaincompress` keyed from the chosen bus.
3. Master applies `equalizer`, `acompressor`, `alimiter`, and `loudnorm` in two passes for the loudness target.
4. Each stem is cached by a hash of its inputs and settings, like node outputs; changing one fader re-renders that channel, its bus and the master, nothing else.
5. `TimelineSpec` gets a `mix` field. When present, `DemoTimeline` plays the single master stem instead of individual sounds, in both the Player and `renderMedia()`.
6. While a fader or knob is being dragged, the preview applies a temporary Web Audio gain or filter so it responds at once; the re-rendered stem replaces it when ready (a small "updating mix" mark shows meanwhile).

Analysis runs alongside: FFmpeg writes per-frame peak and RMS for every stem and `ebur128` loudness for the master into the cache; the meters and the waveforms read these files. Waveform peaks come from the same pass at several zoom levels.

The filter graph is built by a pure function (`src/shared/mix/graph.ts`) and unit-tested by its output string; the executor only runs it.

## Phases

1. **Bottom row**: split Preview and Run into separate panels, add empty Timeline and Mixer panels, View menu entries, persistence.
2. **Analysis**: waveform peaks, filmstrips, action marks, cached in the worker.
3. **Timeline view**: lanes, bars, waveforms, ruler, playhead sync, selection linking.
4. **Edit handles**: `handles` in compile, the round-trip test, move and trim, then split, ripple, roll, slip, snapping, speed.
5. **Keyframes**: `envelope` settings, lanes, Inspector editor.
6. **Mixer, levels**: strips with fader, pan, mute, solo, buses, routing; volume and pan through Remotion.
7. **Mixer, processing**: rendered stems, EQ, compressor, sidechain ducking, master limiter and loudness target, meters.
8. **Windows**: open any panel (Preview, Timeline, Mixer, Run) in its own window, state and playhead sync, live frames to the Run window.

## Risks

- Write-back edits must never produce a graph the user can't follow: inserted nodes are named after the edit ("Delay from timeline") and placed next to their source.
- Stem rendering adds latency between a change and hearing it exactly; the temporary Web Audio path covers dragging, but heavy projects may take seconds to settle.
- Pop-out windows double the renderer memory; the Preview window carries the heaviest load (the Player).
- Very long projects need virtualised lanes and level-of-detail waveforms to stay smooth.

## As built

- **Tabs, not a split row.** The four panels are tabs; each has **Open in new window**. A tab that's in a window shows a window mark in the tab bar and focuses that window when clicked. The Preview tab stays mounted while another tab is showing, so J/K/L in the Timeline still plays sound.
- **Split** joins the two Trim nodes in the Sequence the clip was in when that Sequence uses cuts and has a free input; otherwise it adds a small cut-only Sequence named "Split", so the existing transition setting doesn't put a fade inside the clip.
- **Edits that need a node** add it in one undo step with a toast: Delay and Speed at the clip's place in its container, Trim right after the clip. Moving a clip in a Sequence earlier than the clip before it is refused with an explanation.
- **Sidechain amount** maps to `sidechaincompress` with a −40 dBFS threshold, sized so a key at −12 dBFS lowers the bus by the amount; louder keys lower it more.
- **Temporary Web Audio** while dragging applies at the master output: fader and trim changes are scaled by the channel's share of the master level at the playhead, EQ and low cut become filters, pan a stereo panner. The new stem replaces it when ready.
- **Meters** show one combined peak/RMS bar per strip (not separate left and right), with peak hold over 1.5 s, and a gain-reduction bar when the compressor is on.
- **Pan keyframes** apply in rendered stems only; without the engine the preview ignores pan.
