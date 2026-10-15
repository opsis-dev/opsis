# "Ship It" — the crazy launch demo

A 2½-minute launch film for a fictional startup, **Shipyard**, that takes one feature from whiteboard sketch to "we're live and the money's in" — inside seven real open-source apps running locally in Docker. Every frame is real UI driven by Demoflow; nothing is mocked in a design tool. The goal is to exercise almost every node type: about 200 nodes in 9 acts.

Status: plan only. Nothing here is built yet.

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

## The film

| #   | Act                          | Length | What happens                                                                                                                                                                                                                                                                           |
| --- | ---------------------------- | ------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 0   | Cold open                    | 8 s    | God-rays shader; "SHIP IT." matrix-decodes, glitches in RGB, TV power-off to black.                                                                                                                                                                                                    |
| 1   | The idea — Excalidraw        | 15 s   | The cursor draws the architecture live with keyboard shortcuts (R, A, T) while keystroke chips show every key; a pan glides across the diagram; ink arrow and scribble circle land on "API"; it freezes into a Polaroid still and leaves with a page-turn transition.                  |
| 2   | The code — Gitea             | 20 s   | Open the pull request; a code-morph element animates the diff; Find element early locates Merge mid-camera-move; spotlight and tooltip on Merge, then click; Check text confirms "Merged" or the take fails before export; lower third "Gitea · code review".                          |
| 3   | The pipeline — n8n           | 18 s   | Drag three nodes onto n8n's canvas, connect them, press Execute; a speed ramp flies through the run; speed lines and a CRT terminal type out fake deploy logs; green checks run down a checklist element; whip-pan out.                                                                |
| 4   | Launch day — Grafana         | 20 s   | Fake a response feeds a perfectly shaped traffic spike (same every take); Freeze time pins the clock to launch morning; zoom on the peak with a callout arrow "uh oh 🚀"; a rolling number counts to 1,000,000 requests; light vs dark theme of the same dashboard joined with a wipe. |
| 5   | Still standing — Uptime Kuma | 12 s   | A second Open browser with Mobile on records the status page as a phone, shown in a phone frame, followed by the camera; a status-cycle element flips "Deploying → Live".                                                                                                              |
| 6   | The deal — Twenty CRM        | 18 s   | Real sign-up on camera (typing with pauses), then drag the deal card to Won; confetti, radial burst, cash-register sound; marker highlight on "$240,000".                                                                                                                              |
| 7   | The money — Metabase         | 12 s   | Scroll to a chart, hover the bars for tooltips, an animated bar chart grows over it; zoom-blur transition.                                                                                                                                                                             |
| 8   | Rewind gag                   | 4 s    | "But wait, how did we get here?" — the whole film plays backwards at 4× (Reverse + Change speed) under a VHS effect, then snaps back.                                                                                                                                                  |
| 9   | Outro                        | 10 s   | Logos assemble, punch-line title cards cut on the beat, "Made with Demoflow" title card, power-off.                                                                                                                                                                                    |

## Audio

There is no voice-over; narration is on-screen captions timed from a script. The Music node lowers the music under sound effects ("Lower under other sounds").

### Music

- One instrumental track of about 2½ minutes, ideally around 120 BPM so cuts can land on the beat.
- **Trim file** to the cut length, **Even out volume** (online-video loudness), **Music** node with a fade in, a long fade out, and "Lower under other sounds" set to sound effects at about 40 %.
- **Fit length** stretches or holds the final timeline to the music, not the other way round.
- The 60-second cut (if wanted) uses a second, shorter track rather than chopping the first.

### Sound-effect cue sheet

Each cue is a **Sound effect** node placed at a time, mixed under the act with **Add sound** (up to three sounds per Add sound node, so busy acts nest two).

| Act | Moment                          | Sound                                      |
| --- | ------------------------------- | ------------------------------------------ |
| 0   | Title decodes                   | digital glitch / data noise                |
| 0   | Power-off to black              | CRT power-off thunk                        |
| all | Every transition                | whoosh (2–3 variants so it doesn't repeat) |
| 1   | Shapes snap into place          | soft pops                                  |
| 2   | Merge click                     | heavy satisfying click                     |
| 3   | Execute; checks turn green      | button press; rising chimes                |
| 4   | Spike hits the peak             | riser into impact                          |
| 5   | Status flips to Live            | positive notification ding                 |
| 6   | Deal lands in Won               | cash register + confetti pop               |
| 8   | Rewind                          | tape rewind                                |
| 9   | Logo assembles; final power-off | hit; CRT power-off                         |

### Sourcing

Demoflow's seven built-in CC0 sounds cover whoosh, whip, click, switch, page turn and shutters. Anything else (cash register, glitch, chimes, tape rewind, the music) must be CC0 or clearly royalty-free, with its licence recorded in the project's `assets/` folder. Candidates to verify: Kenney audio packs, Freesound with the CC0 filter, FreePD for music. The user decides the final picks.

## Node usage

| Group                          | Used                                                                                                                                                                                                                                                                                                                                                                           |
| ------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Browser (all 18)               | including Upload file (logo for the Gitea avatar), Choose option (Grafana time range), Run page script (skip n8n's onboarding popup), Hide clutter, Chapter cards per act, an HTML-overlay "LIVE" badge, Wait for a quiet page, Press keys                                                                                                                                     |
| Record                         | Record take per app; Screenshot for the Polaroid and social cards; Sign in once wherever scripting fails                                                                                                                                                                                                                                                                       |
| Camera (all 10)                | Auto zoom everywhere, Zoom here, Pan, Follow cursor for 9:16, Skip quiet parts, Speed ramp, Crop, laptop and phone Device frames, a different studio backdrop per act, Motion blur                                                                                                                                                                                             |
| Cursor & text                  | cursor size, click ripple, keystroke chips, spotlight, highlight box, tooltip, callout arrow, captions from script shown karaoke-style, lower thirds, animated headlines                                                                                                                                                                                                       |
| Graphics, backgrounds, effects | about 25: matrix decode, typewriter, RGB glitch, marker highlight, ink underline, scribble circle, confetti, radial burst, terminal, code morph, checklist, rolling number, bar chart, status cycle, logo assemble, punch lines, phone and laptop frames; god rays, mesh and grain gradients; CRT, VHS, halftone, camera lens, Polaroid, power-off, speed lines, spring settle |
| Transitions                    | a different one between every act: whip pan, push through, zoom blur, glitch cut, page turn, smoke dissolve, wipe, light-leak cut                                                                                                                                                                                                                                              |
| Timeline                       | Sequence, Layers, Trim, Repeat (confetti ×3), Delay, Speed, Fit length, Choose by variable — an `Audience` variable makes two films from the same takes ("devs" keeps the terminal and code acts, "execs" swaps in the charts)                                                                                                                                                 |
| Sound & media                  | Music, Sound effect ×8+, Add sound; Trim file and Even out volume on the music; Join files for the montage; Reverse and Change speed for the rewind; Make GIF of the confetti moment; Grab frame for a thumbnail; Resize; Convert to a ProRes master; Web-ready; File info                                                                                                     |
| Export                         | Export video in 16:9, 9:16, 1:1 and 4:5 (follow the action for tall shapes); Export images (wide, link preview, square, portrait, story); Export as code                                                                                                                                                                                                                       |
| Organise                       | one Group per act, Notes explaining each                                                                                                                                                                                                                                                                                                                                       |

## Limits and risks

1. No voice-over; captions carry the narration.
2. Exports will be slow: seven recordings, heavy shaders and effects, four shapes and two audiences ≈ 8 exports of 2½ minutes, likely 30–60 minutes in total. Fast preview keeps editing smooth.
3. Some first-run flows are unconfirmed (Twenty, n8n) — check by hand first.
4. A project this size will probably find bugs in Demoflow; fix them as they show up and report each one.
5. Uptime Kuma has no official REST API for setup; its monitors may need a Demoflow flow of their own, or a one-time manual setup.

## Order of work

1. Compose file and setup script; check every app by hand on localhost.
2. Record each act on its own and get it clean.
3. Assemble the timeline, transitions, overlays and sound.
4. Exports, then a review pass.

## Open decisions

1. Keep the app list, or swap any (Penpot for design, Plane for issues)?
2. Only the 2½-minute cut, or a 60-second version too?
3. Music and sound effects: provided by the user, or proposed CC0 picks to approve?

## Sources

- Twenty self-hosting: https://docs.twenty.com/developers/self-hosting · https://railway.com/deploy/twenty-crm-railway · https://medium.com/@yashp3020/a-comprehensive-guide-to-self-hosting-twenty-crm-with-docker-compose-40ea3fb4afdc
- Plane: https://docs.plane.so/self-hosting/methods/docker-compose · https://ramnode.com/guides/plane
- Grafana in Docker: https://lumadock.com/tutorials/how-to-run-grafana-with-docker-compose
- Excalidraw: https://sliplane.io/blog/self-hosting-excalidraw-the-easy-way · https://www.virtualizationhowto.com/2023/04/excalidraw-whiteboard-ultimate-docker-self-hosted-home-lab-diagramming/
- n8n: https://www.hostinger.com/uk/tutorials/how-to-self-host-n8n-with-docker · https://www.digitalocean.com/community/tutorials/how-to-setup-n8n
- Metabase: https://www.metabase.com/docs/v0.18/operations-guide/running-metabase-on-docker · https://www.mintlify.com/metabase/metabase/installation
- Uptime Kuma: https://oneuptime.com/blog/post/2026-02-08-how-to-run-uptime-kuma-in-docker-for-status-monitoring/markdown
- Penpot: https://help.penpot.app/technical-guide/getting-started/docker/
