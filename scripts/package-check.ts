import { execFileSync } from "node:child_process";
import { cpSync, mkdirSync, readdirSync, rmSync } from "node:fs";
import { userInfo } from "node:os";
import path from "node:path";
import { devCache, ROOT } from "./lib/tool-config";
import { DEV_ENGINE, seedUserData } from "./lib/user-data";

const OUT = devCache("build");
const IMAGE = "debian:trixie";
const WORK = devCache("package-check");
const EXAMPLE = "changelog";
const BROWSER_PACKAGES = [
  "libnss3",
  "libnspr4",
  "libatk1.0-0t64",
  "libatk-bridge2.0-0t64",
  "libatspi2.0-0t64",
  "libcups2t64",
  "libdrm2",
  "libgbm1",
  "libglib2.0-0t64",
  "libpango-1.0-0",
  "libcairo2",
  "libexpat1",
  "libxcomposite1",
  "libxdamage1",
  "libxext6",
  "libxfixes3",
  "libxrandr2",
  "fonts-liberation",
];
const APPIMAGE_SYSTEM_PACKAGES = [
  "libasound2t64",
  "libfontconfig1",
  "libfreetype6",
  "libx11-6",
  "libx11-xcb1",
  "libxcb1",
  "libwayland-client0",
];

type Check = {
  format: string;
  suffix: string;
  script: (file: string) => string;
  exports?: boolean;
};

const COMMON = String.raw`
set -eu
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt_install() { apt-get install -y -qq --no-install-recommends "$@" > /tmp/apt.log 2>&1 || { tail -30 /tmp/apt.log; exit 1; }; }
deb_depends() { dpkg-deb -f /pkg/$(cd /pkg && ls *.deb | head -1) Depends | tr ',' '\n' | sed 's/|.*//; s/ //g'; }
export_film() {
  apt_install curl __BROWSER__
  data=$OPSIS_CHECK_WORK/user-data
  sock=$data/api.sock
  project=$OPSIS_CHECK_WORK/projects/__EXAMPLE__
  mkdir -p /tmp/apphome && chown "$OPSIS_CHECK_UID:$OPSIS_CHECK_GID" /tmp/apphome
  setpriv --reuid="$OPSIS_CHECK_UID" --regid="$OPSIS_CHECK_GID" --clear-groups \
    env HOME=/tmp/apphome OPSIS_USER_DATA="$data" QT_QPA_PLATFORM=offscreen OPSIS_AUDIO_DEVICE=alsa:null \
    "$1" > /tmp/app.log 2>&1 &
  app=$!
  for _ in $(seq 120); do [ -S "$sock" ] && break; sleep 1; done
  [ -S "$sock" ] || { echo "export FAILED: the Local API never started"; tail -20 /tmp/app.log; exit 1; }
  api() { curl -sS --unix-socket "$sock" -H 'content-type: application/json' "$@"; }
  api -X POST -d "{\"dir\":\"$project\"}" http://localhost/open > /tmp/open.json
  for _ in $(seq 60); do api http://localhost/project | grep -q '"nodes"' && break; sleep 1; done
  curl -sN --unix-socket "$sock" http://localhost/project/run/events > /tmp/events.log &
  sleep 1
  api -X POST -d '{}' http://localhost/project/run > /tmp/started.json
  for _ in $(seq 900); do grep -q '"type":"run:done"' /tmp/events.log && break; sleep 1; done
  done=$(grep -m1 '"type":"run:done"' /tmp/events.log || echo "no run:done")
  echo "$done" | grep -q '"ok":true' || { echo "export FAILED: $done"; cat /tmp/open.json /tmp/started.json; grep -i -E "error|fail" /tmp/events.log | tail -5; tail -30 /tmp/app.log; exit 1; }
  video=$(find "$project" -name '*.mp4' | head -1)
  [ -n "$video" ] || { echo "export FAILED: no video in $project"; exit 1; }
  echo "exported $(basename "$video"), $(stat -c %s "$video") bytes: $(/usr/bin/opsis-ffmpeg -hide_banner -i "$video" 2>&1 | grep -E 'Duration|Video:' | sed 's/^ *//' | tr '\n' ' ')"
  kill "$app"; wait "$app" 2>/dev/null || true
}
views() {
  rm -rf /tmp/run && mkdir -p /tmp/run/projects
  if OPSIS_USER_DATA=/tmp/run/user-data OPSIS_QT_SCENARIO=views OPSIS_QT_PROJECTS=/tmp/run/projects \
    OPSIS_QT_SHOTS=off OPSIS_AUDIO_DEVICE=alsa:null QT_QPA_PLATFORM=offscreen \
    timeout 180 "$@" > /tmp/run/views.log 2>&1 && grep -q "scenario views done" /tmp/run/views.log \
    && ! grep -q " FAILED " /tmp/run/views.log; then
    echo "views: $(grep "scenario views done" /tmp/run/views.log | sed 's/.*scenario views done, //')"
  else
    echo "views FAILED"; tail -20 /tmp/run/views.log; exit 1
  fi
}
installed_from() {
  grep -q "installed from $1" /tmp/run/views.log \
    && echo "the program knows it came from the $1" \
    || { echo "installer FAILED: the program doesn't know it came from the $1"; grep -m1 "installed from\|not installed" /tmp/run/views.log; exit 1; }
}
`;

const CHECKS: Check[] = [
  {
    format: "deb",
    suffix: ".deb",
    exports: true,
    script: (file) => String.raw`
apt_install /pkg/${file}
dpkg-query -W -f='installed ${"${Package}"} ${"${Version}"}\n' opsis
test -f /usr/lib/opsis/THIRD_PARTY_NOTICES.md
/usr/bin/opsis-ffmpeg -hide_banner -version | head -1
views /usr/bin/opsis
installed_from "deb package"
export_film /usr/bin/opsis
dpkg -r --force-depends ca-certificates > /dev/null 2>&1
rm -rf /etc/ssl/certs/*
echo "without certificates:"
views /usr/bin/opsis
`,
  },
  {
    format: "rpm",
    suffix: ".rpm",
    script: (file) => String.raw`
apt_install rpm
rpm -K --nosignature /pkg/${file}
apt_install $(deb_depends)
rpm --root /tmp/root --initdb
rpm --root /tmp/root --nodeps -i /pkg/${file} 2>&1 | grep -v "private mounts" || true
rpm --root /tmp/root -q opsis
test -f /tmp/root/usr/lib/opsis/THIRD_PARTY_NOTICES.md
views /tmp/root/usr/bin/opsis
installed_from "rpm package"
`,
  },
  {
    format: "appimage",
    suffix: ".AppImage",
    script: (file) => String.raw`
apt_install ${APPIMAGE_SYSTEM_PACKAGES.join(" ")}
cp /pkg/${file} /tmp/opsis.AppImage
export APPIMAGE_EXTRACT_AND_RUN=1
views /tmp/opsis.AppImage
installed_from "AppImage"
`,
  },
  {
    format: "pacman",
    suffix: ".pkg.tar.zst",
    script: (file) => String.raw`
apt_install pacman-package-manager zstd
apt_install $(deb_depends)
mkdir -p /tmp/root/var/lib/pacman
pacman -U --root /tmp/root --dbpath /tmp/root/var/lib/pacman --noconfirm --nodeps --nodeps /pkg/${file} > /tmp/pacman.log 2>&1 || { tail -30 /tmp/pacman.log; exit 1; }
pacman --root /tmp/root --dbpath /tmp/root/var/lib/pacman -Q opsis
test -f /tmp/root/usr/lib/opsis/THIRD_PARTY_NOTICES.md
views /tmp/root/usr/bin/opsis
installed_from "pacman package"
`,
  },
];

function packageFile(suffix: string): string | undefined {
  return readdirSync(OUT).find((name) => name.endsWith(suffix));
}

function prepareExport(): string[] {
  rmSync(WORK, { recursive: true, force: true });
  seedUserData(path.join(WORK, "user-data"));
  mkdirSync(path.join(WORK, "projects"), { recursive: true });
  cpSync(path.join(ROOT, "examples", EXAMPLE), path.join(WORK, "projects", EXAMPLE), {
    recursive: true,
  });
  const { uid, gid } = userInfo();
  return [
    "-v",
    `${WORK}:${WORK}`,
    "-v",
    `${DEV_ENGINE}:${DEV_ENGINE}:ro`,
    "-e",
    `OPSIS_CHECK_WORK=${WORK}`,
    "-e",
    `OPSIS_CHECK_UID=${uid}`,
    "-e",
    `OPSIS_CHECK_GID=${gid}`,
  ];
}

function runCheck(check: Check, file: string): boolean {
  console.log(`\n${check.format}: ${file}`);
  const extra = check.exports ? prepareExport() : [];
  try {
    execFileSync(
      "docker",
      [
        "run",
        "--rm",
        "--name",
        `opsis-package-check-${check.format}-${process.pid}`,
        "-v",
        `${OUT}:/pkg:ro`,
        ...extra,
        IMAGE,
        "bash",
        "-c",
        COMMON.replace("__BROWSER__", BROWSER_PACKAGES.join(" ")).replace("__EXAMPLE__", EXAMPLE) +
          check.script(file),
      ],
      { stdio: "inherit" },
    );
    return true;
  } catch {
    return false;
  }
}

const wanted = process.argv.slice(2);
const failed: string[] = [];
for (const check of CHECKS) {
  if (wanted.length > 0 && !wanted.includes(check.format)) continue;
  const file = packageFile(check.suffix);
  if (!file) {
    console.log(
      `\n${check.format}: no package in ${OUT}; build it with bun run make -- --formats ${check.format}`,
    );
    failed.push(check.format);
    continue;
  }
  if (!runCheck(check, file)) failed.push(check.format);
}
if (failed.length > 0) {
  console.error(`\nFailed: ${failed.join(", ")}`);
  process.exit(1);
}
console.log("\nEvery package installed and ran the views scenario; the deb exported a film.");
