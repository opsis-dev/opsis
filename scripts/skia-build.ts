import { execFileSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, readdirSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { ROOT, devCache } from "./lib/tool-config";

const SKIA_REPO = "https://skia.googlesource.com/skia.git";
const SKIA_BRANCH = "chrome/m157";
const SKIA_COMMIT = "fef60e46c522b87e5802ddf0a215a763dd5e0483";
const BUILD_JOBS = "8";
const IMAGE = "opsis-skia-build";
const DOCKER_FLAG = "--docker";
const ROOT_FLAG = "--into=";

const TARGETS = [
  "skia",
  "modules/skottie",
  "modules/sksg",
  "modules/skshaper",
  "modules/skresources",
];

const CHROME_LINUX_DEFINES = [
  "SK_GAMMA_EXPONENT=1.2",
  "SK_GAMMA_CONTRAST=0.2",
  "SK_USE_FREETYPE_EMBOLDEN",
  "SK_SUPPORT_LEGACY_ANISOTROPIC_MIPMAP_SCALE",
  "SK_SUPPORT_LEGACY_CONIC_CHOP",
  "SK_USE_PADDED_BLUR_UPSCALE",
  "SK_LEGACY_INITWITHPREV_LAYER_SIZING",
  "SK_AVOID_SLOW_RASTER_PIPELINE_BLURS",
  "SK_SUPPORT_LEGACY_RRECT_TRANSFORM",
  "SK_USE_SAFE_INSET_FOR_TEXTURE_SAMPLING",
  "SK_LEGACY_LINEAR_INSET",
];
const UNSET_DEFINES = ["SK_GAMMA_APPLY_TO_A8"];

const CFLAGS = [
  "-fPIC",
  "-ffunction-sections",
  "-fdata-sections",
  ...CHROME_LINUX_DEFINES.map((d) => `-D${d}`),
  ...UNSET_DEFINES.map((d) => `-U${d}`),
];

const ARGS: Record<string, string | boolean> = {
  is_official_build: true,
  is_debug: false,
  is_component_build: false,
  cc: '"clang"',
  cxx: '"clang++"',
  extra_cflags: JSON.stringify(CFLAGS),
  skia_enable_ganesh: false,
  skia_enable_graphite: false,
  skia_use_gl: false,
  skia_use_vulkan: false,
  skia_enable_pdf: false,
  skia_enable_svg: false,
  skia_use_expat: false,
  skia_use_fontconfig: false,
  skia_use_freetype: true,
  skia_use_system_freetype2: false,
  skia_enable_fontmgr_custom_empty: true,
  skia_enable_fontmgr_custom_directory: false,
  skia_enable_fontmgr_custom_embedded: false,
  skia_enable_fontmgr_empty: false,
  skia_use_fontations: false,
  skia_use_harfbuzz: false,
  skia_use_icu: false,
  skia_use_client_icu: false,
  skia_use_libgrapheme: false,
  skia_use_bidi: false,
  skia_enable_skottie: true,
  skia_enable_skshaper: true,
  skia_enable_skunicode: false,
  skia_enable_skparagraph: false,
  skia_use_zlib: true,
  skia_use_system_zlib: false,
  skia_use_libpng_decode: true,
  skia_use_libpng_encode: true,
  skia_use_system_libpng: false,
  skia_use_libjpeg_turbo_decode: true,
  skia_use_libjpeg_turbo_encode: false,
  skia_use_system_libjpeg_turbo: false,
  skia_use_libwebp_decode: true,
  skia_use_libwebp_encode: false,
  skia_use_system_libwebp: false,
  skia_use_wuffs: false,
  skia_use_dng_sdk: false,
  skia_use_piex: false,
  skia_use_xps: false,
  skia_use_libavif: false,
  skia_use_libjxl_decode: false,
  skia_use_rust_png_decode: false,
  skia_use_rust_png_encode: false,
  skia_enable_tools: false,
  skia_use_partition_alloc: false,
  use_partition_alloc_as_malloc: false,
  skia_use_perfetto: false,
  skia_enable_precompile: false,
};

const CONSUMER_TARGET = "//modules/skottie:skottie";
const OWN_DEFINE = /_IMPLEMENTATION(=|$)/;

function run(command: string, args: string[], cwd: string, env: NodeJS.ProcessEnv = process.env) {
  execFileSync(command, args, { cwd, stdio: "inherit", env });
}

function platformName(): string {
  return `${process.platform}-${process.arch}`;
}

function skiaRoot(): string {
  const given = process.argv.find((a) => a.startsWith(ROOT_FLAG))?.slice(ROOT_FLAG.length);
  return given ?? devCache("skia");
}

function skiaDir(): string {
  return path.join(skiaRoot(), SKIA_COMMIT);
}

function fetchSource(src: string) {
  if (existsSync(path.join(src, "BUILD.gn"))) {
    console.log(`Skia: ${src} is already fetched`);
    return;
  }
  mkdirSync(src, { recursive: true });
  run("git", ["init", "-q"], src);
  run("git", ["remote", "add", "origin", SKIA_REPO], src);
  run("git", ["fetch", "--depth", "1", "origin", SKIA_COMMIT], src);
  run("git", ["checkout", "-q", "FETCH_HEAD"], src);
  console.log(`Skia: ${SKIA_BRANCH} at ${SKIA_COMMIT}`);
}

const SYNC_ATTEMPTS = 5;
const SYNC_PAUSE_SECONDS = 30;

function syncDependencies(src: string) {
  const env = { ...process.env, GIT_SYNC_DEPS_SKIP_EMSDK: "1" };
  for (let attempt = 1; ; attempt++) {
    try {
      run("python3", ["tools/git-sync-deps"], src, env);
      run("python3", ["bin/fetch-ninja"], src, env);
      return;
    } catch (error) {
      if (attempt >= SYNC_ATTEMPTS) throw error;
      const pause = SYNC_PAUSE_SECONDS * attempt;
      console.log(`Skia: fetching its dependencies failed, trying again in ${pause} s`);
      execFileSync("sleep", [String(pause)]);
    }
  }
}

function gnArgs(): string {
  return Object.entries(ARGS)
    .map(([key, value]) => `${key}=${String(value)}`)
    .join(" ");
}

function build(src: string, out: string) {
  const gn = path.join(src, "bin", process.platform === "win32" ? "gn.exe" : "gn");
  run(gn, ["gen", out, `--args=${gnArgs()}`], src);
  const ninja = path.join(
    src,
    "third_party",
    "ninja",
    process.platform === "win32" ? "ninja.exe" : "ninja",
  );
  run(existsSync(ninja) ? ninja : "ninja", ["-C", out, "-j", BUILD_JOBS, ...TARGETS], src);
  writeConsumer(gn, src, out);
}

function described(gn: string, src: string, out: string, what: string): string[] {
  const text = execFileSync(gn, ["desc", out, CONSUMER_TARGET, what, "--format=json"], {
    cwd: src,
    encoding: "utf8",
  });
  const parsed = JSON.parse(text.slice(text.indexOf("{"))) as Record<
    string,
    Record<string, string[]>
  >;
  return Object.values(parsed)[0]?.[what] ?? [];
}

function resolved(src: string, dir: string): string {
  return dir.startsWith("//") ? path.join(src, dir.slice(2)) : dir;
}

function writeConsumer(gn: string, src: string, out: string) {
  const defines = [
    ...described(gn, src, out, "defines").filter(
      (d) => !OWN_DEFINE.test(d) && !UNSET_DEFINES.includes(d),
    ),
    ...CHROME_LINUX_DEFINES,
  ];
  const includes = described(gn, src, out, "include_dirs").map((d) => resolved(src, d));
  const libraries = readdirSync(out)
    .filter((f) => f.startsWith("lib") && f.endsWith(".a"))
    .map((f) => f.slice("lib".length, -".a".length))
    .toSorted();
  writeFileSync(
    path.join(out, "consumer.json"),
    `${JSON.stringify({ commit: SKIA_COMMIT, defines, includes, libraries }, null, 2)}\n`,
  );
}

function bunProgram(): string {
  return execFileSync("which", ["bun"], { encoding: "utf8" }).trim();
}

function buildInDocker() {
  const context = path.join(devCache("skia"), "image");
  mkdirSync(context, { recursive: true });
  copyFileSync(
    path.join(ROOT, "scripts", "lib", "skia", "Dockerfile"),
    path.join(context, "Dockerfile"),
  );
  run("docker", ["build", "-t", IMAGE, context], ROOT);
  mkdirSync(skiaDir(), { recursive: true });
  const user = `${os.userInfo().uid}:${os.userInfo().gid}`;
  run(
    "docker",
    [
      "run",
      "--rm",
      "--init",
      "--user",
      user,
      "--cpus",
      BUILD_JOBS,
      "-e",
      "HOME=/tmp",
      "-v",
      `${ROOT}:${ROOT}:ro`,
      "-v",
      `${skiaDir()}:${skiaDir()}`,
      "-v",
      `${bunProgram()}:/usr/local/bin/bun:ro`,
      "-w",
      ROOT,
      IMAGE,
      "bun",
      "scripts/skia-build.ts",
      `${ROOT_FLAG}${skiaRoot()}`,
    ],
    ROOT,
  );
}

const args = process.argv.slice(2);
if (args.includes(DOCKER_FLAG)) {
  buildInDocker();
} else {
  const src = path.join(skiaDir(), "src");
  const out = path.join(skiaDir(), "out", platformName());
  fetchSource(src);
  syncDependencies(src);
  build(src, out);
  console.log(`Skia: built into ${out}`);
}
