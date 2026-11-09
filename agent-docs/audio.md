# Audio

Built-in sounds and music, the Mixer and its stems, ducking, how sounds are placed, and the sound nodes with the FFmpeg traps their filters work around. Paths are relative to `crates/` unless they name another folder. The Timeline's sound playback is in [timeline.md](timeline.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                            | Where                                                                                                                                                                                       |
| ---------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Built-in sounds                    | `public/sfx`, listed in `compile/src/data/sounds.json` (catalogue `sounds`)                                                                                                                 |
| Built-in music                     | `public/music` (128 kbps AAC), listed in `compile/src/data/music.json` (catalogue `music`)                                                                                                  |
| Credits                            | `THIRD_PARTY_NOTICES.md`                                                                                                                                                                    |
| Mix settings and the book          | `types/src/mix.rs` → `mixer_book` (catalogue `mixer`)                                                                                                                                       |
| Mix plan                           | `compile/src/mix/plan.rs` → `build_mix_plan`, `lookup_key`; `compile/src/mix/levels.rs` → `audible`, `duck_keys_for`, `duck_spans`, `with_master_stem`                                      |
| Filter graphs                      | `media/src/mix/graph.rs` → `pack_lanes`                                                                                                                                                     |
| Stems                              | `media/src/mix/render.rs`; `.opsis/cache/mix/` (`MIX_DIR`, `types/src/project.rs`)                                                                                                          |
| Channel pruning                    | `compile/src/edits/workflow.rs` → `remove_nodes`; `types/src/workflow.rs` → `Workflow::normalized`                                                                                          |
| Export sound                       | `with_master_stem`; `media/src/codecs.rs` → `mux_args`                                                                                                                                      |
| Stem player and live Mixer changes | `crates/audio/src/output.rs` → `Output`, `Preview`, `set_preview`; `crates/ui/src/mixer.rs` → `Previews`, `share_of`; `crates/ui/src/ui.rs` → `preview_mix`; `crates/qt/cpp/mixer_panel.cc` |
| Placing sounds on marks            | `compile/src/nodes/audio.rs` → `anchor_sounds` (via `flatten_spec`); `types/src/timeline.rs` → `Anchor`                                                                                     |
| Sound file filters                 | `media/src/executors.rs` → `filter_sound`; builders in `media/src/filters.rs`; real-FFmpeg checks `media/tests/sound_fixes.rs`                                                              |
| Clean speech                       | `noise_level_listen_filter`, `noise_level`, `spectral_floor`, `speech_filter`; `media/src/denoise.rs` → `denoised`                                                                          |
| Remove hum                         | `hum_listen_filter`, `rms_level`                                                                                                                                                            |
| Shorten pauses                     | `silence_listen_filter`, `silences`, `joined_parts_graph`; `compile/src/nodes/media.rs` → `kept_parts`                                                                                      |
| Change pitch, voice effects        | `pitch_filter`, `voice_filter`; rooms `media/src/generate.rs` → `room_response`                                                                                                             |
| Fit music to the film              | `fit_graph`, `fit_inputs`; `compile/src/nodes/media.rs` → `fit_plan`                                                                                                                        |
| Sound picture                      | `sound_picture_graph`                                                                                                                                                                       |
| Generated sounds                   | `compile/src/nodes/audio.rs` → `sound_shape`; `media/src/sound_maker.rs` (`fundsp`)                                                                                                         |
| Room tone, tones                   | `media/src/generate.rs`                                                                                                                                                                     |
| Transition and action sounds       | `transition_cues`, `transition_sound_file`, `FlatTimeline.cuts`, `hits`, `AudioValue.hits`, `AudioItem.hit`, `AudioBar.hits`                                                                |

## Rules

### Sounds and songs

- **Never add a sound or a song without a checked free licence**, and never link to a remote sound at render time. Built-in sounds and songs are CC0 files with their credits in `THIRD_PARTY_NOTICES.md`.
- **Sounds take only `audio`**: a file becomes a sound through Music or Sound effect, which own its volume and timing.
- **Music's Song field is the `music` widget and gives Music its length when no file is connected.**
- **Sound effects carry `role: "effect"` and their natural `lengthFrames`.**
- **A Sound effect with On an action stores the action's node id; Add sound places it at that action's mark in the clip** (`anchor_sounds`), plus an offset that may be negative.
- **An action or a clip's end is an `Anchor` (`node_id`, or `clip_end`) that Add sound resolves**; `keep_end` makes a start before the clip trim the head instead of shifting the sound, so it still ends on time. A marker's time counts from the start of the clip the sound is added to.

### Mixer

- **Mix settings are `workflow.mix`**: a strip per channel keyed by node id (Music, each Sound effect, each clip with sound), buses (Music, Effects, Clips built in, plus user buses) with sidechain ducking, and a master with EQ, compressor, limiter and loudness target.
- **The ranges, defaults, built-in buses, loudness targets and strip presets are Rust's (`mixer_book`)**; each channel's bus, strip and whether it is heard come from the mix plan, and the compile result carries the resolved mix and whether solo is on.
- **Removing nodes prunes their channels** (`remove_nodes`; `Workflow::normalized` on open and replace).
- **`build_mix_plan` and the filter graphs are pure; the graphs are tested by string.**
- **Stems render into `.opsis/cache/mix/`, content-hashed, with recent plans kept**: channel, bus and master stems with per-frame peak and RMS, compressor gain reduction and EBU R128 loudness.
- **Exports always play the master stem** (`with_master_stem`): the picture is drawn once without sound and the master stem added with FFmpeg (`mux_args`), so a mix or sound-only change never renders the picture again ([runs.md](runs.md)).
- **Mute and solo are saved and apply to exports, on strips and buses alike** (`audible`): a muted strip or bus is silent, and while anything is soloed only soloed strips and the strips on soloed buses are heard.
- **Strip selection is the editor's node selection** (Ctrl/Cmd-click toggles, Shift-click takes the range in strip order); a strip menu on a selected strip applies to every selected strip in one `MixEdit`, one undo step.
- **A Mixer drag is heard at once, before the new stem is ready**: `preview_mix` weighs the change by the strip's share of the master at the playhead (`share_of`), `Previews` adds it to the other pending changes, and the stem player applies the total as one `Preview` (`set_preview`).
- **The stem player plays through the default sound output (`Output`), and with none playback stays silent**; builds with debug assertions take another output from `OPSIS_AUDIO_DEVICE` ([AGENTS.md](../AGENTS.md) §3).
- **What `MIX_VERSION` keys is in [runs.md](runs.md).**

### Ducking

- **Ducking is a bus setting; rendered stems use `sidechaincompress`.**
- **A bus is keyed only by the audible channels on the buses its ducking names, never by its own** (`duck_keys_for`); channels shorter than its `ignoreShorterMs` and `keepMusic` sound effects never key it, so muting or soloing changes the ducking too.
- **Each ducked sound carries its merged spans (`duck_spans`, `Duck.spans`)**: the mix applies them, and the Timeline draws the ducked level by scaling the waveform with `duck_gain` (`crates/compile/src/timeline/media.rs`).

### Hits on actions and transitions

- **Action sounds (`audio.actionSounds`, `clip → sound`) puts its click sound at every press of a Click or Type and its key sound at every Press keys chord and typed key** in its clip's takes; `flatten_spec` lists them as `hits`, mapped through trims, loops, remaps and speed like the action marks, and the take logs each typed key's time (`keyTimes`).
- **Transition sounds (`audio.transitionSounds`, `clip → sound`) puts a sound at every transition of its clip, centred on its middle** (`transition_cues` over `flatten_spec`). A generated sound is made when the node runs, one file per transition length when Fit is on (`transition_sound_file`), and compile finds each in the node's artifacts.
  - Plain cuts have no transition mark, so Also at plain cuts adds one at each (`FlatTimeline.cuts`, every clip start after the first with no entering transition, never sent to the editor), with the variation's own length.
- **Hits are one item per hit on the node's own Effects channel, never keying the ducking, each with a seeded small change of pitch (`AudioItem.pitch`, `asetrate` in the mix) and level**, so the Timeline and the export sound the same.
- **An action's No sound (`silent`, logged with it) leaves it out**; the card counts hits from the node's own output (`AudioValue.hits`, `Hits.transitions`).
- **The Timeline shows a node's hits as one bar with a tick per hit (`AudioItem.hit`, `AudioBar.hits`) whose only edit is its volume.**
- **A channel with more than `LANE_AFTER` one-shot sounds plays them on a few lanes** (`pack_lanes`), each sound padded to the next one's start and joined with `concat`, instead of one `adelay` and `amix` input each: 500 key sounds render in about 2.5 s this way and about 24 s the other.

### Sound file nodes

- **Each sound file node is one filter chain through `filter_sound`**: a sound file keeps its format and sample rate, a video keeps its picture (copied when it can be), and a file with no sound fails with the node's own message.
- **Shorten pauses works on sound files only**, because a video would lose its sync; parts are joined with short crossfades (`PAUSE_CROSSFADE_S`) and the time saved goes to the run log.
- **Listening passes read levels from FFmpeg's log**: Remove hum's Find it reads 50 and 60 Hz and the louder wins; Clean speech reads astats every 20 ms (960 samples at 48 kHz, `noise_level_listen_filter`) and takes the quietest share (`QUIET_SHARE`) as the noise level.
- **Clean speech removes noise the learned way or the spectral way, then cuts rumble, softens "s" sounds and evens the level in one pass.**
  - Learned: the voice decoded to 48 kHz mono, `nnnoiseless` frame by frame with its one-frame delay taken back, mixed with the original in even decibel steps by strength, and passed back to `filter_sound` as the sound under the file's picture with its channels restored.
  - Spectral: `afftdn` told a floor `FLOOR_ABOVE_NOISE_DB` above the measured level (`spectral_floor`).
  - The evening skips half-waves below 1% so it never lifts the noise in the gaps; the levels before and after go on the result as `Measured::NoiseLevel`.
- **Change pitch uses `rubberband` with `transients=mixed`**, because the default resets the phase at a sudden start and plays the first 0.4 s about 7 dB quieter.
- **`rubberband` drops its last block (69 ms), so Change pitch (when the file's length is known) and Fit music's stretch pad silence (`PITCH_TAIL_S`) before it and trim back to the file's length after it.**
- **"Keep the voice's character" preserves formants**, which suits voices but nearly silences a pure tone moved far.
- **Voice effects blend with the original through `asplit` and `amix`**, because afir's own `dry` and `wet` are input and output gains, not a mix. The telephone is a 4th-order band of 300 to 3400 Hz.
- **Rooms are generated, never recordings (`room_response`)**: seeded early reflections and a noise tail that falls 60 dB over the reverb time with the highs fading faster, at unit energy and different on each side, read by `amovie` into `afir` with `gtype=none:irnorm=-1`, because afir otherwise normalises the response and a room comes out about 26 dB quiet.
- **Fit music to the film takes the film's clip for its length**, so it keys on the project size and frame rate; `fit_plan` stretches within the largest stretch, loops the chosen section with equal-power crossfades when that is too far, or cuts a longer song, and `fit_graph` ends on the film's length with a fade. The card reads `Measured::Fitted`.
- **Each loop piece is its own input of the same file (`fit_inputs`)**, because `acrossfade` drops the second of two branches split from one input.
- **Sound picture outputs an `image`** (waveform or spectrum with a named colour scale, no axes, over a background colour), so the image nodes take it.

### Generated sounds

- **Generated sound (`audio.generate`) is made when it runs, from `sound_shape`**: the preset and its variation become a length, pitch, brightness and take; Default is the variation chosen by ear for each preset and Custom shows the four sliders. It is drawn with `fundsp` graphs, peak-normalised (`PEAK`, `media/src/sound_maker.rs`), as a 16-bit WAV.
- **`fundsp`'s `envelope` is sampled every 2 ms and every sound fades in (`EDGE_SECONDS`), which turns anything shorter into a soft tap**, so the click shapes its press and release per sample and fades in over its own shorter edge (`CLICK_EDGE_SECONDS`).
- **Change a preset only after listening to it.**
- **Room tone and tones are made in Rust from a seeded generator, so a node always makes the same file.**
  - Room tone is a WAV of `LOOP_SECONDS` (`media/src/generate.rs`) whose ends crossfade so it loops without a seam, normalised to the level asked, played looped under the whole film on its own Music channel that never keys the ducking.
  - Tones have soft edges (`EDGE_SECONDS` in `media/src/generate.rs`) so they never click, and are placed like a Sound effect.
- **Riser to a mark ends a riser or swell on an action, a marker or the clip's end, with an optional short hit on that point**, through an `Anchor`.
