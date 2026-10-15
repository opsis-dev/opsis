# Third-party notices

Demoflow vendors source from the registries and projects below. Their licences require the notices to be kept.

## Remocn — src/video/elements/remocn, src/video/lib/remocn*

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

## snapcn — src/video/elements/snap-cn, src/video/lib/snap-cn-ui

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

## RemotionUI — src/video/elements/remotion-ui, src/video/lib/remotion-ui

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

## programatic-demo — src/video/camera, src/worker/capture, public/backdrops

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

Videos are rendered with Remotion (https://www.remotion.dev), which is not under an open-source licence: it is free for individuals, non-profits and for-profit organisations with up to 3 employees, and larger companies need a Remotion company licence (https://www.remotion.dev/license). Demoflow passes the key entered in Settings → Video export licence, or the free licence when it is empty. Remotion packages are installed from npm and keep their own `LICENSE.md`.

## React Flow UI — src/renderer/components/{base-node,labeled-handle,…}.tsx

MIT, https://github.com/xyflow/xyflow

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
