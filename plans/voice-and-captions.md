# Voice and captions

What Opsis needs to speak a script and to caption speech: a Voiceover node that turns text into a voice on this computer (or, when the user turns it on, through an online service), captions from the speech in a file, a film that fits the voice's length, and a Settings page for the voices and speech models. Captions from script, Show captions, the Mixer and its ducking are already built.

Status: plan only. None of the work below is built.

## How to read this plan

- Each section is one feature. The user picks which to build and when; the first section (models, downloads and Settings) comes before the others because they all use it.
- Each feature section has the same parts: **What it adds**, **Fit with Opsis** (In place / Missing), **Work**, **Licence**, **Risks**, **Sources**.
- Sizes are download sizes read from the model listings on the date in **Sources**. Sizes on disk are measured during the work and written into the catalogue.

## Conventions

These apply to every feature in this plan. They restate rules from `AGENTS.md` and `agent-docs/`; when they disagree with this plan, they win.

**Where code goes.**

- `crates/voice` (`opsis-voice`): the model catalogue and downloads, the speech models, word timing and the online services. Its executors register with the runner like `opsis-media`'s, in both lists that name every executor crate: `executors` in `crates/session/src/run.rs` and `all_executors` in `crates/render/tests/executors.rs`; its nodes are definitions in `opsis-compile`; keys come from `opsis-store`. The crate gets its row in the AGENTS.md crate table, and its rules go in `agent-docs/audio.md` or a new `agent-docs/voice.md` listed in AGENTS.md §8.
- Node definitions in `crates/compile/src/nodes/` (settings structs with `schemars` attributes, `card`, `compile`). Every node with `executes: true` has an executor (`crates/render/tests/executors.rs`).
- Speech never runs in the renderer or the editor.

**One inference runtime (decided).** Every model on this computer runs through `ort` (MIT OR Apache-2.0, ONNX Runtime). Nothing else may carry its own inference runtime: no sherpa-onnx (its crate links its own `onnxruntime`, and its program archives ship `libonnxruntime.so`), no whisper.cpp (`whisper-rs` builds ggml).

**Behaviour.**

- Nothing uses the internet unless the user asks. Every download sits behind a button, says what it gets and how big it is, asks first, and saves under `<data folder>/speech`. Online voices and transcription are off until the user turns a service on in Settings and enters a key.
- Synthesis and transcription happen when a node runs, never at render time. The Timeline's playback and the export play the same saved files, so what the Timeline plays is what the export has.
- Keys live in the OS keyring (`Keyring::set_key`, `crates/persistence/src/keys.rs`), never in settings, never in a window.

**User-facing copy.**

- No library, model, tool or format names in node titles, descriptions, field labels, badges, toasts or errors: "Voice on this computer", "Online voice", "English", "European languages". Sentence case, durations in seconds, technical settings under `x-advanced`.
- Decided naming exception, added to `AGENTS.md` when this is built: Settings → Voice and captions may name the online services (ElevenLabs, OpenAI, OpenRouter) in its service choices and key fields. Local voices and models show plain labels ("Natural voices", "English", "European languages"); each label has a small circled `?` whose hover card names the model and gives size, languages and licence. Names appear nowhere else (nodes, cards, toasts, errors), and in Settings → Licences. `plans/ai-assist.md` decides its own exception for its provider choice (OpenRouter there sits under Compatible service); one AGENTS.md exception covers both pages.
- `crates/compile/tests/unit/nodes_c.rs` checks every node's description and the banned words; add Kokoro, Parakeet, NeMo, ElevenLabs, OpenAI, OpenRouter and ONNX to its list.

**Licences.** Downloaded models get a licence entry that appears once they're on disk, like the engine's (`LicenceSource`, `crates/engine/src/licences.rs`). Never offer a voice whose licence hasn't been checked for commercial use, the same rule as built-in sounds.

**Checks when a feature is done.**

- `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`, and the no-comments check.
- Try it in the app: download from Settings, run the node, play it on the Timeline, change the Mixer.
- Export a short film that uses it and compare it with the Timeline's playback: the export must sound the same.

## Summary

| Feature                         | What users get                                                                                                                                                                     | Licence                          | Size                                                   |
| ------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------- | ------------------------------------------------------ |
| Speech models and Settings page | A Voice and captions page: voices and speech models with their sizes, download and remove buttons, online services with a key in the keyring                                       | ONNX Runtime MIT                 | the runtime, measured in step 1                        |
| Voiceover node                  | A voice reading a script with variables, line by line, at a set speed; lines can start on browser actions; its own Voice bus that lowers the music; captions that follow the voice | Apache-2.0 (Kokoro)              | about 107 MB (int8 model and the English voices)       |
| Online voices and transcription | The same nodes using ElevenLabs, OpenAI, OpenRouter or any compatible address, after the user turns it on and enters a key                                                         | Service terms                    | Nothing to download                                    |
| Captions from speech            | Word-timed captions from the speech in a video or sound file, for Show captions, plus a subtitle file                                                                              | CC-BY-4.0 (transcription models) | about 661 MB (English), 671 MB (25 European languages) |
| Fit length to the voiceover     | Fit length holds a tail after the sound                                                                                                                                            | Opsis's own                      | None                                                   |

## Speech models, downloads and the Voice and captions page

### What it adds

A Settings page, **Voice and captions**, that lists everything speech needs and gets it only when asked:

- **Voices on this computer**: "Natural voices" with its languages, number of voices, download size and size on disk; Download, Remove, and Play a sample once downloaded. A circled `?` opens a hover card with the model name, licence and languages.
- **Captions on this computer**: "English" and "European languages" (25 languages), with their sizes and the same `?` card (NVIDIA Parakeet TDT 0.6B v2 and v3, CC-BY-4.0).
- **Online services**: off by default. ElevenLabs, OpenAI and OpenRouter each get a switch, a key field and a Check key button; **Compatible service** takes an address, a voice model, a transcription model and an optional key, for any service or local speech server that speaks OpenAI's audio API.
- The folder it all lives in, and the total space used.

### Fit with Opsis

In place:

- The engine installer (`crates/engine/src/install.rs`): `fetch_verified` streams a pinned download into a temporary file next to its destination, checks size and SHA-256, and honours cancellation; `CalmProgress` throttles progress; downloads are pinned as `Pinned { url, sha256, size }` (`crates/engine/src/downloads.rs`). Settings → Engine shows the install with progress events.
- Ask-first downloads: the font library's "Go online to get fonts?" question (`crates/qt/cpp/fonts_page.cc`).
- Settings pages: `SettingsPage` (`crates/types/src/shell.rs`) and `crates/qt/cpp/settings_window.cc`; app settings in `AppSettings` (`crates/types/src/settings.rs`), written one section at a time with `patch_settings`.
- Keys: `Keyring::set_key` / `key` keep named secrets in the OS keyring and refuse when the platform has no secure store.
- Long work is listed on exit through `AppState.tasks`.
- Licence entries that appear once a file exists: `LicenceSource` and `list_licences` (`crates/engine/src/licences.rs`).

Missing: the crate, the catalogue, the runtime, the page, the `speech` settings section, a key removal call.

### Work

1. **Runtime.** Pin `ort` in the root `Cargo.toml`'s `[workspace.dependencies]`, then `cargo add -p opsis-voice ort` (the same for `misaki-rs` and every other new crate here). Choose how ONNX Runtime ships by measuring the app both ways (`plans/release.md`, size work): linked statically from ort's prebuilt binaries (`download-binaries`, fetched at build time) or one shared library staged with the app and loaded with `load-dynamic`, which ort's docs recommend for apps. Either way it is one runtime for every model. A test loads each model's input and output names, so a model that needs an operator the build lacks fails in CI.
2. **Catalogue.** `crates/voice/src/catalog.rs`: `VOICE_PACKS` and `TRANSCRIBE_MODELS` as Rust data (id, label, languages, voices with display names, files with URL pinned by Hugging Face commit, SHA-256 and size, licence), and `OPENAI_VOICES`, the fixed list of OpenAI's built-in voices with display names (Online voices, step 3), exported to the editor with the other generated types. A test checks every entry has a hash, a size, a checked licence and no banned word in its labels.
3. **Paths and status.** `speech_paths(data)` → `<data folder>/speech/{voices/<id>,models/<id>}`; `speech_status()` reports what's installed.
4. **Installer.** `install_speech_item(id)`: each file through `fetch_verified` (made public in `opsis-engine`) into a staging folder, renamed into place when all are there; `remove_speech_item(id)`. Models are plain files, so nothing is unpacked.
5. **Session calls.** `install_speech(item)` on the bridge, listed in `AppState.tasks` ("Downloading speech files"), with progress the way engine setup reports it: Rust posts an event and the page reads its view again (`install_engine`, `engine_view`, `crates/qt/cpp/engine_card.cc`); `remove_speech(item)`; `speech_status`.
6. **Keys.** `Keyring::clear_key` (in `crates/persistence/src/keys.rs`); keys named `speech:<service>`. `Session` calls `set_service_key(service, key)`, `has_service_key(service)`, `clear_service_key(service)`; views never read a key back.
7. **Settings.** `AppSettings` gains `speech: { voice, transcribe: english | european, online: { elevenlabs: { enabled, ask_first }, openai: { enabled, ask_first }, openrouter: { enabled, ask_first }, compatible: { enabled, ask_first, address, voice_model, transcribe_model } } }`, where `ask_first` is cleared by "Don't ask again" (Online voices, step 5), all off or empty by default. Add `SettingsPage::Voice`, its page in `crates/qt/cpp/settings_window.cc` and its row in the page list ("Voice and captions", icon `audio-lines`). Download buttons ask in a dialog naming the size before anything is fetched. The `?` cards are one small hover card behind a `?`, fed from the catalogue entry.
8. **Licences.** `LicenceSource` entries for each voice pack and each transcription model (CC-BY-4.0, crediting NVIDIA), shown once downloaded; ONNX Runtime's notice in `THIRD_PARTY_NOTICES.md`.
9. **Tests.** Catalogue test; `speech_paths` per platform; a hash mismatch rejects the download and leaves nothing behind (against `wiremock`).

### Risks

- **Runtime size.** ONNX Runtime adds tens of MB to every install even for users who never speak; `load-dynamic` with the library downloaded beside the models would avoid that, at the cost of one more pinned download per platform.
- **macOS.** A shared library loaded at run time must be signed with the app; check on a notarized build (`plans/release.md`, signing).
- **No keyring.** Saving a key refuses without a secure store; Settings shows that where the key is entered, and the service stays off.

### Sources

- crates.io / docs.rs: `ort` 2.0.0-rc.15 (MIT OR Apache-2.0, ONNX Runtime 1.31; `download-binaries`, `load-dynamic`), `sherpa-onnx-sys` 2.0.0 (`build.rs` links `onnxruntime`), `whisper-rs` 0.16.0 (builds ggml) (checked 2026-10-25)
- [sherpa-onnx v1.13.8 release](https://github.com/k2-fsa/sherpa-onnx/releases/tag/v1.13.8): `linux-x64-shared` ships `lib/libonnxruntime.so` (checked 2026-10-16)

## Voiceover node

### What it adds

A **Voiceover** node (Audio): a script, a voice and a speed in, a voice track and its captions out.

- **Script**: one line per spoken line, with variables (`VarText`). Each line is spoken separately, so its start and end are exact.
- **Voice**: a picker of the pack's voices with a play button, and online voices when a service is on.
- **Speed**, **Starts at**, **Pause between lines**, and a per-line nudge that the Timeline writes when a line is dragged.
- **Lines on actions**: any line can start on a browser action of the clip it's added to, plus an offset, like Sound effect's **On an action**.
- Outputs: **Sound** (to Add sound or Fit length) and **Captions** (to Show captions), so captions follow the voice word by word.

### Fit with Opsis

In place:

- Sounds join a clip through Add sound (`audio.mix`, `crates/compile/src/nodes/audio.rs`), which takes any number of `audio` values (`many()`) and places anchored ones at their action's mark with `anchor_sounds` (via `flatten_spec`). Sound effect stores its action through the `action` widget and an offset.
- `AudioItem` (`crates/types/src/timeline.rs`) carries `channel`, `role` (`AudioRole`: Music, Effect), `length_frames`, `anchor` and `handles`; each `channel` is one Mixer strip.
- Ducking is a bus setting: `duck_keys_for` (`crates/compile/src/mix/levels.rs`) keys a bus on the buses it names; the mix plan renders it with `sidechaincompress`. A **Voice** strip preset exists in the Mixer (`strip_presets`, `crates/types/src/mix.rs`).
- `CaptionsValue` (`crates/types/src/values.rs`) is a list of `Caption { text, start_ms, end_ms }` per word; Show captions (`text.captions`, `crates/compile/src/nodes/text.rs`) draws it with `captions`, whose component groups words into lines of up to six or at sentence ends.
- Media executors run FFmpeg behind `MediaTools` and return files (`crates/media/src/executors.rs`); a compile can read its executor's result.

Missing:

- Any text-to-speech. Recordings carry no sound, so all speech comes from this node or from files.
- A voice role: `AudioRole` (`crates/types/src/timeline.rs`) knows music and effects, and `ChannelKind` (`crates/types/src/mix.rs`) music, effects and clips; `default_buses` has Music, Effects and Clips, so there's no Voice bus to duck under.
- Captions can't follow an action: `Caption` has no anchor, and Show captions doesn't resolve marks.
- Icons: the `Icon` enum (`crates/types/src/cards.rs`) has `AudioLines` and `Captions` but no microphone or speech icon.

### Work

1. **Kokoro in Rust.** `crates/voice/src/kokoro.rs`: text to phonemes with `misaki-rs` (MIT; grapheme-to-phoneme made for Kokoro, US and British English) with `default-features = false`, so espeak isn't linked; phoneme ids from the model's `tokenizer.json`; `input_ids` padded with 0, at most 510 tokens per call; `style` = the voice file's vector at index `len(tokens)` (256 values); `speed`. One `ort` session per process, kept while the app runs.
2. **Voice pack.** "Natural voices" = `onnx-community/Kokoro-82M-v1.0-ONNX`: `onnx/model_quantized.onnx` (92.4 MB) and the 29 US and British English voices (`af_*`, `am_*`, `bf_*`, `bm_*`, about 0.5 MB each). Compare it with `model.onnx` (325.5 MB) by ear before the catalogue is final. Other languages and the small Kitten voices need espeak-ng phonemes; they are out of the first version and could come later with espeak-ng as a separate program that only turns text into phonemes (a phonemizer, not a second runtime, and the GPL boundary stays a process boundary). Piper voices are out (decided): licences differ per voice (`ryan` is CC BY-NC-SA 4.0, `lessac` uses the Blizzard licence).
3. **Node.** `audio.voiceover` in `audio.rs`: media phase, `executes: true`, no inputs, outputs `audio` and `captions`. Settings: `script` (`textarea`), `voice` (a new `voice` widget: catalogue voices, installed or not, with a link to Settings), `speed` (0.5–2, step 0.05), `start_ms`, `pause_ms` ("Pause between lines", 0–5 s, default 0.4 s), `nudges_ms` (a new `line-nudges` widget with one field per script line; the `offsets` widget doesn't fit, since it is Sequence's per-layer field and counts its rows from the node's layers, `InspectorContext.layers`), `line_actions` (rows of `{ line, action, offset_ms }` with a new `line-actions` widget that lists the script's lines with the action picker, matched by line text so reordering keeps them), and `word_timing` under advanced ("From the voice" when a transcription model is installed, else "Estimated"). Card: first line quoted, voice name, total length, "N lines on actions"; a warning when the voice isn't downloaded.
4. **Executor.** In `opsis-voice`, registered like the media executors: for each line, synthesise to WAV in a shared cache `.opsis/cache/voice/<sha256 of pack, voice, speed, text>.wav`, so editing one line re-speaks only that line; probe each length; word times from `word_timing` (step 6). Outputs `{ lines: [{ src, duration_ms, words }] }` as JSON in the node's folder, plus the whole voice as an `audio` result for the card.
5. **Compile.** From the result and the settings (so moving or pausing never re-speaks): one `AudioItem` per line with `channel` = the node id, `role: Voice`, `length_frames`, `anchor` from `line_actions`, `from` = start + earlier lines + pauses + nudges, `handles.move` pointing at the line's entry in `nudges_ms` (or its `offset_ms` when anchored). `AudioValue.duration_in_frames` = end of the last unanchored line. `CaptionsValue` with each word's times, shifted the same way.
6. **Word timing.** "Estimated": spread the line's length over its words by character count, with a short lead-in; a pure function in `crates/voice/src/words.rs`, unit-tested. "From the voice": transcribe each line (Captions from speech's code) and keep its word times when the words match the script, else fall back to the estimate.
7. **Voice role and bus.** `AudioRole::Voice` and `ChannelKind::Voice` with `VOICE_BUS`; `default_buses` gains a Voice bus starting from the Voice preset's EQ and compressor; `sound_length` (`crates/compile/src/audio.rs`) treats voice like effects (natural length); the flattener draws it. Mixes saved without the Voice bus get it in `Workflow::normalized` (`crates/types/src/workflow.rs`), which today only prunes channels. When the first Voiceover is added, the Music bus's ducking is keyed on the Voice bus if it's off, in the same undo step. `bun run types:export`.
8. **Captions on actions.** `Caption` gains an optional `anchor`; Show captions resolves them against its **Over clip** input with the same marks as `anchor_sounds`, whose lookup of an action's frame moves into a shared function (today it is inline in `anchor_sounds`; `mark_frame` exists only in `crates/compile/tests/unit/action_sounds.rs`). Without a clip, anchored lines use their unanchored times and the card says so.
9. **Icons.** `MicVocal` and `Speech` in `Icon` (`crates/types/src/cards.rs`) and in `ICONS` (`scripts/node-glyphs.ts`); a look on the node's definition (`.look(…)`), then `bun run glyphs:bundle`.
10. **Tests.** The executor exists; descriptions and banned words; compile placement with pauses, nudges and anchors; a round-trip for the line move handle with the other timeline edit tests; ducking keys include the voice; the Voice bus is added once.
11. **Check.** A film with music, a voiceover of five lines (two on actions) and Show captions: the music dips under each line, captions follow the words, dragging a line on the Timeline moves its captions, re-running after editing one line re-speaks only that line, and the export sounds like the Timeline's playback.

### Licence

Kokoro-82M weights: Apache-2.0 (trained on public-domain, Apache/MIT and two CC BY datasets, credited in the notice). `misaki-rs`: MIT. No GPL part: espeak isn't linked.

### Risks

- **Speed on slow computers.** Measure the int8 model on the CI machines and a 4-core laptop; if it's slower than real time on the laptop, say so next to the voice and speak lines in parallel up to the core count.
- **Numbers, names and URLs.** `misaki-rs` without the espeak fallback spells unknown words letter by letter, and the model doesn't normalise "v2.1" or "$29/mo". Add a small, tested text tidy-up (`crates/voice/src/tidy.rs`) with **Fix words**-style replacements, and show the tidied line as a hint under the script.
- **Long lines.** Very long lines lose pace; split lines over about 400 characters at sentence ends before speaking.

### Sources

- [onnx-community/Kokoro-82M-v1.0-ONNX](https://huggingface.co/onnx-community/Kokoro-82M-v1.0-ONNX): `onnx/model_quantized.onnx` 92.4 MB, `model.onnx` 325.5 MB, 55 voices in 28.7 MB, 29 of them English (Hugging Face API, checked 2026-10-26)
- [hexgrad/Kokoro-82M](https://huggingface.co/hexgrad/Kokoro-82M) (Apache-2.0 weights, 82M parameters, training data) (checked 2026-10-16)
- crates.io: `misaki-rs` 0.6.0 (MIT, features) (checked 2026-10-25)
- [rhasspy/piper-voices](https://huggingface.co/rhasspy/piper-voices) model cards for `lessac`, `ryan` (checked 2026-10-16); GitHub licence API: `espeak-ng/espeak-ng` GPL-3.0 (checked 2026-10-16)

## Online voices and transcription (opt-in)

### What it adds

The Voiceover node and Captions from speech can use an online service instead of this computer, once the user turns the service on in Settings and enters a key. Four choices: ElevenLabs, OpenAI, OpenRouter (its address filled in and its transcription format handled), and **Compatible service**, any address that speaks OpenAI's audio API, including speech servers the user runs on their own computer. Voices from the service appear in the voice picker under "Online voices". Results are saved in the node's cache, so a run with nothing changed sends nothing.

### Fit with Opsis

In place: the keys and the ask-first pattern from the first section; `reqwest` is already used by `opsis-engine` and `opsis-fonts`.

Missing: request code for the services, a voice list cache, a first-use confirmation, and converters from each service's timings to `Caption`s.

### Work

1. **Requests** in `crates/voice/src/online.rs`, with `reqwest` and the key read from the keyring on a blocking thread, as `off_runtime` does for sign-ins (`crates/capture/src/take.rs`), since the Linux keyring blocks and panics on a runtime thread. The keyring reaches the executors the way saved sign-ins reach capture: a trait of `opsis-voice` that the session implements over its keyring and hands to the executors it registers (`executors`, `crates/session/src/run.rs`, as `ProjectSignIns` implements `SignIns`).
   - ElevenLabs voice: `POST /v1/text-to-speech/{voice_id}/with-timestamps` (`xi-api-key` header, `voice_settings.speed`); decode `audio_base64`, turn `alignment` (per-character start and end times) into word times with a pure, tested function.
   - OpenAI voice: `POST /v1/audio/speech` with `gpt-4o-mini-tts`, a built-in voice and `response_format: "wav"`. It returns no timestamps and documents no speed setting, so word timing uses the estimate or the transcriber, and Speed is applied with FFmpeg's `atempo`.
   - ElevenLabs transcription: `POST /v1/speech-to-text` with `model_id: "scribe_v2"` and `timestamps_granularity: "word"`, words read into `Caption`s.
   - OpenAI transcription: `POST /v1/audio/transcriptions` with `whisper-1`, `response_format: "verbose_json"` and `timestamp_granularities[]=word` (only `whisper-1` has word timestamps), multipart.
   - OpenRouter voice: `POST https://openrouter.ai/api/v1/audio/speech`, OpenAI-compatible (`model`, `input`, `voice`), raw audio back (MP3 or PCM). Models come from its model list filtered by `output_modalities`; no timestamps, so timing works as for OpenAI.
   - OpenRouter transcription: `POST https://openrouter.ai/api/v1/audio/transcriptions` with JSON `{ model, input_audio: { data: <base64>, format } }`; its docs disagree on whether OpenAI-style multipart uploads work, so use the JSON form. Without word timestamps in the response, captions are timed by the local transcriber when installed, else estimated, and the card says so.
   - Compatible service: the OpenAI requests against the user's address (`{address}/audio/speech`, `{address}/audio/transcriptions`), with the user's model names and the key when one is set. Use word timestamps when the response has them, else fall back as for OpenRouter. Check key sends one tiny request and reports what the server supports.
   - `async-openai` (MIT) can replace the OpenAI request types if its request for transcriptions carries `timestamp_granularities[]`; otherwise plain `reqwest` multipart.
2. **Converters.** The ElevenLabs alignment → words, the OpenAI `verbose_json` words and SRT writing are small Rust functions in `crates/voice/src/captions.rs`, tested on recorded responses.
3. **Voice list.** `GET /v1/voices` for ElevenLabs and OpenRouter's model list only when the user opens the picker with the service on; cached in `<data folder>/speech/online-voices.json`. OpenAI's list is `OPENAI_VOICES` in the catalogue (first section, step 2); Compatible service uses the voice names the user types.
4. **Choosing a service in a node.** Voiceover's `voice` holds an online voice as `<service>:<voice>`, so picking an online voice picks its service. Captions from speech gains `transcribe_with` (advanced): this computer, the default, or any service that is turned on.
5. **Asking first.** The first run that would send text or sound to a service shows a confirmation in the Run panel naming what is sent; "Don't ask again" is per service in `settings.speech.online`.
6. **Errors.** Plain messages without service names in nodes ("The online voice refused the key. Check it in Settings."); the HTTP status goes to the app's log.
7. **Tests.** Alignment → words; recorded responses through the converters; no request is made when the service is off (against `wiremock`).

### Licence

Use of the services is under each service's terms with the user's own key; Opsis ships nothing from them.

### Risks

- **Compatible servers vary.** Some ignore `speed`, some return no word timings, some need no key. Check key records what the server supports, and the node falls back instead of failing.
- **Size limits.** Check the services' current upload limits before sending; for long films, send mono speech-band audio made with FFmpeg instead of the original file.
- **Cost surprises.** Show the script's character count next to an online voice; re-runs reuse the line cache.

### Sources

- [ElevenLabs: Create speech with timing](https://elevenlabs.io/docs/api-reference/text-to-speech/convert-with-timestamps) and [Speech to text](https://elevenlabs.io/docs/api-reference/speech-to-text/convert) (checked 2026-10-16)
- [OpenRouter: New audio APIs](https://openrouter.ai/blog/announcements/announcing-audio-apis/), [Text-to-speech](https://openrouter.ai/docs/guides/overview/multimodal/tts), [Create speech](https://openrouter.ai/docs/api/api-reference/tts/create-speech), [Speech-to-text](https://openrouter.ai/docs/guides/overview/multimodal/stt) and the [OpenRouter STT skill](https://github.com/OpenRouterTeam/skills/blob/main/skills/openrouter-stt/SKILL.md), which says the endpoint isn't OpenAI-compatible (checked 2026-10-17)
- [OpenAI: Text to speech guide](https://developers.openai.com/docs/guides/text-to-speech) and [Speech to text guide](https://developers.openai.com/docs/guides/speech-to-text) (`timestamp_granularities` only for `whisper-1`) (checked 2026-10-16)
- crates.io: `async-openai` 0.42.2 (MIT, configurable base URL, `audio` feature) (checked 2026-10-25)

## Captions from speech

### What it adds

A **Captions from speech** node (Text & overlays): a video or sound file in, word-timed captions out, transcribed on this computer (or online, when turned on). Settings: **Language** ("English" or "European languages", from the downloaded models), **Fix words** (a find-and-replace list applied after, for names and product terms), and **Starts at**. It also saves a subtitle file next to its result.

### Fit with Opsis

In place:

- The captions pipeline: `CaptionsValue`, the `captions` port, Show captions, and Captions from script (`text.captionsFromText`) as the model for the node's shape.
- Files reach nodes as `MediaValue` on `media` ports from Media file, Extract sound and the other media nodes; media executors get the file's path and run FFmpeg with progress.

Missing: the transcription model in Rust, the node, its executor, and turning tokens into words.

### Work

1. **Parakeet in Rust.** `crates/voice/src/parakeet.rs`, everything in the same `ort`:
   - Features: `nemo128.onnx`, NeMo's 128-band log-mel preprocessing (512-point STFT, 400-sample window, 160-sample hop, Slaney mel filter bank, per-feature normalisation) written as an ONNX graph by onnx-asr (MIT).
   - Decoding: a TDT greedy loop: `encoder-model.int8.onnx` once per piece, then `decoder_joint-model.int8.onnx` per step, taking a token and a duration (frames to skip) until the audio ends. onnx-asr's Python implementation is the reference.
   - Tokens: `vocab.txt` pieces (`▁` starts a word) and each token's encoder frame (16 kHz audio, 160-sample hop, subsampling factor 8 from `config.json`: 80 ms per frame).
2. **Models.** `istupakov/parakeet-tdt-0.6b-v2-onnx` (English) and `istupakov/parakeet-tdt-0.6b-v3-onnx` (25 European languages), each `encoder-model.int8.onnx` (652.2 MB), `decoder_joint-model.int8.onnx` (9.0 and 18.2 MB), `nemo128.onnx`, `vocab.txt` and `config.json`, pinned by commit and SHA-256.
3. **Node.** `text.captionsFromSpeech` in `text.rs`: media phase, `executes: true`, input `media` ("Speech"), output `captions`. Settings above; `language` defaults to `settings.speech.transcribe`. Card: first words of the result, language, "N words"; a warning when the model isn't downloaded.
4. **Executor** in `opsis-voice`:
   1. FFmpeg to 16 kHz mono 16-bit WAV in the node folder (`-ar 16000 -ac 1 -c:a pcm_s16le`).
   2. Long files: split at silences into pieces under 10 minutes (`silencedetect`), transcribe each, and offset their times.
   3. Run the model (step 1) on each piece.
   4. `tokens_to_captions` in `crates/voice/src/words.rs` (pure, tested): a word starts at its first token's time and ends at the next word's start, capped at 0.8 s after its last token; punctuation tokens join the word before them, so sentence ends still end lines in Show captions. Then **Fix words** and **Starts at**.
   5. Write `captions.json` and `captions.srt` as results.
5. **Shared with the Voiceover.** The transcribe step is a function the Voiceover's "From the voice" word timing calls.
6. **Tests.** Golden token and frame lists recorded from onnx-asr on a few short files (checked in as test data with the script that wrote them), compared exactly with the Rust loop; `tokens_to_captions` on spaces, punctuation, fixes, offset and the 0.8 s cap; the silence split keeps times continuous; the executor exists.
7. **Check.** Import a screen recording with narration, run Captions from speech with each model, show the captions over the clip: words land within a frame or two of the speech in the export, on Linux, Windows and macOS.

### Licence

NVIDIA Parakeet TDT 0.6B v2 and v3: CC-BY-4.0 (the Hugging Face model data for `nvidia/parakeet-tdt-0.6b-v2` and `-v3`; NVIDIA's hosted versions have separate terms that don't apply to the downloaded weights). Credit NVIDIA in Settings → Licences. onnx-asr's preprocessing graph: MIT, in `THIRD_PARTY_NOTICES.md`.

### Risks

- **Languages.** The models cover English and 24 more European languages. Other languages need an online service.
- **Word ends.** TDT gives start frames, so word ends are estimated; check by eye that captions don't linger.
- **Speed and memory.** The int8 encoder is 652 MB; measure a 1-minute file and peak memory on a laptop without a fast CPU, and show the expected time from that measurement on first use.

### Sources

- Hugging Face API for `istupakov/parakeet-tdt-0.6b-v2-onnx` and `-v3-onnx` (file list and sizes; checked 2026-10-26) and for `nvidia/parakeet-tdt-0.6b-v2` and `-v3` (`license: cc-by-4.0`; checked 2026-10-17)
- onnx-asr (MIT): the Parakeet TDT decoder and `nemo128.onnx` preprocessing (read 2026-10-25)
- [NVIDIA model card](https://build.nvidia.com/nvidia/parakeet-tdt-0_6b-v2/modelcard) (English, word timestamps)

## Fit length to the voiceover

### What it adds

Fit length makes a clip as long as the voiceover, by speeding up, holding the end or cutting, with an extra **Hold after the sound** so the film doesn't end on the last word.

### Fit with Opsis

In place:

- Fit length (`timeline.durationFit`, `crates/compile/src/nodes/timeline.rs`) has an `audio` input "Fit to this sound" and uses `AudioValue.duration_in_frames` as its target. Music sets it from its file's length, and a Voiceover that sets it (Voiceover step 5) fits with no change to this node.
- Add sound cuts sounds where the clip ends, so fitting first and adding the voice after keeps every line; action marks are read after the speed change (`anchor_sounds` runs in Add sound), so lines on actions still land on them.

Missing: a tail; the card says "Fits the sound" whatever is connected.

### Work

1. **Tail.** A `tail_ms` setting ("Hold after the sound", 0–10 s, shown only when a sound is connected) added to the target.
2. **Card.** "Fits the sound + 1.5 s"; a warning when the voiceover has lines on actions, since those don't count toward its length.
3. **Template.** A "Narrated demo" template in `crates/project/src/templates.json`: Record take → Clip from take → Fit length (voice) → Add sound (voice, music) → Show captions (voice) → Export video; write `examples/` again with `bun run examples`.
4. **Tests.** Fit to a Voiceover value and to Music, with a tail, in all three modes.

### Risks

- **Large speed changes.** A 40-second take fitted to a 15-second voice plays at 2.7×. The existing fit warning covers it; the template starts in "Hold the end" mode.

## Decided

- **One inference runtime:** `ort` for every model on this computer; no sherpa-onnx, no whisper.cpp.
- **Voices:** Kokoro's English voices first; Kitten and other languages wait for an espeak-ng phonemizer program. Piper is out. Which Kokoro precision ships is settled by ear (Voiceover step 2).
