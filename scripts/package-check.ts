import { execFileSync } from "node:child_process";
import { readdirSync } from "node:fs";
import { homedir } from "node:os";
import path from "node:path";

const OUT = path.join(homedir(), ".cache", "opsis-build");
const IMAGE = "debian:trixie";
const APPIMAGE_SYSTEM_PACKAGES = [
  "libasound2t64",
  "libfontconfig1",
  "libfreetype6",
  "libx11-6",
  "libx11-xcb1",
  "libxcb1",
  "libwayland-client0",
];

type Check = { format: string; suffix: string; script: (file: string) => string };

const COMMON = String.raw`
set -eu
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt_install() { apt-get install -y -qq --no-install-recommends "$@" > /tmp/apt.log 2>&1 || { tail -30 /tmp/apt.log; exit 1; }; }
deb_depends() { dpkg-deb -f /pkg/$(cd /pkg && ls *.deb | head -1) Depends | tr ',' '\n' | sed 's/|.*//; s/ //g'; }
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
`;

const CHECKS: Check[] = [
  {
    format: "deb",
    suffix: ".deb",
    script: (file) => String.raw`
apt_install /pkg/${file}
dpkg-query -W -f='installed ${"${Package}"} ${"${Version}"}\n' opsis
test -f /usr/lib/opsis/THIRD_PARTY_NOTICES.md
/usr/bin/opsis-ffmpeg -hide_banner -version | head -1
views /usr/bin/opsis
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
`,
  },
  {
    format: "pacman",
    suffix: ".tar.gz",
    script: (file) => String.raw`
apt_install pacman-package-manager makepkg fakeroot libarchive-tools binutils zstd file
apt_install $(deb_depends)
useradd -m builder
mkdir -p /tmp/build && cp /pkg/PKGBUILD /pkg/${file} /tmp/build/ && chown -R builder /tmp/build
su builder -c "cd /tmp/build && makepkg --nodeps --noconfirm > /tmp/makepkg.log 2>&1" || { tail -30 /tmp/makepkg.log; exit 1; }
built=$(ls /tmp/build/*.pkg.tar.*)
echo "built $(basename $built)"
mkdir -p /tmp/root/var/lib/pacman
pacman -U --root /tmp/root --dbpath /tmp/root/var/lib/pacman --noconfirm --nodeps --nodeps "$built" > /tmp/pacman.log 2>&1 || { tail -30 /tmp/pacman.log; exit 1; }
pacman --root /tmp/root --dbpath /tmp/root/var/lib/pacman -Q opsis
test -f /tmp/root/usr/lib/opsis/THIRD_PARTY_NOTICES.md
views /tmp/root/usr/bin/opsis
`,
  },
];

function packageFile(suffix: string): string | undefined {
  return readdirSync(OUT).find((name) => name.endsWith(suffix));
}

function runCheck(check: Check, file: string): boolean {
  console.log(`\n${check.format}: ${file}`);
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
        IMAGE,
        "bash",
        "-c",
        COMMON + check.script(file),
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
console.log("\nEvery package installed and ran the views scenario.");
