import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { copyFileSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { fetchFilament, filamentDir, readFilamentPins } from "./lib/filament";
import { ROOT, devCache } from "./lib/tool-config";

const IMAGE = "opsis-filament-build";
const DOCKER_FLAG = "--docker";
const ROOT_FLAG = "--into=";
const BUILD_JOBS = "8";
const SOURCES_DIR = path.join(ROOT, "crates", "filament", "cpp");
const LIBRARIES = [
  "gltfio_core",
  "uberarchive",
  "uberzlib",
  "dracodec",
  "meshoptimizer",
  "basis_transcoder",
  "stb",
  "mikktspace",
  "filament",
  "backend",
  "bluevk",
  "bluegl",
  "webgpu_dawn",
  "filabridge",
  "filaflat",
  "utils",
  "geometry",
  "ktxreader",
  "image",
  "ibl-lite",
  "shaders",
  "smol-v",
  "abseil",
  "perfetto",
  "zstd",
];
const LIBRARY_ARCH: Record<string, string> = { x64: "x86_64", arm64: "aarch64" };
const PERSONALITY_REF = "DW.ref.__gxx_personality_v0";

function run(command: string, args: string[], cwd: string) {
  execFileSync(command, args, { cwd, stdio: "inherit" });
}

function cacheRoot(): string {
  return (
    process.argv.find((a) => a.startsWith(ROOT_FLAG))?.slice(ROOT_FLAG.length) ??
    devCache("filament")
  );
}

function sources(): string[] {
  return readdirSync(SOURCES_DIR)
    .filter((f) => f.endsWith(".cc") || f.endsWith(".h"))
    .toSorted();
}

function sourcesHash(): string {
  const hash = createHash("sha256");
  for (const file of sources()) {
    hash.update(file);
    hash.update(readFileSync(path.join(SOURCES_DIR, file)));
  }
  return hash.digest("hex");
}

function cxxLibraryDir(): string {
  const file = execFileSync("clang++", ["-stdlib=libc++", "-print-file-name=libc++.a"], {
    encoding: "utf8",
  }).trim();
  return path.dirname(file);
}

function build(filament: string, out: string, version: string) {
  rmSync(out, { recursive: true, force: true });
  mkdirSync(out, { recursive: true });
  const objects = sources()
    .filter((f) => f.endsWith(".cc"))
    .map((file) => {
      const object = path.join(out, `${path.basename(file, ".cc")}.o`);
      run(
        "clang++",
        [
          "-std=c++20",
          "-stdlib=libc++",
          "-fno-rtti",
          "-fPIC",
          "-O2",
          `-I${path.join(filament, "include")}`,
          "-c",
          path.join(SOURCES_DIR, file),
          "-o",
          object,
        ],
        out,
      );
      return object;
    });
  const libraries = path.join(filament, "lib", LIBRARY_ARCH[process.arch] ?? process.arch);
  const cxx = cxxLibraryDir();
  const joined = path.join(out, "joined.o");
  run(
    "ld",
    [
      "-r",
      "-o",
      joined,
      ...objects,
      "--start-group",
      ...LIBRARIES.map((name) => path.join(libraries, `lib${name}.a`)),
      path.join(cxx, "libc++.a"),
      path.join(cxx, "libc++abi.a"),
      "--end-group",
    ],
    out,
  );
  const sealed = path.join(out, "sealed.o");
  run(
    "objcopy",
    [
      "--wildcard",
      "--keep-global-symbol=opsis_*",
      `--redefine-sym=${PERSONALITY_REF}=opsis.${PERSONALITY_REF}`,
      joined,
      sealed,
    ],
    out,
  );
  run("ar", ["rcs", path.join(out, "libopsis_filament.a"), sealed], out);
  rmSync(joined);
  rmSync(sealed);
  writeFileSync(
    path.join(out, "build.json"),
    `${JSON.stringify({ version, sources: sourcesHash(), links: ["X11", "pthread", "dl", "m"] }, null, 2)}\n`,
  );
}

function buildInDocker(root: string) {
  const context = path.join(root, "image");
  mkdirSync(context, { recursive: true });
  copyFileSync(
    path.join(ROOT, "scripts", "lib", "filament", "Dockerfile"),
    path.join(context, "Dockerfile"),
  );
  run("docker", ["build", "-q", "-t", IMAGE, context], ROOT);
  const bun = execFileSync("which", ["bun"], { encoding: "utf8" }).trim();
  run(
    "docker",
    [
      "run",
      "--rm",
      "--init",
      "--user",
      `${os.userInfo().uid}:${os.userInfo().gid}`,
      "--cpus",
      BUILD_JOBS,
      "--network",
      "none",
      "-e",
      "HOME=/tmp",
      "-v",
      `${ROOT}:${ROOT}:ro`,
      "-v",
      `${root}:${root}`,
      "-v",
      `${bun}:/usr/local/bin/bun:ro`,
      "-w",
      ROOT,
      IMAGE,
      "bun",
      "scripts/filament-build.ts",
      `${ROOT_FLAG}${root}`,
    ],
    ROOT,
  );
}

const pins = readFilamentPins();
const root = cacheRoot();
const filament = fetchFilament(pins, root);
if (process.argv.includes(DOCKER_FLAG)) {
  buildInDocker(root);
} else {
  const out = path.join(filamentDir(root, pins.version), "build");
  build(filament, out, pins.version);
  console.log(`Filament: sealed into ${path.join(out, "libopsis_filament.a")}`);
}
