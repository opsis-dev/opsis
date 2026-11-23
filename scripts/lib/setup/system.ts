import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import path from "node:path";
import { LINUX_LIBRARIES } from "../trixie";

const LINUX_PROGRAMS = [
  "clang",
  "clang++",
  "cmake",
  "make",
  "ninja",
  "pkg-config",
  "git",
  "uvx",
  "python3",
];
const CONTAINER_PROGRAM = "docker";

function hasProgram(name: string): boolean {
  return spawnSync("sh", ["-c", `command -v ${name}`], { stdio: "ignore" }).status === 0;
}

function hasStaticLibcxx(): boolean {
  const found = spawnSync("clang++", ["-stdlib=libc++", "-print-file-name=libc++.a"], {
    encoding: "utf8",
  });
  const file = found.stdout.trim();
  return found.status === 0 && path.isAbsolute(file) && existsSync(file);
}

function hasPackage(name: string): boolean {
  const found = spawnSync("dpkg-query", ["-W", "-f=${Status}", name], { encoding: "utf8" });
  return found.status === 0 && found.stdout.includes("install ok installed");
}

export function checkSystem(container: boolean): void {
  if (process.platform !== "linux") return;
  const needed = container ? [...LINUX_PROGRAMS, CONTAINER_PROGRAM] : LINUX_PROGRAMS;
  const programs = needed.filter((name) => !hasProgram(name));
  const packages = hasProgram("dpkg-query") ? LINUX_LIBRARIES.filter((p) => !hasPackage(p)) : [];
  const libcxx = hasProgram("clang++") && hasStaticLibcxx();
  if (programs.length === 0 && packages.length === 0 && libcxx) return;
  const lines = ["setup: building the app on this machine needs these too:"];
  if (programs.length > 0) lines.push(`  programs: ${programs.join(", ")}`);
  if (packages.length > 0) lines.push(`  sudo apt install ${packages.join(" ")}`);
  if (!libcxx)
    lines.push(
      "  libc++ and libc++abi for clang (on Debian and Ubuntu: libc++-<clang version>-dev libc++abi-<clang version>-dev)",
    );
  console.warn(lines.join("\n"));
}
