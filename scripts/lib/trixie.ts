import { execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { homedir, userInfo } from "node:os";
import path from "node:path";

const IMAGE = "debian:trixie";
const PACKAGES = [
  "build-essential",
  "ca-certificates",
  "cmake",
  "ninja-build",
  "perl",
  "python3",
  "pkg-config",
  "xz-utils",
  "file",
  "libasound2-dev",
  "libdbus-1-dev",
  "libfontconfig1-dev",
  "libfreetype-dev",
  "libwayland-dev",
  "wayland-protocols",
  "libx11-dev",
  "libx11-xcb-dev",
  "libxcb1-dev",
  "libxcb-cursor-dev",
  "libxcb-glx0-dev",
  "libxcb-icccm4-dev",
  "libxcb-image0-dev",
  "libxcb-keysyms1-dev",
  "libxcb-randr0-dev",
  "libxcb-render-util0-dev",
  "libxcb-shape0-dev",
  "libxcb-shm0-dev",
  "libxcb-sync-dev",
  "libxcb-util-dev",
  "libxcb-xfixes0-dev",
  "libxcb-xkb-dev",
  "libxext-dev",
  "libxfixes-dev",
  "libxi-dev",
  "libxkbcommon-dev",
  "libxkbcommon-x11-dev",
  "libxrender-dev",
];

export type TrixieRun = {
  script: string;
  mounts: string[];
  env?: Record<string, string>;
};

function aptCache(): string {
  const dir = path.join(homedir(), ".cache", "opsis-trixie-apt");
  mkdirSync(path.join(dir, "partial"), { recursive: true });
  return dir;
}

export function runInTrixie(run: TrixieRun): void {
  const { uid, gid } = userInfo();
  const setup = [
    "set -e",
    "rm -f /etc/apt/apt.conf.d/docker-clean",
    "apt-get update -qq",
    `DEBIAN_FRONTEND=noninteractive apt-get install -y -qq --no-install-recommends ${PACKAGES.join(" ")} > /dev/null`,
    `chown -R ${uid}:${gid} /var/cache/apt/archives`,
    `exec setpriv --reuid=${uid} --regid=${gid} --clear-groups bash -c "$OPSIS_TRIXIE_SCRIPT"`,
  ].join("\n");
  const env = { HOME: homedir(), ...run.env, OPSIS_TRIXIE_SCRIPT: `set -eu\n${run.script}` };
  const args = [
    "run",
    "--rm",
    "--name",
    `opsis-trixie-${process.pid}`,
    "-v",
    `${aptCache()}:/var/cache/apt/archives`,
    ...run.mounts.flatMap((mount) => ["-v", `${mount}:${mount}`]),
    ...Object.entries(env).flatMap(([name, value]) => ["-e", `${name}=${value}`]),
    IMAGE,
    "bash",
    "-c",
    setup,
  ];
  execFileSync("docker", args, { stdio: "inherit" });
}
