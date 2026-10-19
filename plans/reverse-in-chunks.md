# Reverse in chunks

The Reverse file node (`media.reverse`) runs one FFmpeg command with `-vf reverse`, which keeps every decoded frame in memory until the input ends. A long file runs the computer out of memory. This plan makes it reverse the file in pieces, so memory stays flat whatever the length, with the same output.

Status: plan only. Nothing here is built yet.

## How to read this plan

- One fix, built in the order of **Work**; each step is checked before the next starts.
- Sections: **The problem (measured)**, **Fit with Opsis** (Already in place / Missing, every file and function named was checked in the repo), **Work**, **Risks**, **Sources**.

## Conventions

These restate rules from `AGENTS.md`; when the two disagree, `AGENTS.md` wins.

- No comments anywhere. File names in kebab-case.
- Node-only code stays in `src/worker`. Pure functions that plan the work go in `src/worker/media/` with unit tests next to them, like `filters.ts` / `filters.test.ts`.
- Intermediates go in the node's own folder (`ctx.dir`), never `os.tmpdir()`.
- User-facing copy names no tools or formats ("Reversed part 2 of 5", never "FFmpeg" or "segment").
- **Checks.** `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`, the no-comments check from `AGENTS.md`, and the manual run in **Work** item 6. Don't commit.

## The problem (measured)

Measured on 2026-10-18 with the dev engine's FFmpeg (`<userData>/engine/ffmpeg/ffmpeg`, the pinned ffmpeg-static build) on a 1920×1080, 30 fps H.264 test file (`testsrc2`), with the command `reverseExecutor` runs today (video only, `EDIT_VIDEO`):

| Input length | Peak memory | Time  |
| ------------ | ----------- | ----- |
| 5 s          | 1.59 GB     | 2.0 s |
| 10 s         | 2.06 GB     | 2.9 s |
| 20 s         | 2.91 GB     | 4.6 s |

About 1.2 GB of fixed cost plus about 85 MB per second of 1080p at 30 fps, one raw frame (about 3 MB in `yuv420p`) per frame. That is about 5 GB per minute; 60 fps doubles it, 4K multiplies it by four. Nothing in the app limits it; the system kills FFmpeg partway through.

**Why.** Compressed video decodes only forwards (most frames store changes from earlier ones), and an FFmpeg filter graph is one forward stream that can't go back to the input. The first output frame is the last input frame, so `reverse` can't output anything until the input ends, and holds every decoded frame until then. `areverse` does the same for sound, which is small (about 20 MB per minute of 48 kHz stereo).

**The fix, tested.** The same 20 s file cut into four 5 s pieces (`-ss <start> -t 5 -i src -vf reverse`), each reversed, then joined last piece first with the concat demuxer and `-c copy`:

- each piece peaked at 1.20–1.28 GB, independent of the file's length;
- the joined file had 600 frames, like the full reverse, and with both encoded losslessly (`-crf 0`) the PSNR between them was infinite: identical frames.

Sound was not part of that test; see **Work** item 3.

## Fit with Opsis

Already in place:

- `reverseExecutor` in `src/worker/executors/media.ts` calls `transcodeMedia` with `-vf reverse`, `-af areverse` when `input.hasAudio`, else `-an`, then `EDIT_VIDEO`, writing `reversed.mp4`; `mediaResult` probes the output and returns it as the node's `media` with an artifact.
- `ctx.ffmpeg.probe` returns `durationMs` and `video: { width, height, fps, pixFmt }` (`ProbeResult`, `src/worker/media/ffmpeg.ts`).
- `ctx.ffmpeg.run(args, { signal, durationMs, onProgress })` reports progress from `-progress pipe:1` and stops on `ctx.signal`.
- `concatList` in `src/worker/executors/media.ts` writes a concat demuxer list (used by Join files with `-f concat -safe 0 -c copy`).
- `EDIT_VIDEO` (`h264Args({ crf: 18 })`) and `EDIT_AUDIO` (`-c:a aac`) in `src/worker/media/codecs.ts`.

Missing:

- Nothing plans pieces from a memory budget.
- `reverseExecutor` has no piece loop, no per-piece progress and no clean-up of intermediates.
- No test covers reversing; executor tests don't run FFmpeg, so the planner must be pure and tested on its own.

## Work

1. **Planner.** `src/worker/media/reverse.ts`, pure, no `node:*`:
   - `frameBytes(width, height, pixFmt)`: bytes of one decoded frame. 1.5 bytes per pixel for `yuv420p`/`yuvj420p`/`nv12`, 2 for `yuv422p`, 3 for `yuv444p`/`rgb24`/`bgr24`, 4 for `rgba`/`bgra`; twice that for pixel formats with a bit depth above 8 (`p10`, `p12`, `p16` in the name); 4 bytes per pixel for anything unknown.
   - `reversePieces({ durationMs, fps, frameBytes, budgetBytes })` → `Array<{ startMs, durationMs }>`, **last piece first** (the order they are joined in). Piece length is `floor(budgetBytes / frameBytes)` frames, at least 1 s, rounded down to whole frames so boundaries fall on frame times; the last piece takes the remainder. A file that fits in one piece gives one piece.
   - `REVERSE_BUDGET_BYTES = 1 GiB`. At 1080p 30 fps `yuv420p` that is about 11 s per piece.
   - `reverse.test.ts`: one piece for a short file; pieces cover the whole length with no gap or overlap; order is last first; boundaries are whole frames; 4K and 10-bit give shorter pieces; the 1 s floor holds for a huge frame; a zero or missing fps falls back to the project fps (passed in by the caller).
2. **Pieces.** In `reverseExecutor`: probe the input; if `reversePieces` gives one piece, keep today's single command unchanged. Otherwise make `ctx.dir/reverse-parts/` and, for each piece in order, run
   `-ss <start> -t <length> -i <src> -vf reverse [-af areverse | -an] EDIT_VIDEO -c:a pcm_s16le part-<n>.mkv`
   with `ctx.signal`. `-ss` before `-i` with re-encoding is frame-exact (the decoder starts at the earlier keyframe and drops frames before the start), which the measured test confirmed. Format the times with `ffmpegSeconds`.
3. **Sound in pieces as PCM.** Pieces keep their sound uncompressed (`pcm_s16le` in Matroska) so the join has no encoder delay at each boundary: AAC adds priming samples to the start of every encode, which would put a short gap at every join if each piece were AAC. The final join encodes the sound once.
4. **Join.** Write the list with `concatList` (pieces in the planned order) and run `-f concat -safe 0 -i reverse-parts.txt -map 0 -c:v copy EDIT_AUDIO reversed.mp4` (`-an` when there's no sound). The picture is copied, so it is still encoded only once. Then delete `reverse-parts/` and the list, also when a piece fails or the run is stopped (`try`/`finally`).
5. **Progress and log.** Weight progress by piece length: piece `n` maps its own 0–1 onto its share of 90%, the join takes the last 10%. `ctx.log("info", "Reversing in 5 parts to keep memory low.")` once. The node's output, file name and label stay the same, so `ENGINE_VERSION` doesn't change.
6. **Check.**
   - Unit tests from item 1.
   - Manually, with the dev engine's FFmpeg and a scratch project: a 20 s 1080p file with sound reversed by the new path and by the old single command; same frame count (`-f framemd5`), and with the budget lowered for the test, PSNR infinite at `-crf 0`; the sound plays without clicks or gaps at the joins (listen, and compare `ebur128` totals).
   - A 3-minute 1080p file: peak memory (`/usr/bin/time -v`) stays under about 1.5 GB; with the old command it would need about 16 GB.
   - A variable-frame-rate file (a phone recording) and a 60 fps file: frame count and length match the input, no repeated or dropped frame at a boundary.
   - Stop the run during the third piece: the node stops, `reverse-parts/` is gone.

## Risks

- **Variable frame rate.** Pieces are cut by time. On a file with uneven frame times, a frame exactly on a boundary could land in both pieces or neither. The test in item 6 decides; if it fails, cut at the probed frame times instead of whole-frame multiples.
- **Many pieces on huge frames.** 8K or high bit depth gives short pieces and more FFmpeg starts (each re-reads from the keyframe before its start). Slower, never more memory.
- **Fixed cost.** The 1.2 GB measured per piece is mostly the decoder and encoder at 1080p, not the buffer; the budget only bounds the buffer. Measure at 4K before lowering the budget.
- **Sound length.** `-t` cuts sound and picture at the same time, but AAC files can have a few ms more sound than picture; check the joined length against the input with `ffprobe`.

## Sources

- [FFmpeg filters: reverse, areverse](https://ffmpeg.org/ffmpeg-filters.html#reverse)
- [FFmpeg-user: split into chunks, reverse each, concat in reverse order](https://lists.ffmpeg.org/pipermail/ffmpeg-user/2018-April/039520.html)
- [FFmpeg-devel: the patch that added the reverse filter](https://ffmpeg.org/pipermail/ffmpeg-devel/2015-July/175725.html)
- [Avoiding out-of-memory errors in long video filtergraphs](https://www.vmeg.ai/research/avoiding-oom-in-long-video-filtergraphs/)
- [FFmpeg wiki: concatenate](https://trac.ffmpeg.org/wiki/Concatenate)
