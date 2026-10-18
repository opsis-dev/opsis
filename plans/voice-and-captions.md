# Voice and captions

What Demoflow needs to speak a script and to caption speech: a Voiceover node that turns text into a voice on this computer (or, when the user turns it on, through an online service), captions from the speech in a file, a film that fits the voice's length, and a Settings page for the voices and speech models. This is milestone M4 ("Audio & captions") of the original spec, minus the parts that are already built (Captions from script, Show captions, the Mixer and its ducking).

Status: plan only. Nothing here is built yet.

## How to read this plan

- Each section is one feature. The user picks which to build and when; the first section (downloads and Settings) comes before the others because they all use it.
- Each feature section has the same parts: **What it adds**, **Fit with Demoflow**, **Work**, **Licence**, **Risks**, **Sources**.
- Sizes are download sizes of the pinned archives, read from the release listings on the date in **Sources**. Sizes on disk are measured during the work and written into the catalogue.

## Conventions

These apply to every feature in this plan. They restate rules from `AGENTS.md`; when the two disagree, `AGENTS.md` wins.

**Versions.**

- Every `remotion` / `@remotion/*` package stays at exact `4.0.535`: `bun add --exact @remotion/<name>@4.0.535`. `@remotion/captions`, `@remotion/openai-whisper` and `@remotion/elevenlabs` are already in `node_modules` (through `@remotion/studio`) but Demoflow imports them directly, so each becomes a direct dependency.
- Other packages: the latest release, through `bun add` / `bun add -d`. Never edit dependency blocks by hand.
- Downloaded programs and models are pinned like FFmpeg (`FFMPEG_RELEASE` in `src/engine/paths.ts`): a fixed release, a fixed URL per platform, and a SHA-256 checked after the download.

**Where code goes.**

- Node definitions in `src/shared/nodes/` (zod params with `.meta()`, `card()`, `compile()`); executors in `src/worker/executors/`, keyed by node type. Every node with `executes: true` has an executor (`executors.test.ts`).
- Download and path code is Node-only and goes in `src/engine/`, next to `install.ts` and `paths.ts`. The catalogue of voices and models (ids, sizes, licences, URLs, hashes) is plain data the renderer also shows, so it goes in `src/shared/speech/`.
- Speech never runs in `src/shared` or `src/video`: no `node:*`, no child processes, no calls to the speech program there.

**Behaviour.**

- Nothing uses the internet unless the user asks. Every download sits behind a button, says what it gets and how big it is, asks first, and saves under `<userData>/speech`. Online voices and transcription are off until the user turns a service on in Settings and enters a key.
- Synthesis and transcription happen when a node runs, never at preview or render time. The preview and the export play the same saved files, so preview equals export.
- Keys are encrypted with `safeStorage` through `src/main/secrets.ts`, never written to `settings.json`, never passed in `WorkerEnv`.

**User-facing copy.**

- No library, model, tool or format names in node titles, descriptions, field labels, badges, toasts or errors: "Voice on this computer", "Online voice", "English", "European languages". Sentence case, durations in seconds, technical settings under `advanced`.
- Decided naming exception, added to `AGENTS.md` when this is built: Settings → Voice and captions may name the online services (ElevenLabs, OpenAI, OpenRouter) in its service choices and key fields. Local voices and models show plain labels ("Natural voices", "Small voices", "English", "European languages"); each label has a small circled `?` whose hover card names the engine and model and gives size, languages and licence, for users who want the detail. Names appear nowhere else (nodes, cards, toasts, errors), and in Settings → Licences.
- `src/shared/nodes/card.test.ts` checks every node's description and the banned words; add the new names (Kokoro, Kitten, Piper, Whisper, Parakeet, NeMo, ElevenLabs, OpenAI, OpenRouter, ONNX) to its banned list.

**Licences.**

- MIT and Apache-2.0 packages and models get an entry in `THIRD_PARTY_NOTICES.md` (shown in Settings → Licences). Downloaded programs and voices get a licence entry in `sources()` (`src/main/licences.ts`) that appears once they're on disk, like FFmpeg's.
- Never offer a voice whose licence hasn't been checked for commercial use, the same rule as built-in sounds.

**Checks when a feature is done.**

- `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`.
- The no-comments check from `AGENTS.md`.
- Try it in the app: download from Settings, run the node, play it in the Preview, change the Mixer.
- Export a short film that uses it and compare with the Preview: preview must equal export.
- Don't commit or push.

## Summary

| Feature                            | What users get                                                                                                                                                                     | Licence                                                       | Size                                                          |
| ---------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------- | ------------------------------------------------------------- |
| Speech downloads and Settings page | A Voice and captions page: voices and speech models with their sizes, download and remove buttons, online services with an encrypted key                                           | Apache-2.0 (speech program), GPL-3.0 (its pronunciation part) | Speech program 20–28 MB per platform, for voices and captions |
| Voiceover node                     | A voice reading a script with variables, line by line, at a set speed; lines can start on browser actions; its own Voice bus that lowers the music; captions that follow the voice | Apache-2.0 voices (Kokoro, Kitten)                            | 31 MB (small voice), 132 MB (natural voices)                  |
| Online voices and transcription    | The same nodes using ElevenLabs, OpenAI, OpenRouter or any compatible address, after the user turns it on and enters a key                                                         | Service terms; converter packages MIT                         | Nothing to download                                           |
| Captions from speech               | Word-timed captions from the speech in a video or sound file, for Show captions, plus a subtitle file                                                                              | CC-BY-4.0 (transcription models)                              | 460 MB (English), 464 MB (25 European languages)              |
| Fit length to the voiceover        | Fit length makes the film as long as the voice, plus a tail, by speeding up, holding or cutting                                                                                    | Demoflow's own                                                | None                                                          |

## Speech downloads and the Voice and captions settings page

### What it adds

A Settings page, **Voice and captions**, that lists everything speech needs and gets it only when asked:

- **Voices on this computer**: each voice pack under a plain label ("Natural voices", "Small voices") with its languages, number of voices, download size and size on disk; Download, Remove, and Play a sample once downloaded. A circled `?` next to the label opens a hover card with the engine, model name, licence and languages.
- **Captions on this computer**: two transcription models, "English" and "European languages" (25 languages), with their sizes, each with the same `?` card (NVIDIA Parakeet TDT 0.6B v2 and v3, CC-BY-4.0).
- **Online services**: off by default. ElevenLabs, OpenAI and OpenRouter each get a switch, an encrypted key field and a Check key button; **Compatible service** takes an address, a voice model, a transcription model and an optional key, for any service or local speech server that speaks OpenAI's audio API.
- The folder it all lives in, and the total space used.

The first voice download also gets the speech program (shown in the same size line: "132 MB voice + 21 MB program"); the first model download also gets the same program if no voice was downloaded yet: one speech program speaks and transcribes.

### Fit with Demoflow

Already in place:

- The engine download pattern: `installFfmpeg` and `download()` in `src/engine/install.ts` stream with progress, write `.part` then rename; `megabytes()` formats sizes; `calmProgress()` throttles progress; the install runs in a worker (`case "install-engine"` in `src/worker/worker.ts`) started by `installEngine` in `src/main/workers.ts`, which relays `EngineInstallEvent`s to the renderer.
- Ask-first downloads in the renderer: the font library's online fonts confirm with an `AlertDialog` before the first request (`onlineAllowed` in `src/renderer/fonts/fonts-panel.tsx`).
- Settings pages: `PAGES` and `Page()` in `src/renderer/app/settings/settings-window.tsx`, `SettingsPage` in `src/shared/ipc.ts`, app settings in `appSettingsSchema` (`src/shared/settings.ts`) saved by `patchSettings` (`src/main/settings.ts`).
- Encrypted secrets: `writeSecret` / `readSecret` (`src/main/secrets.ts`) take a folder, so `app.getPath("userData")` works as one; the worker asks main for secrets with `secret:read` and `secret:write` (`src/main/workers.ts`, `src/shared/worker-protocol.ts`).
- Long main-process work is listed on exit through `track(label, promise)` (`src/main/exit.ts`).
- Licence entries that appear once a file exists: `sources()` / `listLicences()` in `src/main/licences.ts`.

Missing:

- A speech folder, catalogue and installer. `download()` isn't exported, checks no hash, and only un-gzips; the speech program comes as `.tar.bz2`. `fflate` (already a dependency) does zip and gzip; tar and bzip2 need `tar-stream` and `seek-bzip` (both MIT).
- App-wide secrets: `secret:read` reads from the project's folder only.
- A `"speech"` `SettingsPage` and its page component.

### Work

1. **Dependencies.** `bun add tar-stream seek-bzip`, `bun add -d @types/tar-stream`.
2. **Catalogue.** `src/shared/speech/catalog.ts`: `SPEECH_TOOLS` (the speech program per platform: URL, SHA-256, download bytes, bytes on disk, licence), `VOICE_PACKS` (id, label, languages, voices with display names, URL, hash, sizes, licence id) and `TRANSCRIBE_MODELS` (`english` = `sherpa-onnx-nemo-parakeet-tdt-0.6b-v2-int8`, 460 MB; `european` = `sherpa-onnx-nemo-parakeet-tdt-0.6b-v3-int8`, 464 MB, from sherpa-onnx's `asr-models` release). Pinned release: `sherpa-onnx v1.13.8`, the `shared` build per platform, which has both `sherpa-onnx-offline-tts` and `sherpa-onnx-offline`. No whisper.cpp on any platform (decided: it has no maintained macOS build, and one program for both jobs is simpler). A test checks every entry has a hash, a size, a checked licence and no banned word in its labels.
3. **Paths.** `src/engine/speech.ts`: `speechPaths(userData)` → `<userData>/speech/{program,voices/<id>,models/<id>}`, and `speechStatus()` that reports what's installed, like `engineStatus()`.
4. **Installer.** `src/engine/speech-install.ts`: export `download()` from `install.ts` and add an optional SHA-256 check; `installSpeechTool(kind)`, `installVoicePack(id)`, `installTranscribeModel(id)`, each extracting into a staging folder and renaming into place, `chmod 0o755` on programs; `removeSpeechItem(id)`. Models download with the same hashed `download()` as voice packs.
5. **Worker and main.** A `install-speech` worker message beside `install-engine`, a `installSpeech(item)` IPC in `src/main/main.ts` wrapped in `track("Downloading speech files", …)`, progress events through the same shape as `EngineInstallEvent`.
6. **App-wide secrets.** Service keys live in the app state database's `secrets` table (`scope: "app"`), sealed with `encryptedText` (`plans/app-state.md`, **Encrypted values**); project sign-ins stay in the project's `secrets/` folder. New IPC `setServiceKey(service, key)` / `hasServiceKey(service)`; the renderer never reads a key back.
7. **Settings.** `appSettingsSchema` gains `speech: { voice: string, transcribe: "english" | "european", online: { elevenlabs: boolean, openai: boolean, openrouter: boolean, compatible: { enabled: boolean, address: string, voiceModel: string, transcribeModel: string } } }`, all off or empty by default. Add `"speech"` to `SettingsPage`, a `SpeechPage` in `src/renderer/app/settings/speech-page.tsx` and a `PAGES` row ("Voice and captions", icon `AudioLines`). Download buttons open an `AlertDialog` naming the size before anything is fetched. The `?` cards are one small `InfoHint` component (the installed shadcn `HoverCard` with a `CircleHelp` icon trigger) fed from the catalogue entry, so the details live in one place.
8. **Licences.** Entries in `sources()` for the speech program (with the pronunciation data's GPL-3.0 text), each voice pack and each transcription model (CC-BY-4.0, crediting NVIDIA), read from the downloaded folders.
9. **Tests.** Catalogue test; `speechPaths` per platform (like `paths.test.ts`); hash mismatch rejects and deletes the `.part`.

### Licence

`tar-stream`, `seek-bzip`: MIT. The downloaded programs keep their own licences (see the features below) and are shown in Settings → Licences once downloaded.

### Risks

- **Linux builds and old systems.** The `linux-x64-shared` build links the C++ runtime of the machine it was built on; an AppImage user on an older distribution may not have it. Check on the oldest supported distribution; fall back to building it in CI if it fails.
- **macOS.** sherpa-onnx ships macOS builds (`osx-arm64-shared` 20.3 MB, `osx-x64-shared` 22.8 MB, and universal2). Check that a program the app downloads runs without Gatekeeper prompts on a notarized and an unsigned app (`plans/shipping.md`).
- **Unencrypted keys.** `writeSecret` falls back to plain text when `safeStorage.isEncryptionAvailable()` is false (Linux without a keyring). For service keys, say so in Settings before saving.

### Sources

- Demoflow source: `src/engine/install.ts`, `src/main/secrets.ts`, `src/main/licences.ts`, `src/renderer/app/settings/settings-window.tsx` (checked 2026-10-16)
- [sherpa-onnx v1.13.8 release](https://github.com/k2-fsa/sherpa-onnx/releases/tag/v1.13.8): `linux-x64-shared` 28.2 MB, `linux-aarch64-shared-cpu` 28.1 MB, `osx-arm64-shared` 20.3 MB, `osx-x64-shared` 22.8 MB, `win-x64-shared-MD-Release` 20.5 MB (checked 2026-10-16)
- npm metadata: `tar-stream@3.2.2`, `seek-bzip@2.0.0` (checked 2026-10-16)

## Voiceover node

### What it adds

A **Voiceover** node (Audio): a script, a voice and a speed in, a voice track and its captions out.

- **Script**: one line per spoken line, with variables (`VarText`). Each line is spoken separately, so its start and end are exact.
- **Voice**: a picker grouped by pack ("Natural voices", "Small voices"), with a play button, and online voices when a service is on.
- **Speed**, **Starts at**, **Pause between lines**, and a per-line nudge that the Timeline writes when a line is dragged.
- **Lines on actions**: any line can start on a browser action of the clip it's added to, plus an offset, like Sound effect's **On an action**.
- Outputs: **Sound** (to Add sound or Fit length) and **Captions** (to Show captions), so captions follow the voice word by word.

### Fit with Demoflow

Already in place:

- Sounds join a clip through Add sound (`addAudioNode`, `src/shared/nodes/audio.ts`), which takes any number of `audio` values (`many()`) and places anchored ones at their action's mark with `anchorSounds()` (via `flattenSpec`). Sound effect stores its action in `atAction` (`widget: "action"`) and an `offsetMs`.
- `AudioItem` (`src/shared/schema/timeline.ts`) carries `channel`, `role`, `lengthFrames`, `anchor` and `handles`; each `channel` is one Mixer strip.
- Ducking is a bus setting: `duckKeysFor` (`src/shared/mix/levels.ts`) keys a bus on the buses named in `bus.duck.keys`; `buildMixPlan` (`src/shared/mix/plan.ts`) renders it with `sidechaincompress`. A **Voice** strip preset already exists in `STRIP_PRESETS` (`src/shared/mix/types.ts`).
- `CaptionsValue` (`src/shared/schema/values.ts`) is `{ text, startMs, endMs }` per word; `captionsNode` (Show captions, `src/shared/nodes/text.ts`) draws it with `core/captions`, whose `Captions` component (`src/video/core/captions.tsx`) groups words into lines of up to six or at sentence ends.
- Executors that run FFmpeg and return files: `mediaResult()` and `transcode()` in `src/worker/executors/media.ts`; `ExecutorContext` has `ffmpeg`, `dir`, `progress()`, `signal`, and `projectFileRef(ctx, file)` (`src/worker/executors/util.ts`) makes a file's `project:` ref. A compile can read its executor's outputs from `result` (as `src/shared/nodes/media.ts` does).

Missing:

- No text-to-speech anywhere. Recordings carry no sound, so all speech comes from this node or from files.
- `role` is `"music" | "effect"`; `ChannelKind` is `"music" | "effect" | "clip"`; `kindOfAudio` maps everything that isn't music to `effect`, so a voice would land on the Effects bus.
- `DEFAULT_BUSES` has Music, Effects and Clips; there's no Voice bus to duck under.
- Captions can't follow an action: `CaptionsValue` has no anchor, and Show captions doesn't resolve marks.
- Icons: `icons.ts` has `AudioLines` and `Captions` but no microphone or speech icon.

### Work

1. **Speech program.** The voices run in sherpa-onnx's `sherpa-onnx-offline-tts` program (downloaded in the section above) as a separate process: `--kokoro-model`, `--kokoro-voices`, `--kokoro-tokens`, `--kokoro-data-dir` (or the Kitten equivalents), `--sid`, `--speed`, `--num-threads`, `--output-filename`. Keeping it out of Demoflow's process avoids native addons in Electron and keeps the GPL-3.0 pronunciation part in a separate program, as with FFmpeg.
2. **Voice packs** in `VOICE_PACKS`: "Natural voices" = `kokoro-int8-multi-lang-v1_0` (132 MB, 53 voices, English and Chinese); "Small voices" = `kitten-nano-en-v0_8-int8` (31 MB, English). Measure the full-precision Kokoro pack (350 MB) against int8 by ear before deciding whether to offer it. Piper voices are out (decided): licences differ per voice (`ryan` is CC BY-NC-SA 4.0, `lessac` uses the Blizzard licence).
3. **Node.** `audio.voiceover` in `src/shared/nodes/audio.ts`: `phase: "media"`, `executes: true`, `inputs: {}`, outputs `audio` and `captions`. Params: `script` (`text(…, "textarea")`), `voice` (new `voice` widget: catalogue voices, installed or not, with a Download link to Settings), `speed` (0.5–2, step 0.05), `startMs`, `pauseMs` (`ms(400, "Pause between lines", 5000)`), `nudgesMs` (`offsets` widget, one per line), `lineActions` (`z.array(z.object({ line, atAction, offsetMs }))` with a new `lineActions` widget that lists the script's lines with the existing action picker, matched by line text so reordering keeps them), and `wordTiming` under `advanced` ("From the voice" when a transcription model is installed, else "Estimated"). Card: first line quoted, voice name, total length, "N lines on actions"; warn when the voice isn't downloaded.
4. **Executor.** `voiceoverExecutor` in a new `src/worker/executors/speech.ts`, added to `EXECUTORS` (`src/worker/executors/index.ts`): for each line, synthesise to WAV in a shared cache `.demoflow/cache/voice/<sha256 of pack, voice, speed, text>.wav`, so editing one line re-speaks only that line; probe each length with `ctx.ffmpeg.probe`; word times from `wordTiming` (see step 6). Outputs `{ lines: [{ src, durationMs, words }] }` as JSON in the node's folder, plus an `audio` artifact of the whole voice for the card.
5. **Compile.** From `result` and the params (so moving or pausing never re-speaks): one `AudioItem` per line with `channel: nodeId`, `role: "voice"`, `lengthFrames`, `anchor` from `lineActions`, `from` = start + earlier lines + pauses + nudges, `handles.move` = `{ param: "nudgesMs", index: i }` (or the line's `offsetMs` when anchored). `AudioValue.durationInFrames` = end of the last unanchored line. `CaptionsValue` with each word's times, shifted the same way.
6. **Word timing.** "Estimated": spread the line's length over its words by character count, with a short lead-in; a pure function in `src/shared/speech/words.ts`, unit-tested. "From the voice": run the transcriber on each line (Captions from speech's code) and keep its word times when the words match the script, else fall back to the estimate.
7. **Voice role and bus.** `role` gains `"voice"` in `audioItemSchema`; `ChannelKind` and `DEFAULT_BUS_FOR` gain `voice → "voice"`; `kindOfAudio` returns `"voice"`; `soundLength` (`src/shared/compile/duck.ts`) treats voice like effects (natural length); the `flatten.ts` role union gains it. `DEFAULT_BUSES` gains a Voice bus starting from the Voice preset's EQ and compressor; `normalizeWorkflow` (`src/shared/nodes/registry.ts`) adds it to saved mixes that lack it. When the first Voiceover is added, the workflow store turns on the Music bus's ducking keyed on `"voice"` if it's off, in the same undo step.
8. **Captions on actions.** `CaptionsValue` words gain an optional `anchor`; `captionsNode` resolves them against its **Over clip** input with the same marks as `anchorSounds()` (extract a shared `markFrame()` helper). Without a clip, anchored lines use their unanchored times and the card says so.
9. **Icons.** Add `MicVocal` (`mic-vocal`) and `Speech` (`speech`) to `src/renderer/editor/icons.ts`.
10. **Tests.** Executor exists (`executors.test.ts`); `card.test.ts`; compile placement with pauses, nudges and anchors; a round-trip in `edits.test.ts` for the line move handle; ducking keys include the voice; `normalizeWorkflow` adds the Voice bus once.
11. **Check.** A film with music, a voiceover of five lines (two on actions) and Show captions: the music dips under each line, captions follow the words, dragging a line on the Timeline moves its captions, re-running after editing one line re-speaks only that line, and the export sounds and looks like the Preview.

### Licence

- Speech program: sherpa-onnx, Apache-2.0. Its text-to-sound step uses espeak-ng (GPL-3.0) and ships `espeak-ng-data` in each Kokoro pack; it runs as a separate program, like FFmpeg, and its licence text and source link go in Settings → Licences.
- Voices: Kokoro-82M weights Apache-2.0 (trained on public-domain, Apache/MIT and two CC BY datasets, credited in the notice); Kitten TTS 0.8 weights Apache-2.0 per their model cards.

### Risks

- **Speed on slow computers.** sherpa-onnx's only published Kokoro numbers are for a Raspberry Pi 4 (real-time factor 3.2 with four threads, slower than real time). Measure Kokoro int8 and Kitten on the CI machines and a 4-core laptop; if Kokoro is slower than real time on the laptop, make Small voices the default.
- **Numbers, names and URLs.** Neither pack normalises text much ("v2.1", "$29/mo", product names). Add a small, tested text tidy-up (`src/shared/speech/tidy.ts`) and show the tidied line as a hint under the script.
- **Long lines.** Very long lines lose pace; split lines over about 400 characters at sentence ends before speaking.

### Sources

- [sherpa-onnx: Kokoro models](https://k2-fsa.github.io/sherpa/onnx/tts/pretrained_models/kokoro.html) (command line, 53 speakers in v1.0, 24 kHz, Raspberry Pi RTF table) (checked 2026-10-16)
- sherpa-onnx `sherpa-onnx/csrc/sherpa-onnx-offline-tts.cc` on GitHub (`--speed`, `--sid`, `--output-filename`; no word timestamps in `offline-tts.h`) (checked 2026-10-16)
- [sherpa-onnx `tts-models` release](https://github.com/k2-fsa/sherpa-onnx/releases/tag/tts-models): `kokoro-int8-multi-lang-v1_0` 132.3 MB, `kokoro-multi-lang-v1_0` 349.9 MB, `kitten-nano-en-v0_8-int8` 31.2 MB, `kitten-mini-en-v0_8` 67.5 MB, `vits-piper-en_US-libritts_r-medium` 82.0 MB (checked 2026-10-16)
- [hexgrad/Kokoro-82M](https://huggingface.co/hexgrad/Kokoro-82M) (Apache-2.0 weights, 82M parameters, training data) and [KittenML/KittenTTS](https://github.com/KittenML/KittenTTS); Hugging Face API for `KittenML/kitten-tts-mini-0.8` (apache-2.0) (checked 2026-10-16)
- [rhasspy/piper-voices](https://huggingface.co/rhasspy/piper-voices) model cards for `lessac`, `amy`, `ryan`, `libritts_r` (checked 2026-10-16); GitHub licence API: `espeak-ng/espeak-ng` GPL-3.0, `k2-fsa/sherpa-onnx` Apache-2.0, `OHF-Voice/piper1-gpl` GPL-3.0, `rhasspy/piper` MIT and archived (checked 2026-10-16)
- npm metadata: `kokoro-js@1.2.1` (Apache-2.0, depends on `phonemizer`, an espeak-ng build, and `@huggingface/transformers`), `onnxruntime-node@1.30.0` (301 MB unpacked): the in-process route this plan doesn't take (checked 2026-10-16)

## Online voices and transcription (opt-in)

### What it adds

The Voiceover node and Captions from speech can use an online service instead of this computer, once the user turns the service on in Settings and enters a key. Four choices: ElevenLabs, OpenAI, OpenRouter (a preset with its address filled in and its transcription format handled), and **Compatible service**, any address that speaks OpenAI's audio API, including speech servers the user runs on their own computer. Voices from the service appear in the voice picker under "Online voices". Results are saved in the node's cache, so a run with nothing changed sends nothing.

### Fit with Demoflow

Already in place:

- `@remotion/elevenlabs` (`elevenLabsTranscriptToCaptions`) and `@remotion/openai-whisper` (`openAiWhisperApiToCaptions`) turn the services' transcripts into `Caption[]` (`{ text, startMs, endMs, timestampMs, confidence }`); both are in `node_modules` at 4.0.535, MIT, and make no requests themselves.
- Encrypted keys and the ask-first pattern from the first section.

Missing:

- Request code for the services, a voice list cache, and a first-use confirmation.

### Work

1. **Dependencies.** `bun add --exact @remotion/captions@4.0.535 @remotion/elevenlabs@4.0.535 @remotion/openai-whisper@4.0.535`.
2. **Requests** in `src/worker/speech/online.ts`, with `fetch` and the key read through `secret:read` (app scope):
   - ElevenLabs voice: `POST /v1/text-to-speech/{voice_id}/with-timestamps` (`xi-api-key` header, `voice_settings.speed`); decode `audio_base64`, turn `alignment` (per-character start and end times) into word times with a pure, tested function.
   - OpenAI voice: `POST /v1/audio/speech` with `gpt-4o-mini-tts`, a built-in voice and `response_format: "wav"`. It returns no timestamps and documents no speed setting, so word timing uses the Voiceover's estimate or the transcriber, and Speed is applied with FFmpeg's `atempo`.
   - ElevenLabs transcription: `POST /v1/speech-to-text` with `model_id: "scribe_v2"` and `timestamps_granularity: "word"`, then `elevenLabsTranscriptToCaptions`.
   - OpenAI transcription: `POST /v1/audio/transcriptions` with `whisper-1`, `response_format: "verbose_json"` and `timestamp_granularities[]=word` (only `whisper-1` has word timestamps), then `openAiWhisperApiToCaptions`.
   - OpenRouter voice: `POST https://openrouter.ai/api/v1/audio/speech`, OpenAI-compatible (`model`, `input`, `voice`), raw audio back (MP3 or PCM). Models come from its model list filtered by `output_modalities`; no timestamps, so timing works as for OpenAI.
   - OpenRouter transcription: `POST https://openrouter.ai/api/v1/audio/transcriptions` with JSON `{ model, input_audio: { data: <base64>, format } }`. Its docs disagree on whether OpenAI-style multipart uploads work, so use the JSON form. Check whether the response has word timestamps; if not, captions are timed by the local transcriber when installed, else estimated, and the card says so.
   - Compatible service: the OpenAI requests above against the user's address (`{address}/audio/speech`, `{address}/audio/transcriptions`, multipart), with the user's model names and the key when one is set. Servers differ in what they return: use word timestamps when the response has them, else fall back as for OpenRouter. Check key sends one tiny request and reports what the server supports.
3. **Voice list.** `GET /v1/voices` for ElevenLabs and OpenRouter's model list only when the user opens the picker with the service on; cached in `<userData>/speech/online-voices.json`. OpenAI's list is fixed in the catalogue; Compatible service uses the voice names the user types.
4. **Asking first.** The first run that would send text or sound to a service shows a confirmation in the Run panel naming what is sent; "Don't ask again" is per service in `settings.speech.online`.
5. **Errors.** Plain messages without service names in nodes ("The online voice refused the key. Check it in Settings."); the HTTP status goes to the Run log.
6. **Tests.** Alignment → words; recorded fixture responses through both converters; no request is made when the service is off.

### Licence

`@remotion/captions`, `@remotion/elevenlabs`, `@remotion/openai-whisper`: MIT, entries in `THIRD_PARTY_NOTICES.md`. Use of the services is under each service's terms with the user's own key; Demoflow ships nothing from them.

### Risks

- **Compatible servers vary.** Some ignore `speed`, some return no word timings, some need no key. Check key records what the server supports, and the node falls back instead of failing.
- **Size limits.** Check the services' current upload limits before sending; for long films, send FFmpeg-made mono speech-band audio instead of the original file.
- **Cost surprises.** Show the character count of the script next to an online voice; re-runs reuse the line cache.

### Sources

- [ElevenLabs: Create speech with timing](https://elevenlabs.io/docs/api-reference/text-to-speech/convert-with-timestamps) and [Speech to text](https://elevenlabs.io/docs/api-reference/speech-to-text/convert) (checked 2026-10-16)
- [OpenRouter: New audio APIs](https://openrouter.ai/blog/announcements/announcing-audio-apis/), [Text-to-speech](https://openrouter.ai/docs/guides/overview/multimodal/tts), [Create speech](https://openrouter.ai/docs/api/api-reference/tts/create-speech), [Speech-to-text](https://openrouter.ai/docs/guides/overview/multimodal/stt) and the [OpenRouter STT skill](https://github.com/OpenRouterTeam/skills/blob/main/skills/openrouter-stt/SKILL.md), which says the endpoint isn't OpenAI-compatible (checked 2026-10-17)
- [OpenAI: Text to speech guide](https://developers.openai.com/docs/guides/text-to-speech) and [Speech to text guide](https://developers.openai.com/docs/guides/speech-to-text) (`timestamp_granularities` only for `whisper-1`) (checked 2026-10-16)
- Package source of `@remotion/elevenlabs@4.0.535`, `@remotion/openai-whisper@4.0.535`, `@remotion/captions@4.0.535` in `node_modules` (exports, MIT) (checked 2026-10-16)

## Captions from speech

### What it adds

A **Captions from speech** node (Text & overlays): a video or sound file in, word-timed captions out, transcribed on this computer (or online, when turned on). Settings: **Language** ("English" or "European languages", from the downloaded models), **Fix words** (a find-and-replace list applied after, for names and product terms), and **Starts at**. It also saves a subtitle file next to its result.

### Fit with Demoflow

Already in place:

- The captions pipeline: `CaptionsValue`, the `captions` port, Show captions, and Captions from script (`captionsFromTextNode`) as the model for the node's shape.
- Files reach nodes as `MediaValue` from Import file, Extract audio (`media.extractAudio`) and the other Media nodes; `requireMedia()` and `refToPath()` (`src/worker/executors/util.ts`) give an executor the file's path; `transcode()` runs FFmpeg with progress.

Missing:

- The speech program and models (first section), the node, its executor, and turning tokens into words.

### Work

1. **Dependency.** `@remotion/captions` from the online section (for the `Caption` type and `serializeSrt`), imported only from `src/worker` and `src/shared/speech`.
2. **Node.** `text.captionsFromSpeech` in `src/shared/nodes/text.ts`: `phase: "media"`, `executes: true`, input `media: port("media", "Speech")`, output `captions`. Params above; `language` defaults to `settings.speech.transcribe`. Card: first words of the result, language, "N words"; warn when the model isn't downloaded.
3. **Executor** in `src/worker/executors/speech.ts`:
   1. FFmpeg to 16 kHz mono 16-bit WAV in the node folder (`-ar 16000 -ac 1 -c:a pcm_s16le`).
   2. Long files: split at silences into pieces under 10 minutes (FFmpeg `silencedetect`), transcribe each, and offset their times.
   3. `sherpa-onnx-offline --encoder=… --decoder=… --joiner=… --tokens=… --model-type=nemo_transducer <wav>` with `--num-threads`; read the one-line JSON it prints (`text`, `tokens`, `timestamps` in seconds).
   4. `tokensToCaptions()` in `src/shared/speech/words.ts` (pure, tested): a token starting with a space starts a word; a word starts at its first token's time and ends at the next word's start, capped at 0.8 s after its last token; punctuation tokens join the word before them, so `/[.!?]$/` still ends lines in `Captions`. Then **Fix words** and **Starts at**.
   5. Write `captions.json` and, with `serializeSrt`, `captions.srt` as artifacts.
4. **Captions from speech and the Voiceover.** Expose the transcribe step as a function the Voiceover's "From the voice" word timing calls (Voiceover step 6).
5. **Tests.** Executor exists; a recorded output line from each model through `tokensToCaptions` (spaces, punctuation, fixes, offset, the 0.8 s cap); the silence split keeps times continuous; `card.test.ts`.
6. **Check.** Import a screen recording with narration, run Captions from speech with each model, show the captions with Show captions over the clip: words land within a frame or two of the speech in the Preview and the export, on Linux, Windows and macOS.

### Licence

NVIDIA Parakeet TDT 0.6B v2 and v3: CC-BY-4.0 (the Hugging Face model data for `nvidia/parakeet-tdt-0.6b-v2` and `-v3`; NVIDIA's hosted versions have separate terms that don't apply to the downloaded weights). Credit NVIDIA in Settings → Licences. sherpa-onnx: Apache-2.0.

### Risks

- **Languages.** The models cover English and 24 more European languages, not the roughly 99 Whisper covers. Other languages need an online service.
- **Word ends.** The program prints start times only (no durations in its output), so word ends are estimated; check by eye that captions don't linger.
- **Speed.** Measure a 1-minute file on a laptop without a fast CPU; show the expected time from that measurement on first use.

### Sources

- [sherpa-onnx: NeMo transducer models](https://k2-fsa.github.io/sherpa/onnx/pretrained_models/offline-transducer/nemo-transducer-models.html) (command line, output JSON with `tokens` and `timestamps`, model files, 25 languages for v3; checked 2026-10-17)
- [sherpa-onnx `asr-models` release](https://github.com/k2-fsa/sherpa-onnx/releases/tag/asr-models): `sherpa-onnx-nemo-parakeet-tdt-0.6b-v2-int8.tar.bz2` 460 MB, `…-v3-int8.tar.bz2` 464 MB (checked 2026-10-17)
- Hugging Face model API for `nvidia/parakeet-tdt-0.6b-v2` and `nvidia/parakeet-tdt-0.6b-v3`: `license: cc-by-4.0` (checked 2026-10-17); [NVIDIA model card](https://build.nvidia.com/nvidia/parakeet-tdt-0_6b-v2/modelcard) (English, word timestamps)
- [whisper.cpp PR #4029](https://github.com/ggml-org/whisper.cpp/pull/4029) (macOS CLI builds, closed 2026-10-02): why whisper.cpp was dropped

## Fit length to the voiceover

### What it adds

Fit length makes a clip as long as the voiceover, by speeding up, holding the end or cutting, with an extra **Hold after the sound** so the film doesn't end on the last word.

### Fit with Demoflow

Already in place:

- `durationFitNode` (`timeline.durationFit`, "Fit length", `src/shared/nodes/timeline.ts`) has an `audio` input "Fit to this sound" and uses `AudioValue.durationInFrames` as its target. A Voiceover that sets `durationInFrames` (Voiceover step 5) fits with no change to this node.
- Add sound cuts sounds where the clip ends, so fitting first and adding the voice after keeps every line; action marks are read after the speed change (`anchorSounds` runs in Add sound), so lines on actions still land on them.

Missing:

- A connected file is ignored: the `audio` port accepts `media` (`ACCEPTS` in `src/shared/nodes/ports.ts`), but `compile` only reads `durationInFrames`, not `MediaValue.durationMs`.
- No tail; the card says "Fits the sound" whatever is connected.

### Work

1. **Target.** `durationInFrames ?? msToFrames(media.durationMs)` from the connected value, plus a new `tailMs` param (`ms(0, "Hold after the sound", 10_000)`, shown only when a sound is connected).
2. **Card.** "Fits the sound + 1.5 s"; warn when the voiceover has lines on actions, since those don't count toward its length.
3. **Template.** A "Narrated demo" template in `src/shared/templates/index.ts`: Record → Auto zoom → Fit length (voice) → Add sound (voice, music) → Show captions (voice) → Export; regenerate `examples/` with `bun run examples`.
4. **Tests.** Fit to a Voiceover value, to a file's `durationMs`, with a tail, in all three modes.

### Licence

Demoflow's own code.

### Risks

- **Large speed changes.** A 40-second take fitted to a 15-second voice plays at 2.7×. The existing `fitRow` warning covers it; the template starts in "Hold the end" mode.

### Sources

- Demoflow source: `durationFitNode`, `fitRow` (`src/shared/nodes/timeline.ts`), `addAudioNode`, `anchorSounds` (`src/shared/nodes/audio.ts`), `canConnect` (`src/shared/nodes/ports.ts`) (checked 2026-10-16)

## Decided

- **Which voices ship** is settled by listening, not by a choice now: Work step 2 of the Voiceover section compares the int8 and full-precision Kokoro packs by ear before the catalogue is final.
