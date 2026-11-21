# Renderer pipe

How the app and the renderer program `opsis-renderer` talk: messages that carry their own data as one rkyv archive instead of a JSON header and loose byte blobs, then those archives in memory both processes share, so a frame's pixels exist once instead of once per process. The renderer itself and its passes don't change; [film.md](../agent-docs/film.md) has how a film reaches it.

Status: plan only. Nothing here is built yet.

## How to read this plan

- Section 1 makes every message one rkyv archive over the existing pipe. Section 2 moves those archives into shared memory, leaving the pipe for the small signals. Section 2 builds on section 1 and reuses its messages unchanged.
- Each section has the same parts where they apply: **In place**, **Work** (numbered), **Decided**, **Decision for the user**, **Risks**. Rules from `AGENTS.md` and `agent-docs/` hold throughout; when they disagree with this plan, they win.
- Each piece of work ends with the usual checks: `bun run typecheck`, `bun run lint:fix`, `bun run format:fix`, `bun run test`, and the renderer's own tests with the engine (`cargo test -p opsis-effects -p opsis-render -p opsis-film -- --ignored`), since effects and 3D stills draw through the renderer program.

## Summary

| Topic            | What users get                                                                | Left to do                                            | Status      |
| ---------------- | ----------------------------------------------------------------------------- | ----------------------------------------------------- | ----------- |
| 1. rkyv messages | Nothing visible; a smaller protocol whose messages say what they carry        | the message types, `send` and `receive`, both ends    | not started |
| 2. Shared memory | Exports with effects and 3D that hold half the frame memory per export thread | an `opsis-shm` crate, the buffer hand-over, both ends | not started |

## 1. rkyv messages

**In place.**

- `crates/renderer/src/protocol.rs` (86 lines): each message is one line of JSON, `{"message": …, "blobs": [lengths]}`, followed by the blobs' raw bytes back to back. The JSON header exists only so large data never goes through JSON.
- `Request::Run { program, size, settings }` sends the input layers as blobs; `Request::Stage { stage, size, pictures, models, environment }` sends picture sizes in the header and the pictures' RGBA, the model files not sent before, and the room's reflection map and irradiance as blobs, in that order (`Renderer::stage`, `crates/renderer/src/client.rs`).
- The child unpicks them in the same order (`stage`, `crates/effects/src/child.rs`), and keeps models and the room's maps between requests (`Kept`); the client remembers what it sent (`models`, `environment`).
- `Reply::Drawn` carries the finished pixels as one blob; `Reply::Failed` carries the error text.
- Nobody reads this pipe while debugging; it only has to be correct and small.

**Work.**

1. **Messages that hold their data.** `Request::Run { program, size, settings, layers: Vec<Vec<u8>> }`; `Request::Stage { stage, size, pictures: Vec<Option<Picture { width, height, rgba }>>, models: Vec<Model { name, bytes }>, environment: Option<Room { reflections, irradiance }> }`; `Reply::Drawn { pixels }` and `Reply::Failed { message }`. Every one derives rkyv's `Archive`, `Serialize` and `Deserialize`.
2. **One frame per message.** `send` writes the archive's length and then the archive; `receive` reads into rkyv's aligned buffer and hands back the checked archived message (`rkyv::access` with `bytecheck`), so the child reads the layers, pictures and models in place with no copy into a `Vec` per blob. `Header`, the blob lengths and the order conventions in `Renderer::stage` and the child's `stage` go away.
3. **The stage.** `Stage3d` and the types under it in `opsis-scene` gain the rkyv derives beside their serde ones (or carry over as an embedded field, Decision for the user).
4. **Both ends at once.** The client and the child change in the same commit; they ship together, so there is no old message format to accept.
5. **Tests.** A round trip of every request and reply through `send` and `receive` as a property over generated messages (sizes, settings, any number of layers, pictures and models), and a message the child can't check failing the request instead of the program.

**Decided.**

- rkyv 0.8 (MIT), checked archives, no `unsafe`.
- The whole message is the archive; there is no separate header.
- The pipe and its one-request-one-reply order stay.

**Decision for the user.**

- **How `Stage3d` crosses.** rkyv derives on the scene types (clean, the bulk of the diff, rkyv attributes in a crate that otherwise has nothing to do with the pipe), or the stage serialized as its own field inside the archive (no change to `opsis-scene`, one small decode per stage).

**Risks.**

- rkyv's archived types are their own types (`ArchivedVec`, `ArchivedString`), so code that reads a message works with those, not with the Rust types it was built from.
- A rkyv major version changes the archive format; it moves on purpose, both ends in one commit, like the other pinned protocol crates.

## 2. Shared memory

**In place.**

- Every input layer and every result exists twice while a pass runs: the app keeps its layer buffers until the reply, and the child holds a full copy of each it received; the child's read-back pixels and the app's received copy are two more. At 1920 × 1080 that is about 33 MB in flight for one pass with one input layer where one copy each would be about 17 MB, and about 133 MB against 66 MB at 3840 × 2160, per export thread, since each has its own `opsis-renderer`.
- The pipe's own buffer holds about 64 KB; the duplicates are the two processes' buffers.
- `unsafe_code` is denied outside `opsis-vt`, `opsis-qt`, `opsis-skia` and `opsis-filament`; mapping memory is `unsafe`, and the user allows a crate of its own for it.

**Work.**

1. **`opsis-shm`.** A crate allowed `unsafe` that owns one shared buffer per renderer child: made by the app, sized for the largest message so far and grown when a frame needs more, mapped by both processes. Its `unsafe` stays in the mapping and in handing out byte slices whose lifetime the one-request-one-reply order bounds.
2. **Per platform.** `memfd` on Linux, `shm_open` on macOS, a named file mapping on Windows, behind one safe API; the child learns its buffer at start (an inherited descriptor on Linux and macOS, a name on Windows).
3. **Archives in the buffer.** The app writes each request's rkyv archive straight into the shared buffer and sends only its length over the pipe; the child checks it in place and writes its reply's archive into the same buffer, the pixels included, and answers with its length.
4. **Painting into it.** Where it saves a copy, the app paints a layer straight into the buffer (Skia drawing into memory it is given) instead of painting and then copying.
5. **Tests.** Messages of growing size through the shared buffer, a child that ends mid-message leaving the app with an error rather than a hang, and the export of an effects template matching its golden.

**Decided.**

- One shared buffer per child, owned by the app, reused for every request.
- The pipe stays for start, length and done signals; the order of requests and replies is what keeps the two processes from touching the buffer at the same time.
- Section 1's messages are used as they are.

**Decision for the user.**

- **Mapped file or anonymous memory.** A file in the child's own temp folder mapped by both works the same on every system with `memmap2` (MIT or Apache-2.0); `memfd`, `shm_open` and named mappings never touch a disk, which on macOS and Windows a temp file may.

**Risks.**

- Growing the buffer while the child holds a mapping needs both sides to remap between requests.
- A crash on either side must never leave the other waiting on a buffer; every wait has the pipe closing as its way out.

## Sources

- rkyv 0.8.18, MIT, minimum Rust 1.81: `cargo info rkyv`, checked 2026-11-22.
- memmap2 0.9.12, MIT or Apache-2.0: its `Cargo.toml`, checked 2026-11-22.
- memfd 0.6.6: `cargo search memfd`, checked 2026-11-22.
