# Third-party notices

Opsis vendors source from the registries and projects below. Their licences require the notices to be kept.

## Remocn — packages/video/src/elements/remocn, packages/video/src/lib/remocn*

```
MIT License

Copyright (c) 2026 Remocn

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## snapcn — packages/video/src/elements/snap-cn, packages/video/src/lib/snap-cn-ui

```
MIT License

Copyright (c) 2026 Sri Nath (snap-cn)

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## RemotionUI — packages/video/src/elements/remotion-ui, packages/video/src/lib/remotion-ui

From https://github.com/riaz37/remotion-ui (commit b7e0e6becc3d22b8b03dfef72c064f72ff9fad1f), installed from the https://remotionui.com registry.

```
MIT License

Copyright (c) 2026 RemotionUI

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## programatic-demo — packages/video/src/camera, src/worker/capture, public/backdrops

```
MIT License

Copyright (c) 2026 ashrafchowdury

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## remotion-dev/template-electron — src/worker/render/{compositor-package,bundle,render}.ts, forge.config.ts (adapted)

Used under the Remotion licence (https://www.remotion.dev/docs/license).

## FFmpeg (ffmpeg / ffprobe)

The FFmpeg binaries are GPLv3 builds from the pinned ffmpeg-static GitHub release, shipped as separate executables in the engine folder (resources/engine/ffmpeg in the full edition, or downloaded into the app's data folder by the lite edition) together with their licence file. Source: https://ffmpeg.org/download.html · builds: https://github.com/eugeneware/ffmpeg-static

## Chrome Headless Shell (recording and export browser)

The full edition bundles Google's Chrome Headless Shell from Chrome for Testing (Chromium, BSD-3-Clause, plus the licences of every library it includes) in `resources/engine`. Its complete licence notices are in `resources/engine/LICENSES.chromium.html`, saved at build time from the `chrome://credits` page of the full Chrome for Testing build of the exact same version. The lite edition doesn't include the browser; Settings downloads it from Google onto the user's computer. Source: https://chromium.googlesource.com/chromium/src · builds: https://googlechromelabs.github.io/chrome-for-testing/

## Remotion

Videos are rendered with Remotion (https://www.remotion.dev), which is not under an open-source licence: it is free for individuals, non-profits and for-profit organisations with up to 3 employees, and larger companies need a Remotion company licence (https://www.remotion.dev/license). Opsis passes the key entered in Settings → Video export licence, or the free licence when it is empty. Remotion packages are installed from npm and keep their own `LICENSE.md`.

## React Flow UI — apps/ui/src/components/{base-node,labeled-handle,…}.tsx

MIT, https://github.com/xyflow/xyflow

## Lucide icons — apps/ui/src/editor/node-glyphs.generated.ts, and the lucide-react package

Node icons copied from the `lucide` package by `bun run glyphs:bundle`, and the editor's other icons from `lucide-react`. Lucide is under the ISC licence; the icons it derives from Feather are also under Feather's MIT licence, both below. https://lucide.dev

```
ISC License

Copyright (c) 2026 Lucide Icons and Contributors

Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted, provided that the above
copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

---

The following Lucide icons are derived from the Feather project:

airplay, alert-circle, alert-octagon, alert-triangle, aperture, arrow-down-circle, arrow-down-left, arrow-down-right, arrow-down, arrow-left-circle, arrow-left, arrow-right-circle, arrow-right, arrow-up-circle, arrow-up-left, arrow-up-right, arrow-up, at-sign, calendar, cast, check, chevron-down, chevron-left, chevron-right, chevron-up, chevrons-down, chevrons-left, chevrons-right, chevrons-up, circle, clipboard, clock, code, columns, command, compass, corner-down-left, corner-down-right, corner-left-down, corner-left-up, corner-right-down, corner-right-up, corner-up-left, corner-up-right, crosshair, database, divide-circle, divide-square, dollar-sign, download, external-link, feather, frown, hash, headphones, help-circle, info, italic, key, layout, life-buoy, link-2, link, loader, lock, log-in, log-out, maximize, meh, minimize, minimize-2, minus-circle, minus-square, minus, monitor, moon, more-horizontal, more-vertical, move, music, navigation-2, navigation, octagon, pause-circle, percent, plus-circle, plus-square, plus, power, radio, rss, search, server, share, shopping-bag, sidebar, smartphone, smile, square, table-2, tablet, target, terminal, trash-2, trash, triangle, tv, type, upload, x-circle, x-octagon, x-square, x, zoom-in, zoom-out

The MIT License (MIT) (for the icons listed above)

Copyright (c) 2013-present Cole Bemis

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## Tabler Icons — apps/ui/src/editor/node-glyphs.generated.ts

Node icons copied from the `@tabler/icons` package by `bun run glyphs:bundle`. https://tabler.io/icons

```
MIT License

Copyright (c) 2020-2026 Paweł Kuna

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## morphicons — the morphicons package

Draws node icons that change shape on hover. Installed from npm and bundled into the editor. https://www.morphicons.com

```
MIT License

Copyright (c) 2026 Guillermo

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## Dice UI Sortable and Media Player — apps/ui/src/components/ui/sortable.tsx, apps/ui/src/components/ui/media-player.tsx

The drag-to-reorder lists in the Inspector, and the player for videos and sounds in the Run panel and Replay. Installed from the Dice UI registry with the shadcn CLI. https://www.diceui.com

```
MIT License

Copyright (c) 2024 Sadman Sakib

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## dnd-kit — the @dnd-kit/core, @dnd-kit/sortable, @dnd-kit/modifiers and @dnd-kit/utilities packages

Drag and drop for the sortable lists. Installed from npm and bundled into the editor. https://dndkit.com

```
MIT License

Copyright (c) 2021, Claudéric Demers

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## Media Chrome — the media-chrome package

Keeps the state of the player for videos and sounds in the Run panel and Replay. Installed from npm and bundled into the editor. https://www.media-chrome.org

```
MIT License

Copyright (c) 2020 Mux, Inc.

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## 3D, letter outlines and hand-drawn marks — the three, @react-three/fiber, @remotion/three, fontkit, woff2-encoder, @remotion/rough-notation and roughjs packages

Draw the 3D elements (3D logo, card, text, model, screen and floating shapes), read font files for 3D text, and draw the hand-drawn marks of Highlight box and Title card. Installed from npm and bundled into the video code that loads only when one of these is used. All are under the MIT License:

- three.js: Copyright © 2010-2026 three.js authors, https://threejs.org
- React Three Fiber (@react-three/fiber): by Paul Henschel, https://github.com/pmndrs/react-three-fiber (the package ships no licence file; its package.json names MIT)
- @remotion/three: by Björn Zeutzheim, https://www.remotion.dev/docs/three (the package ships no licence file; its package.json names MIT)
- fontkit and brotli: by Devon Govett, https://github.com/foliojs/fontkit (the packages ship no licence file; their package.json names MIT)
- woff2-encoder: Copyright (c) 2023-PRESENT Kyedo, https://github.com/itskyedo/woff2-encoder, a WebAssembly build of Google's woff2 decoder
- @remotion/rough-notation: by Jonny Burger, https://www.remotion.dev/docs/rough-notation (the package ships no licence file; its package.json names MIT)
- Rough.js (roughjs): Copyright (c) 2019 Preet Shihn, https://roughjs.com
- meshoptimizer decoder (three/addons/libs/meshopt_decoder.module.js): Copyright (C) 2016-2026 Arseny Kapoulkine, https://github.com/zeux/meshoptimizer

```
MIT License

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## Draco decoder — public/three/draco

Google's Draco 3D mesh decoder, copied from three.js (`three/examples/jsm/libs/draco/gltf`) by `bun run three:decoders`, so 3D models with compressed meshes open offline. By Google, https://github.com/google/draco. Licensed under the Apache License, Version 2.0, the same licence as Opsis; its full text is in `LICENSE`.

## Drizzle ORM — the drizzle-orm package

Reads and writes the app's saved state (settings, recent projects, window positions, the font library and each project's view). Installed from npm and bundled into the app. By Drizzle Team, https://github.com/drizzle-team/drizzle-orm

Licensed under the Apache License, Version 2.0, the same licence as Opsis; its full text is in `LICENSE`. The package ships no NOTICE file.

## Built-in sound effects — public/sfx

Public domain (Creative Commons 0, https://creativecommons.org/publicdomain/zero/1.0/). No attribution is required; credits are kept here. Files were taken from the set Remotion lists in `@remotion/sfx` (https://www.remotion.dev/docs/sfx), with each licence checked on its source page.

| File               | Sound                                                    | Source                          |
| ------------------ | -------------------------------------------------------- | ------------------------------- |
| whoosh.wav         | Woosh by 1bob                                            | https://freesound.org/s/831936/ |
| whip.wav           | SWSH_Badminton Racquet_Recording_01_JW Audio by JW_Audio | https://freesound.org/s/838766/ |
| mouse-click.wav    | Mouse Click Sound.mp3 by Pixeliota                       | https://freesound.org/s/678248/ |
| switch.wav         | UI Audio - Switch 35 by Kenney                           | https://kenney.nl/              |
| page-turn.wav      | Draw Knife 1 by Kenney                                   | https://kenney.nl/              |
| shutter-modern.wav | DSLR Shutter fast 006.wav by ristooooo1                  | https://freesound.org/s/539136/ |
| shutter-old.wav    | Werra.wav by hmilleo                                     | https://freesound.org/s/409093/ |

## Bundled fonts — public/fonts/bundled

SIL Open Font License 1.1, taken from the Fontsource packages. Each family's licence text is next to its files in `public/fonts/bundled/<family>/LICENSE`: Inter, Geist, Geist Mono, JetBrains Mono, Manrope, DM Sans, Space Grotesk, Outfit, Montserrat, Plus Jakarta Sans, IBM Plex Sans, Playfair Display, Fraunces, Caveat, Google Sans Flex, Instrument Serif.
