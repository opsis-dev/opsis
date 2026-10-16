# "Ship It" — the crazy launch demo

A 2½-minute launch film for a fictional startup, **Shipyard**, that takes one feature from whiteboard sketch to "we're live and the money's in" — inside seven real open-source apps running locally in Docker. Every frame of product UI is real and driven by Demoflow; nothing is mocked in a design tool. It's a stress test: the point is to see how far the app goes, using almost every node type, both editor panels (Timeline, Mixer), pop-out windows and every export.

Status: built. The project is `~/.cache/demoflow-crazy/project`, the exports are in its `.demoflow/out/`, and the generator is `~/.cache/demoflow-crazy/tools/build-project.mts`. See **As built** at the end for what changed from this plan.

## Decisions

| Question          | Answer                                                                                                                                                                       |
| ----------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Apps              | All seven below. Plane and Penpot are out.                                                                                                                                   |
| Cuts              | One 2½-minute film. No 60-second cut.                                                                                                                                        |
| Audience          | Both audiences are built with **Choose by variable**; only **devs** is exported.                                                                                             |
| Exports           | Devs film in 16:9, 9:16, 1:1 and 4:5; stills in all five sizes; a GIF; a ProRes master; a web-ready MP4; Export as code.                                                     |
| Music and effects | Made locally with the engine's FFmpeg: an original ~120 BPM bed and every non-built-in effect. No downloads, no licence questions; recipes kept in `assets/audio/README.md`. |
| Fonts             | Bundled families everywhere; one title face, **Bricolage Grotesque**, downloaded through Settings → Fonts → Get fonts online.                                                |
| Start             | Nothing (containers, downloads, builds) starts until the user says go.                                                                                                       |

## Apps (one `docker compose`, all on localhost)

Everything lives in `~/.cache/demoflow-crazy/` (compose file, volumes, setup script, the project, renders). Each service has a memory limit and a compose **profile**, so the whole stack can run together or one act at a time if memory runs short. The user's other containers are never touched.

| App         | Image (pinned at build time)                   | Port | Role in the story                                                        | Setup                                                                                                                                                             |
| ----------- | ---------------------------------------------- | ---- | ------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Excalidraw  | `excalidraw/excalidraw`                        | 5000 | whiteboard sketch of the architecture                                    | none                                                                                                                                                              |
| Gitea       | `gitea/gitea`                                  | 3100 | pull request, review, merge                                              | `INSTALL_LOCK=true`; `gitea admin user create` inside the container (`shipyard` / `Demoflow!2026`); repo, branch and a waiting pull request through the REST API  |
| n8n         | `docker.n8n.io/n8nio/n8n`                      | 5678 | deploy pipeline drawn on its own node canvas (a node editor filming one) | owner account through n8n's owner-setup endpoint or CLI; if neither works on the pinned version, a **Sign in once** node does the first-run form                  |
| Grafana     | `grafana/grafana`                              | 3400 | launch-day traffic spike                                                 | admin through `GF_SECURITY_ADMIN_*`; TestData data source and the "Launch day" dashboard provisioned from files                                                   |
| Uptime Kuma | `louislam/uptime-kuma` (1.23.x, for its API)   | 3500 | "all systems green" status page                                          | admin, monitors and the public status page through `uptime-kuma-api` (Socket.IO); fallback: a hidden **Setup** take in the project that fills the first-run form  |
| Twenty CRM  | `twentycrm/twenty` + Postgres + Redis + worker | 3200 | the big deal moves to Won                                                | `SERVER_URL=http://localhost:3200`, a generated `APP_SECRET`; the first sign-up is filmed. `setup.sh reset twenty` wipes its volumes so every take signs up fresh |
| Metabase    | `metabase/metabase`                            | 3300 | revenue chart reveal                                                     | `GET /api/session/properties` → setup token → `POST /api/setup` creates the admin; questions on the sample database through the API                               |

- Memory: about 6–8 GB for all seven; this machine has 14 GB with ~6 GB free, so Twenty and Metabase get limits and the profiles allow act-by-act recording.
- Images are about 6 GB in total; they're pulled once, when work starts.
- `setup.sh` is idempotent: `up`, `seed`, `reset <app>`, `down`. Every take starts from the same seeded state.
- Each app's first-run flow is checked by hand before its act is built.

## Look and type

- **Project fonts** (all bundled): Text **Plus Jakarta Sans**, Headings **Fraunces**, Code **JetBrains Mono**.
- **Per-node fonts** (bundled): the neon sign in **Space Grotesk**, the handwriting in **Caveat**, the outro's punch lines in **Instrument Serif**.
- **Downloaded**: **Bricolage Grotesque** for the "SHIP IT." title, through the font library, so the project copy in `assets/fonts/` is exercised.
- **Brand colour**: coral `#FC6B33` (Demoflow's logo) for highlights, callouts and the logo reveal.

## The film

Element names below are the app's own titles.

| #   | Act                          | Length | What happens                                                                                                                                                                                                                                                                                                                                                            |
| --- | ---------------------------- | ------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 0   | Cold open                    | 8 s    | **Aurora** under **Shader God Rays**; a **Neon sign** flickers on "Shipyard presents"; "SHIP IT." **Matrix Decode**s, then **Rgb Glitch Text**; **Colour split slide** into Act 1.                                                                                                                                                                                      |
| 1   | The idea — Excalidraw        | 15 s   | The cursor draws the architecture with shortcuts (R, A, T) while **Keystroke chips** show each key; **Handwriting** writes "the plan"; **Pan** glides across; **Ink Arrow** and **Scribble Circle** land on "API"; **File upload** drops the spec in; a **Screenshot** becomes a **Polaroid**; **Page Turn** out.                                                       |
| 2   | The code — Gitea             | 20 s   | Open the pull request; **Code diff** replays the change; **Commit graph** draws branch and merge; **Find element early** locates Merge mid-move; **Spotlight** and **Tooltip** on Merge, click; **Check text** confirms "Merged" or the take fails; **Mosaic** hides the access token; **Lower third** "Gitea · code review"; **Upload file** sets the Shipyard avatar. |
| 3   | The pipeline — n8n           | 18 s   | Drag three nodes, connect them, Execute; **Run page script** closes the onboarding pop-up first; **Speed ramp** flies through the run; **Crt Screen** around a **Terminal Simulator** typing deploy logs; a **Lottie animation** checkmark per green step; **File tree** of the build; **Blinds** out.                                                                  |
| 4   | Launch day — Grafana         | 20 s   | **Fake a response** feeds a perfect spike; **Freeze time** pins launch morning; **Choose option** sets the time range; **Zoom here** on the peak with **Callout arrow** "uh oh 🚀"; **Gauge** into the red; **Metric cards** count up; light and dark takes (session colour scheme) joined with **Pixel grid wipe**.                                                    |
| 5   | Still standing — Uptime Kuma | 12 s   | A second **Open browser** with Mobile on records the status page as a phone in a **Device frame**; **Device lineup** of phone, tablet and laptop; **Notification stack** "All systems operational"; **Status Cycle** "Deploying → Live"; **Whip Pan** out.                                                                                                              |
| 6   | The deal — Twenty CRM        | 18 s   | Real sign-up on camera (typing with pauses); **Mosaic** over the contacts' emails; drag the deal card to Won; **Confetti** (**Repeat** ×3) and **Radial Burst**; **Stat card** with the amount; **Marker Highlight** on "$240,000"; **Frosted glass wipe** out.                                                                                                         |
| 7   | The money — Metabase         | 12 s   | Scroll to a chart, **Hover** the bars; **Bar chart race** of plans by revenue, **Funnel chart** sign-up → trial → paid, **Before and after bars** for launch week; **Plan comparison table** and **Pricing card**; **Zoom Blur** out.                                                                                                                                   |
| 8   | Rewind gag                   | 4 s    | "But wait, how did we get here?" — a half-size **Export video** of acts 1–7 goes through **Reverse** and **Change speed** (4×), comes back with **Use as clip** under a **Vhs Filter**, then **Tv Power Off** snaps back.                                                                                                                                               |
| 9   | Outro                        | 10 s   | **Logo wall** warms from grey to colour; **Logo reveal** draws Shipyard's mark; **Punch Lines** cut on the beat; **End card** with a typed URL; **Shape morph** to black; **Light leak cut** on the last frame.                                                                                                                                                         |

Act cards come from **Chapter** nodes; an **HTML overlay** puts a "LIVE" badge on the Grafana take; **Hide clutter** removes banners; **Wait** settles slow pages. Captions run karaoke-style from a script (**Captions from script** → **Show captions**).

**Choose by variable** on `Audience`: "devs" keeps acts 2 and 3 (code, pipeline); "execs" swaps in a **Data story** and a **Donut chart** recap. Only devs is exported.

## Audio

No voice-over; captions carry the narration.

### Music and effects (made locally)

- **Music**: an original ~2½-minute bed at 120 BPM made with FFmpeg's sound generators: kick and hat on the grid, a chord pad, a bass line, a build into act 4 and a drop at act 6. Rendered to `assets/audio/music.wav`.
- **Effects**: hum, glitch, pops, chimes, riser, impact, ding, cash register, confetti pop, tape rewind, made the same way into `assets/audio/sfx/`.
- **Built-in** (CC0, shipped): Whoosh, Whip, Mouse click, Switch, Page turn, both shutters.
- The recipe for every file is in `assets/audio/README.md`.

### Music in the film

- **Trim file** to the cut length, then a **Music** node with fades. **Fit length** holds the final film to the music.
- The Timeline's beat markers snap every act cut and every outro punch line to the beat; the outro is ripple-trimmed so the last hit lands on the end card.
- Volume keyframes lift the music into the act 6 confetti and drop it to silence for the rewind.

### Mix

| Bus     | What's on it       | Processing                                                                      |
| ------- | ------------------ | ------------------------------------------------------------------------------- |
| Music   | the music bed      | low cut at 40 Hz; ducks under Effects (about 8 dB, fast attack, 300 ms release) |
| Effects | every sound effect | "Punchy effects" preset; EQ cut around 3 kHz on the harsher clicks              |
| Clips   | clips with sound   | none today (the apps are silent); kept for later                                |
| Master  | everything         | gentle EQ tilt, light compressor, limiter at −1 dBTP, loudness target −14 LUFS  |

Meters must read −14 LUFS integrated and a true peak under −1 dBTP before exporting. A "music only" solo state and a "no music" mute state are saved check renders.

### Sound-effect cue sheet

Cues are **Sound effect** nodes placed in the Timeline, snapped to action marks. ★ = built in.

| Act | Moment                           | Sound                          |
| --- | -------------------------------- | ------------------------------ |
| 0   | Neon sign strikes; title decodes | hum; glitch                    |
| all | Act transitions                  | ★ Whoosh / ★ Whip, alternating |
| 1   | Shapes snap; Polaroid            | pops; ★ Camera shutter         |
| 1   | Page turn                        | ★ Page turn                    |
| 2   | Merge click                      | ★ Mouse click + impact         |
| 3   | Execute; checks turn green       | ★ Switch; chimes               |
| 4   | Spike hits the peak              | riser into impact              |
| 5   | Notification slides in           | ding                           |
| 6   | Deal lands in Won                | cash register + confetti pop   |
| 8   | Rewind                           | tape rewind                    |
| 9   | Logo drawn; end card             | impact; ★ Old camera shutter   |

## Node usage

| Group                          | Used                                                                                                                                                                                                                                                                 |
| ------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Browser (all 18)               | every action, including Upload file, Choose option, Run page script, Fake a response, Freeze time, Hide clutter, Chapter, HTML overlay, Wait, Press keys, Find element early, Check text                                                                             |
| Record (all 3)                 | a Record take per app (two for Grafana, two for Uptime Kuma); Screenshot for the Polaroid; Sign in once wherever scripting a login fails                                                                                                                             |
| Camera (all 10)                | Auto zoom everywhere, Zoom here, Pan, Follow cursor (9:16), Skip quiet parts, Speed ramp, Crop, Device frame, Backdrop per act, Motion blur                                                                                                                          |
| Cursor & text (all 9)          | Cursor, Click ripple, Keystroke chips, Spotlight, Highlight box, Tooltip, Callout arrow, Captions from script, Show captions                                                                                                                                         |
| Graphics, backgrounds, effects | about 45 elements named in the acts above, plus Shader Mesh Gradient and Shader Grain Gradient act backgrounds and Speed Lines on the spike                                                                                                                          |
| Transitions                    | a different one between every act: Colour split slide, Page Turn, Blinds, Pixel grid wipe, Whip Pan, Frosted glass wipe, Zoom Blur, Shape morph, Light leak cut                                                                                                      |
| Timeline (all 8 nodes)         | Sequence, Layers, Trim, Repeat, Delay, Speed, Fit length, Choose by variable                                                                                                                                                                                         |
| Timeline panel                 | beat-snapped cuts, ripple trims, a split at the playhead, roll and slip on the act cuts, volume and opacity keyframes, action marks to place effects; every edit writes back to nodes                                                                                |
| Sound & media (all 17)         | Music, Sound effect ×15+, Add sound; Media file, Use as clip, Trim file, Join files (raw takes into a "making of" file), Change speed, Reverse, Resize, Convert (ProRes), Make GIF, Extract sound, Even out volume (on one effect), Grab frame, Web-ready, File info |
| Mixer panel                    | three buses with sidechain ducking, EQ and compressor per strip, presets, master limiter and −14 LUFS target, saved mute and solo states                                                                                                                             |
| Export (all 3)                 | Export video (four shapes, follow the action), Export images (all five sizes), Export as code                                                                                                                                                                        |
| Organise                       | a Group per act with a Note explaining it                                                                                                                                                                                                                            |

## How it gets built

1. **Stack**: write the compose file and `setup.sh`; pull images; bring the stack up; seed; check each app's first-run flow by hand.
2. **Assets**: generate the music and effects; download Bricolage Grotesque through the app.
3. **Project**: a generator script (`build-project.mts`, using the app's own graph builder) writes `project.demoflow.json` with every group, node and edge, so the 220-node graph is reproducible. Selectors are confirmed in each app with the element picker.
4. **Takes**: run each act's Record take on its own until it's clean (the Run panel, live view and trace show what failed).
5. **Edit in the app**: open the project and do the panel work by hand through the UI: beat-snapped cuts, ripple trims, keyframes, effect placement, mixer settings. The Timeline is popped out into its own window for part of this, to stress the window sync.
6. **Export**: Run. Check card stills give a readable canvas, meters read −14 LUFS, and mosaic boxes cover the private text in every shape.
7. **Report**: the export folder path, render times, and every bug found in Demoflow with what was fixed.

## Limits and risks

1. No voice-over; captions carry the narration.
2. Rendering will be slow: seven recordings, heavy shaders, four shapes, a half-size draft for the rewind, plus stems. Expect 30–60 minutes.
3. Mosaic boxes are fixed on the frame and don't follow the camera; each zoom over a token or email is checked, and tall shapes get their own boxes.
4. Unconfirmed first-run flows (n8n's owner setup, Twenty's sign-up, Uptime Kuma's admin) get Sign in once or a Setup take as fallbacks.
5. Twenty signs up once per database; takes reset its volumes first.
6. Memory: if the full stack doesn't fit next to the user's other containers, acts are recorded one profile at a time.
7. A project this size will find bugs in Demoflow. Each one is fixed and reported.
8. The downloaded title font's licence (SIL Open Font License for Bricolage Grotesque, to be confirmed in the library) must allow use in video.

## Sources

- Twenty: https://docs.twenty.com/developers/self-hosting · https://www.mintlify.com/twentyhq/twenty/developers/self-hosting/configuration · https://www.mintlify.com/twentyhq/twenty/developers/self-hosting/docker-compose · https://ramnode.com/guides/twenty-crm
- Metabase setup API: https://www.metabase.com/docs/master/api/setup · https://discourse.metabase.com/t/post-api-setup-token/9121
- Uptime Kuma: https://pypi.org/project/uptime-kuma-api · https://github.com/lucasheld/ansible-uptime-kuma/wiki/setup · https://oneuptime.com/blog/post/2026-02-08-how-to-run-uptime-kuma-in-docker-for-status-monitoring/markdown
- n8n: https://community.latenode.com/t/how-to-automatically-configure-admin-user-for-self-hosted-n8n-deployment/27058 · https://www.hostinger.com/tutorials/n8n-api · https://www.digitalocean.com/community/tutorials/how-to-setup-n8n
- Grafana in Docker: https://lumadock.com/tutorials/how-to-run-grafana-with-docker-compose
- Excalidraw: https://sliplane.io/blog/self-hosting-excalidraw-the-easy-way

## As built

The film kept the story and the music map; anything that made it a feature list was cut.

| Act           | Time      | Changes from the plan                                                                                                                                                                                                  |
| ------------- | --------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 0 Cold open   | 0:00–0:08 | As planned. The decode and glitch titles are drawn over the aurora (they needed a transparent background).                                                                                                             |
| 1 Whiteboard  | 0:08–0:24 | The sketch is loaded before the page opens; on camera the text tool adds two notes (Excalidraw can't draw shapes from clicks). Ink circle, arrow and "the plan" sit on the Polaroid. Pan and File upload were dropped. |
| 2 Code review | 0:24–0:42 | Sign in once for the login. Mosaic and the avatar upload were dropped (nothing private on screen, not part of the story).                                                                                              |
| 3 Pipeline    | 0:42–0:58 | Execute on the existing workflow; no dragging (Demoflow has no drag action). Lottie and File tree dropped.                                                                                                             |
| 4 Launch day  | 0:58–1:18 | One dark take. The card was lengthened in the Timeline so the traffic peak lands on the drop at 1:02. Fake a response, Freeze time, Choose option, Gauge and the light take were dropped.                              |
| 5 Status page | 1:18–1:30 | Device lineup of phone, tablet and laptop with notifications, then the status cycle.                                                                                                                                   |
| 6 The deal    | 1:30–1:48 | Signed in (the workspace was set up off camera); the stage is changed in the deal panel. Celebration is the burst, "Closed: $240,000" and confetti ×3.                                                                 |
| 7 The money   | 1:48–2:02 | Hover the plan bars, scroll to monthly revenue, then the funnel. Bar race, comparison bars, table and pricing card dropped.                                                                                            |
| 8 Rewind      | 2:02–2:06 | As planned, at 8× and squeezed to 4 s, with "But wait. How did we get here?" captions.                                                                                                                                 |
| 9 Outro       | 2:06–2:16 | Logo wall of the seven apps, punch lines, logo reveal, end card, shape morph to black.                                                                                                                                 |

Transitions between acts: Colour split slide, Page Turn, Light leak cut, Blinds, Pixel grid wipe, Whip Pan, Frosted glass wipe, Glitch Cut. Choose by variable swaps acts 2–3 for an execs version (Data story and Donut chart); only devs is exported.

Mix: music ducks 8 dB under the effects bus (39 ms attack, 298 ms release), Music bed preset on the music, Punchy effects pasted onto every effect strip, master EQ +1.5 dB shelves, light compressor, limiter at −2 dB, −14 LUFS target. The music is lifted around the confetti with Timeline keyframes.
