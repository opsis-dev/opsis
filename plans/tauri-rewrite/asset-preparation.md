# Asset preparation

Work the video bundle does on every load today, done once in Rust and cached: 3D text typefaces, decoded 3D models, sized textures, simplified SVG logos, and the font library's font inspection. The bundle then fetches prepared files by address and stays free of Node code.

Crate: `opsis-prepare` (phase 7).

## What it adds

- `fontkit` (5.6 MB) and `woff2-encoder` (1.3 MB) leave the video bundle; the Draco decoder (`public/three`, 0.76 MB) leaves `public/`.
- 3D text and models load faster in every render tab and in the streamed preview, because parsing happens once per file and setting.
- `KHR_texture_basisu` models, refused today, could be converted here later.

## Fit with Opsis

Already in place:

- **3D text** (`src/video/three/outlines.ts`): fetches the font, decompresses WOFF2 with `woff2-encoder` (fontkit's variations break on WOFF2), opens it with `fontkit`, sets the `wght` axis, and converts 564 code points (`0x20–0x7e`, `0xa0–0x24f`, `0x2010–0x2027`, `0x2030–0x203a`, `€`, `™`) into three.js typeface JSON.
- **Models** (`src/video/three/load-model.ts`): reads the GLB header for extensions, refuses `KHR_texture_basisu`, and decodes with `GLTFLoader`, `DRACOLoader` (decoder files in `public/three/draco`, written by `bun run three:decoders`) and `MeshoptDecoder`.
- **Textures and SVG** (`src/video/three/load.ts`): `TextureLoader` and `SVGLoader().parse`; `logo-3d.tsx` extrudes the shapes.
- **Font library** (`src/main/fonts.ts`): `fontkit` reads names, weights and italics; `planFontFiles` keeps one file per style.
- Every load is held with `useHeldLoad` under `delayRender`.

Missing:

- A preparation step and an address for prepared files.

## Work

1. **Prepared-file route.** Rust serves `…/prepared/<kind>?src=<project: or app: ref>&<settings>` on both paths the shell uses: the asset server for render tabs and the preview page (`sidecar.md`, Work step 7). The first request prepares the file into `.opsis/cache/prepared/<sha256 of kind, source stamp, settings, PREPARED_VERSION>` and later requests serve it. `src/video` still only fetches an address.
2. **3D text typefaces.** `wuff` 0.2.9 (MIT, pure-Rust WOFF and WOFF2 decoding) then `skrifa` 0.49.0 (MIT OR Apache-2.0) to draw each glyph's outline at the requested `wght` with an outline pen, writing the same typeface JSON `outlines.ts` produces. `outlines.ts` becomes a fetch of `prepared/typeface?src=…&wght=…`. Test: for the 16 bundled families, compare glyph advance widths and outline point counts against today's output.
3. **Models.** Decode Draco-compressed GLBs to plain GLBs with `draco-gltf` 0.7.0 (Apache-2.0; reads ordinary accessors or decodes Draco and can write them back as ordinary accessors), and meshopt-compressed ones with the `meshopt` crate 0.7.0 (MIT OR Apache-2.0, FFI to meshoptimizer), using `gltf` 1.4.1 for the file structure. `load-model.ts` then uses plain `GLTFLoader`; `DRACOLoader`, `MeshoptDecoder`, `public/three/draco` and `three:decoders` go. Test: vertex counts and bounding boxes match three.js's own decode for each test model.
4. **Textures.** Images above a size cap (proposed: 4096 px on the long side) are resized with `image`/`fast_image_resize` before the shell sees them; GPU memory, not speed, is the gain.
5. **SVG logos.** `usvg` 0.49.0 (Apache-2.0 OR MIT) flattens transforms, `use` elements and styles into plain paths; `SVGLoader` then parses a simple file. The extrusion stays in three.js.
6. **Font library.** `skrifa`/`read-fonts` replace `fontkit` in the Rust port of `fonts.ts` (`desktop-shell.md`, Work step 11).
7. **Version.** `PREPARED_VERSION` in Rust keys every prepared file; bump it whenever a preparer's output changes (like `PICTURE_VERSION`).
8. **Packages.** `bun remove fontkit woff2-encoder` from `packages/video` once nothing imports them.

## Risks

- **Young Draco crates.** `draco-gltf` is new (0.7.0 released 2026-10-10, 3,953 recent downloads on crates.io); compare against three.js's decoder on many files before deleting the wasm. `draco_decoder` (0.0.31) calls itself work in progress.
- **Outline parity.** skrifa and fontkit may differ in curve conversion; compare rendered 3D text stills before and after.

## Sources

- `src/video/three/outlines.ts`, `load-model.ts`, `load.ts`, `logo-3d.tsx`, `src/main/fonts.ts` (read 2026-10-25); `du -sb` of `node_modules/fontkit`, `node_modules/woff2-encoder`, `public/three` (2026-10-25).
- crates.io API for `wuff`, `skrifa`, `read-fonts`, `draco-gltf`, `draco_decoder`, `meshopt`, `gltf`, `usvg`, `image`, `fast_image_resize` (2026-10-25); [draco-gltf on docs.rs](https://docs.rs/crate/draco-gltf/latest) (2026-10-25).
