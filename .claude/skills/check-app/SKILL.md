---
name: check-app
description: Start the Opsis dev app off screen with its own short data folder, drive it through the Local API and take screenshots, to see a change working in the real app. Use when asked to run, start or screenshot the app, or to confirm a change outside the tests.
---

# Check a change in the running app

The app runs in a virtual X display, never on the real screen, and with a data folder of its own in `~/.cache/opsis-check`, so it never touches the user's projects or settings. Never move the real pointer and never use xdotool.

The data folder must stay short. The Local API listens on `<data folder>/api.sock`, and a socket path over 100 bytes is refused, so the API never starts. A folder under the session scratchpad is already too long; use `~/.cache/opsis-check`.

## 1. Make the data folder

```bash
bun .claude/skills/check-app/seed.ts
```

It deletes and remakes `~/.cache/opsis-check`: a database made from `crates/store/schema.sql` with the Local API on and the dev engine's recording browser (`~/.config/Opsis/engine`, linked as `user-data/engine`) as `settings.engine`, and `examples/` copied to `~/.cache/opsis-check/projects`. It prints the data folder.

## 2. Start the app

Run it as one background command (`run_in_background`), and stop it with TaskStop when done:

```bash
CARGO_BUILD_JOBS=8 OPSIS_USER_DATA=$HOME/.cache/opsis-check/user-data OPSIS_AUDIO_DEVICE=alsa:null env -u WAYLAND_DISPLAY QT_QPA_PLATFORM=xcb xvfb-run -n 99 -f $HOME/.cache/opsis-check/xauth -s "-screen 0 1600x1000x24" bun start > $HOME/.cache/opsis-check/app.log 2>&1
```

- `env -u WAYLAND_DISPLAY QT_QPA_PLATFORM=xcb` keeps Qt off the desktop's Wayland display: Qt picks Wayland whenever `WAYLAND_DISPLAY` is set and ignores `DISPLAY`.
- `OPSIS_AUDIO_DEVICE=alsa:null` keeps the app's sound off the speakers.
- `-n 99` names the display; check `/tmp/.X11-unix/X99` doesn't exist first, and pick another number if it does.
- `bun start` stages FFmpeg when missing and runs the editor (`cargo run -p opsis-ui`).

The app is ready when the socket exists. Wait for it with one background command, not a polling loop:

```bash
S=$HOME/.cache/opsis-check; until [ -S $S/user-data/api.sock ] || rg -q "error\[|panicked|Error:" $S/app.log; do sleep 2; done
```

A first start compiles the app, which takes minutes.

## 3. Drive it

Through the Local API (`/openapi.json` lists every route):

```bash
S=$HOME/.cache/opsis-check
curl -s --unix-socket $S/user-data/api.sock http://localhost/status
curl -s --unix-socket $S/user-data/api.sock -X POST -H 'content-type: application/json' -d "{\"dir\":\"$S/projects/changelog\"}" http://localhost/open
curl -s --unix-socket $S/user-data/api.sock http://localhost/project/problems
```

`/open` shows the project in the main window; `/project/...` edits, saves and runs it as the editor would (`crates/api/src/routes.rs`).

## 4. Look at it

```bash
DISPLAY=:99 XAUTHORITY=$HOME/.cache/opsis-check/xauth import -window root $HOME/.cache/opsis-check/shot.png
```

Then read the PNG. The app's log is `~/.cache/opsis-check/app.log`.

## 5. Stop it

Stop the background task with TaskStop, then check that nothing is left: `command ps -o pid,args -C Xvfb,opsis` should show nothing from this run. Never kill by name with `pkill -f`; the user may have their own copy of the app open.
