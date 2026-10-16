# "Ship It" — the crazy launch demo

A 2½-minute launch film for a fictional startup, **Shipyard**, that takes one feature from whiteboard sketch to "we're live and the money's in" — inside seven real open-source apps running locally in Docker. Every frame is real UI driven by Demoflow; nothing is mocked in a design tool. The goal is to exercise almost every node type and editor panel: about 220 nodes in 9 acts.

Status: plan only. Nothing here is built yet. Everything it uses exists in the app today, including the Timeline and Mixer.

## Apps (one `docker compose`, all on localhost)

| App         | Role in the story                                                                  | Port | Login                                                                       |
| ----------- | ---------------------------------------------------------------------------------- | ---- | --------------------------------------------------------------------------- |
| Excalidraw  | whiteboard sketch of the architecture                                              | 5000 | none                                                                        |
| Gitea       | pull request, review, merge                                                        | 3100 | `shipyard` / `Demoflow!2026`, created by the setup script                   |
| n8n         | deploy pipeline drawn on its own node canvas (a node editor filming a node editor) | 5678 | owner account created by the setup script                                   |
| Grafana     | launch-day traffic spike                                                           | 3400 | `admin` / `Demoflow!2026` via environment variables; dashboards provisioned |
| Uptime Kuma | "all systems green" status page                                                    | 3500 | admin created on first start                                                |
| Twenty CRM  | the big deal moves to Won                                                          | 3200 | first sign-up becomes the owner — filmed on camera                          |
| Metabase    | revenue chart reveal                                                               | 3300 | admin created by the setup script, with the sample database                 |

Optional extra act: **Plane** (issue tracker). It runs about a dozen containers, so it stays optional.

Infrastructure:

- The whole stack needs roughly 6–8 GB of RAM.
- Compose file, data volumes and renders live in `~/.cache/demoflow-crazy/` (large files never go in `/tmp`).
- A setup script seeds every app through its own API so each take starts from the same state: users, a repository with a waiting pull request, Grafana dashboards, Uptime Kuma monitors, CRM deals, Metabase questions.
- Before building on any app, its first-run flow is checked by hand. Twenty's and n8n's first-run screens are not confirmed in their official docs (one n8n guide says it emails an activation key, another says it does not). Where scripting fails, fall back to a **Sign in once** node.

## Look and type

- **Project fonts:** Text **Plus Jakarta Sans**, Headings **Fraunces** (big serif titles against product UI), Code **JetBrains Mono**. Every element follows them; the terminal and code acts pick up the code font automatically.
- **Per-node fonts:** the cold open's neon sign in **Space Grotesk**; the Excalidraw act's handwriting element in **Caveat**; the outro's punch lines in **Instrument Serif** italic. One display face downloaded through **Get fonts online** (e.g. Bricolage Grotesque) for the "SHIP IT." title, so the font library and the project copy in `assets/fonts/` are both exercised.
- **Brand colour:** Shipyard's accent is coral (`#FC6B33`, the same as Demoflow's logo), used for highlights, callouts and the logo reveal.

## The film

| #   | Act                          | Length | What happens                                                                                                                                                                                                                                                                                                                                                          |
| --- | ---------------------------- | ------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 0   | Cold open                    | 8 s    | Aurora background under god rays; a neon sign flickers on "Shipyard presents"; "SHIP IT." matrix-decodes and glitches in RGB; a colour-split slide transition into Act 1.                                                                                                                                                                                             |
| 1   | The idea — Excalidraw        | 15 s   | The cursor draws the architecture live with keyboard shortcuts (R, A, T) while keystroke chips show every key; a handwriting element writes "the plan" beside it; a pan glides across the diagram; ink arrow and scribble circle land on "API"; a file-upload element drops the spec PDF in; it freezes into a Polaroid still and leaves with a page-turn transition. |
| 2   | The code — Gitea             | 20 s   | Open the pull request; a code-diff element replays the change; a commit graph draws the branch and merge; Find element early locates Merge mid-camera-move; spotlight and tooltip on Merge, then click; Check text confirms "Merged" or the take fails before export; a mosaic hides the access token in the settings panel; lower third "Gitea · code review".       |
| 3   | The pipeline — n8n           | 18 s   | Drag three nodes onto n8n's canvas, connect them, press Execute; a speed ramp flies through the run; a CRT terminal types out deploy logs in the code font; a Lottie checkmark animation plays as each step turns green; a file tree shows the build output; blinds transition out.                                                                                   |
| 4   | Launch day — Grafana         | 20 s   | Fake a response feeds a perfectly shaped traffic spike (same every take); Freeze time pins the clock to launch morning; zoom on the peak with a callout arrow "uh oh 🚀"; a gauge dial swings into the red; metric cards count up requests, errors and latency; light vs dark theme of the same dashboard joined with a pixel-grid wipe.                              |
| 5   | Still standing — Uptime Kuma | 12 s   | A second Open browser with Mobile on records the status page as a phone; a device lineup shows the same page on phone, tablet and laptop; a notification stack slides in "All systems operational"; a status-cycle element flips "Deploying → Live".                                                                                                                  |
| 6   | The deal — Twenty CRM        | 18 s   | Real sign-up on camera (typing with pauses); a mosaic hides the contacts' emails in the list; drag the deal card to Won; confetti, radial burst and the deal amount in a stat card; marker highlight on "$240,000"; frosted-glass wipe out.                                                                                                                           |
| 7   | The money — Metabase         | 12 s   | Scroll to a chart, hover the bars for tooltips; a bar-chart race ranks the plans by revenue, a funnel shows sign-up → trial → paid, and before/after bars show the launch week; a comparison table and pricing card close the act; zoom-blur transition.                                                                                                              |
| 8   | Rewind gag                   | 4 s    | "But wait, how did we get here?" — the whole film plays backwards at 4× (Reverse + Change speed) under a VHS effect, then snaps back.                                                                                                                                                                                                                                 |
| 9   | Outro                        | 10 s   | A logo wall of "teams that ship with us" warms from grey to colour; Shipyard's logo is drawn by the logo-reveal element; punch-line cards cut on the beat; an end card with a typed URL and a shape-morph to black.                                                                                                                                                   |

Card stills make the 220-node canvas readable while building it: each act's nodes show a saved frame once they've been previewed or exported, with no limit on how many are visible.

## Audio

There is no voice-over (TTS and transcription stay out of scope); narration is on-screen captions timed from a script. With the Mixer, the film gets a real mix that the export reproduces exactly.

### Music

- One instrumental track of about 2½ minutes, ideally around 120 BPM.
- **Trim file** to the cut length and a **Music** node with a fade in and a long fade out. **Even out volume** is no longer needed on the music: the master's loudness target handles levels.
- In the **Timeline** panel, beat markers from the music snap every act cut and every punch-line card in the outro to the beat; the outro is trimmed with ripple edits so the last hit lands on the end card.
- **Fit length** holds the final timeline to the music, not the other way round.
- The 60-second cut (if wanted) uses a second, shorter track rather than chopping the first.

### Mix

| Bus     | What's on it                                   | Processing                                                                                            |
| ------- | ---------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| Music   | the music track                                | high-pass at 40 Hz; sidechain ducking from the Effects bus (about −8 dB, fast attack, 300 ms release) |
| Effects | every sound effect                             | gentle compressor so hits sit at one level; EQ cut around 3 kHz on the harsher clicks                 |
| Clips   | any recording or video file with its own sound | muted by default (the apps are silent); kept as a bus so a narrated clip could be dropped in later    |
| Master  | everything                                     | EQ tilt, light compressor, limiter at −1 dBTP, loudness target **−14 LUFS** (web)                     |

- Ducking replaces the simple "Lower under other sounds" setting on the Music node, which migrates into the Music bus's sidechain.
- Volume keyframes in the Timeline lift the music into the act 6 confetti moment and drop it to silence for the rewind gag.
- Meters confirm the master lands at −14 LUFS integrated with true peak under −1 dBTP before exporting.
- Mute and solo are saved with the project, so a "music only" check render and a "no music" social cut are just Mixer states.

### Sound-effect cue sheet

Each cue is a **Sound effect** node; built-in sounds are marked ★ (CC0, shipped with Demoflow). Cues are placed and nudged in the Timeline, snapped to action marks (clicks, keystrokes) on the recording bars.

| Act | Moment                           | Sound                                        |
| --- | -------------------------------- | -------------------------------------------- |
| 0   | Neon sign strikes; title decodes | electric hum; digital glitch                 |
| all | Act transitions                  | ★ Whoosh / ★ Whip, alternating               |
| 1   | Shapes snap into place; Polaroid | soft pops; ★ Camera shutter                  |
| 1   | Page turn                        | ★ Page turn                                  |
| 2   | Merge click                      | ★ Mouse click, layered with a deeper thud    |
| 3   | Execute; checks turn green       | ★ Switch; rising chimes                      |
| 4   | Spike hits the peak              | riser into impact                            |
| 5   | Notification slides in           | positive notification ding                   |
| 6   | Deal lands in Won                | cash register + confetti pop                 |
| 8   | Rewind                           | tape rewind                                  |
| 9   | Logo drawn; end card             | hit; ★ Old camera shutter on the final frame |

### Sourcing

The seven built-in sounds cover whoosh, whip, click, switch, page turn and both shutters. Everything else (hum, glitch, chimes, riser, ding, cash register, tape rewind, the music) must be CC0 or clearly royalty-free, with its licence recorded in the project's `assets/` folder. Candidates to verify: Kenney audio packs, Freesound with the CC0 filter, FreePD for music. The user decides the final picks.

## Node usage

| Group                          | Used                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| ------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Browser (all 18)               | including Upload file (logo for the Gitea avatar), Choose option (Grafana time range), Run page script (skip n8n's onboarding popup), Hide clutter, Chapter cards per act, an HTML-overlay "LIVE" badge, Wait for a quiet page, Press keys                                                                                                                                                                                                                                                                                                                           |
| Record                         | Record take per app; Screenshot for the Polaroid and social cards; Sign in once wherever scripting fails                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| Camera (all 10)                | Auto zoom everywhere, Zoom here, Pan, Follow cursor for 9:16, Skip quiet parts, Speed ramp, Crop, laptop and phone Device frames, a different studio backdrop per act, Motion blur                                                                                                                                                                                                                                                                                                                                                                                   |
| Cursor & text                  | cursor size, click ripple, keystroke chips, spotlight, highlight box, tooltip, callout arrow, captions from script shown karaoke-style, lower thirds, animated headlines — all in the project fonts, with per-node fonts where the act calls for it                                                                                                                                                                                                                                                                                                                  |
| Graphics, backgrounds, effects | about 40: matrix decode, typewriter, RGB glitch, neon sign, handwriting, marker highlight, ink underline, scribble circle, confetti, radial burst, terminal, code diff, commit graph, file tree, file upload, checklist, stat card, metric cards, gauge, funnel, bar-chart race, before/after bars, comparison table, pricing card, status cycle, notification stack, device lineup, logo wall, logo reveal, end card, a Lottie checkmark; aurora, god rays, mesh and grain gradients; CRT, VHS, Polaroid, power-off, speed lines; **Mosaic** over tokens and emails |
| Transitions                    | a different one between every act: colour-split slide, page turn, blinds, pixel-grid wipe, whip pan, frosted-glass wipe, zoom blur, shape morph, light-leak cut                                                                                                                                                                                                                                                                                                                                                                                                      |
| Timeline (nodes)               | Sequence, Layers, Trim, Repeat (confetti ×3), Delay, Speed, Fit length, Choose by variable — an `Audience` variable makes two films from the same takes ("devs" keeps the terminal and code acts, "execs" swaps in the charts)                                                                                                                                                                                                                                                                                                                                       |
| Timeline (panel)               | beat-snapped cuts, ripple trims in the outro, split at the playhead for the rewind gag, volume and opacity keyframes, action marks to place sound effects; every edit writes back to the nodes                                                                                                                                                                                                                                                                                                                                                                       |
| Sound & media                  | Music, Sound effect ×12+, Add sound; Trim file on the music; Join files for the montage; Reverse and Change speed for the rewind; Make GIF of the confetti moment; Grab frame for a thumbnail; Resize; Convert to a ProRes master; Web-ready; File info                                                                                                                                                                                                                                                                                                              |
| Mixer (panel)                  | Music / Effects / Clips buses, sidechain ducking, EQ and compressor per strip, master limiter and −14 LUFS target, saved mute/solo states for check renders                                                                                                                                                                                                                                                                                                                                                                                                          |
| Export                         | Export video in 16:9, 9:16, 1:1 and 4:5 (follow the action for tall shapes; mosaic boxes re-checked per shape, since they stay fixed on screen); Export images (wide, link preview, square, portrait, story); Export as code                                                                                                                                                                                                                                                                                                                                         |
| Organise                       | one Group per act, Notes explaining each                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |

## Limits and risks

1. No voice-over; captions carry the narration.
2. Exports will be slow: seven recordings, heavy shaders and effects, four shapes and two audiences ≈ 8 exports of 2½ minutes, likely 30–60 minutes in total, plus the mixer's stem renders. Fast preview keeps editing smooth.
3. Mosaic areas are fixed on the frame and don't follow the camera; every zoom over a token or email must be checked, and in tall shapes the areas need their own placement. Hiding the text inside the page while recording would be safer for anything truly private.
4. Some first-run flows are unconfirmed (Twenty, n8n) — check by hand first.
5. A project this size will probably find bugs in Demoflow, especially in the new Timeline and Mixer write-back; fix them as they show up and report each one.
6. Uptime Kuma has no official REST API for setup; its monitors may need a Demoflow flow of their own, or a one-time manual setup.
7. The downloaded display font travels with the project in `assets/fonts/`; check its licence allows use in videos.

## Order of work

1. Compose file and setup script; check every app by hand on localhost.
2. Project fonts and the downloaded display font.
3. Record each act on its own and get it clean.
4. Assemble acts with transitions and overlays; check card stills give a readable canvas.
5. Music, beat markers and cuts in the Timeline.
6. Sound effects, then the mix in the Mixer; meter the master.
7. Exports in all shapes and both audiences, then a review pass (mosaic placement per shape, loudness, sync).

## Open decisions

1. Keep the app list, or swap any (Penpot for design, Plane for issues)?
2. Only the 2½-minute cut, or a 60-second version too?
3. Music and the non-built-in sound effects: provided by the user, or proposed CC0 picks to approve?
4. Which display font to download for the title?

## Sources

- Twenty self-hosting: https://docs.twenty.com/developers/self-hosting · https://railway.com/deploy/twenty-crm-railway · https://medium.com/@yashp3020/a-comprehensive-guide-to-self-hosting-twenty-crm-with-docker-compose-40ea3fb4afdc
- Plane: https://docs.plane.so/self-hosting/methods/docker-compose · https://ramnode.com/guides/plane
- Grafana in Docker: https://lumadock.com/tutorials/how-to-run-grafana-with-docker-compose
- Excalidraw: https://sliplane.io/blog/self-hosting-excalidraw-the-easy-way · https://www.virtualizationhowto.com/2023/04/excalidraw-whiteboard-ultimate-docker-self-hosted-home-lab-diagramming/
- n8n: https://www.hostinger.com/uk/tutorials/how-to-self-host-n8n-with-docker · https://www.digitalocean.com/community/tutorials/how-to-setup-n8n
- Metabase: https://www.metabase.com/docs/v0.18/operations-guide/running-metabase-on-docker · https://www.mintlify.com/metabase/metabase/installation
- Uptime Kuma: https://oneuptime.com/blog/post/2026-02-08-how-to-run-uptime-kuma-in-docker-for-status-monitoring/markdown
- Penpot: https://help.penpot.app/technical-guide/getting-started/docker/
