import { execFileSync } from "node:child_process";
import { existsSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fetchGitSource } from "./git-source";
import { readPins } from "./setup/manifest";
import { visualStudioFolder } from "./static-qt";
import { ROOT } from "./tool-config";
import { runInTrixie } from "./trixie";

const BUILD_JOBS = "8";
const NM_BUFFER = 64 * 1024 * 1024;
const PRIVATE_PREFIX = "opsis_skia_";
const PRIVATE_LIBRARIES = ["jpeg", "jpeg12", "jpeg16", "png", "freetype2", "webp", "webp_sse41"];
const INSIDE_FLAG = "--inside";

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

const SAME_ON_EVERY_X86 = ["SK_DISABLE_AVX512_OPTS"];

const DEFINE_FLAGS = [
  ...CHROME_LINUX_DEFINES.map((d) => `-D${d}`),
  ...SAME_ON_EVERY_X86.map((d) => `-D${d}`),
  ...UNSET_DEFINES.map((d) => `-U${d}`),
];
const UNIX_CFLAGS = ["-fPIC", "-ffunction-sections", "-fdata-sections"];
const LINUX_CXXFLAGS = ["-stdlib=libc++"];
const WINDOWS_CFLAGS = ["/MD"];

const ARGS: Record<string, string | boolean> = {
  is_official_build: true,
  is_debug: false,
  is_component_build: false,
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

function llvmFolder(): string {
  const llvm = path.win32.join(visualStudioFolder(), "VC", "Tools", "Llvm", "x64");
  if (!existsSync(path.win32.join(llvm, "bin", "clang-cl.exe")))
    throw new Error(
      `Building Skia on Windows needs Visual Studio's Clang tools; ${llvm} has no clang-cl.`,
    );
  return llvm.replaceAll("\\", "/");
}

export function compilerArgs(platform: SkiaPlatform, llvm = ""): Record<string, string> {
  if (platform === "win32")
    return {
      clang_win: JSON.stringify(llvm),
      extra_cflags: JSON.stringify([...WINDOWS_CFLAGS, ...DEFINE_FLAGS]),
      extra_cflags_cc: JSON.stringify([]),
    };
  return {
    cc: JSON.stringify("clang"),
    cxx: JSON.stringify("clang++"),
    extra_cflags: JSON.stringify([...UNIX_CFLAGS, ...DEFINE_FLAGS]),
    extra_cflags_cc: JSON.stringify(platform === "linux" ? LINUX_CXXFLAGS : []),
  };
}

const BUILD_ONLY_SOURCES = [path.join("third_party", "externals"), ".git"];
const REMOVE_ATTEMPTS = 5;

const SYNC_ATTEMPTS = 5;
const SYNC_PAUSE_SECONDS = 30;

function syncDependencies(src: string) {
  const env = {
    ...process.env,
    GIT_SYNC_DEPS_SKIP_EMSDK: "1",
    GIT_CONFIG_COUNT: "1",
    GIT_CONFIG_KEY_0: "core.longpaths",
    GIT_CONFIG_VALUE_0: "true",
  };
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

function gnArgs(platform: SkiaPlatform): string {
  const compiler = compilerArgs(platform, platform === "win32" ? llvmFolder() : "");
  return Object.entries({ ...ARGS, ...compiler })
    .map(([key, value]) => `${key}=${String(value)}`)
    .join(" ");
}

function build(src: string, out: string, commit: string) {
  const platform = hostPlatform();
  const gn = path.join(src, "bin", platform === "win32" ? "gn.exe" : "gn");
  run(gn, ["gen", out, `--args=${gnArgs(platform)}`], src);
  const ninja = path.join(
    src,
    "third_party",
    "ninja",
    platform === "win32" ? "ninja.exe" : "ninja",
  );
  run(existsSync(ninja) ? ninja : "ninja", ["-C", out, "-j", BUILD_JOBS, ...TARGETS], src);
  makePrivate(out, platform);
  writeConsumer(gn, src, out, commit, platform);
}

export type SkiaPlatform = "linux" | "darwin" | "win32";

function hostPlatform(): SkiaPlatform {
  const platform = process.platform;
  if (platform === "linux" || platform === "darwin" || platform === "win32") return platform;
  throw new Error(`Skia isn't built for ${platform}.`);
}

export function archiveFile(name: string, platform: SkiaPlatform): string {
  return platform === "win32" ? `${name}.lib` : `lib${name}.a`;
}

export function archiveName(file: string, platform: SkiaPlatform): string | null {
  if (platform === "win32") return file.endsWith(".lib") ? file.slice(0, -".lib".length) : null;
  return file.startsWith("lib") && file.endsWith(".a")
    ? file.slice("lib".length, -".a".length)
    : null;
}

export function privateSymbol(symbol: string, platform: SkiaPlatform): string {
  if (platform === "darwin" && symbol.startsWith("_"))
    return `_${PRIVATE_PREFIX}${symbol.slice(1)}`;
  return `${PRIVATE_PREFIX}${symbol}`;
}

function rustLlvmTools(): string | null {
  try {
    const sysroot = execFileSync("rustc", ["--print", "sysroot"], { cwd: ROOT, encoding: "utf8" });
    const version = execFileSync("rustc", ["-vV"], { cwd: ROOT, encoding: "utf8" });
    const host = /^host: (.+)$/m.exec(version)?.[1];
    return host ? path.join(sysroot.trim(), "lib", "rustlib", host, "bin") : null;
  } catch {
    return null;
  }
}

function llvmTool(name: string): string {
  const program = process.platform === "win32" ? `${name}.exe` : name;
  const tools = rustLlvmTools();
  const bundled = tools ? path.join(tools, program) : null;
  return bundled && existsSync(bundled) ? bundled : program;
}

function definedSymbols(nm: string, file: string): string[] {
  const listed = execFileSync(nm, ["-g", "--defined-only", "--format=just-symbols", file], {
    encoding: "utf8",
    maxBuffer: NM_BUFFER,
  });
  return listed
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line !== "" && !line.endsWith(":"));
}

function makePrivate(out: string, platform: SkiaPlatform) {
  const nm = llvmTool("llvm-nm");
  const objcopy = llvmTool("llvm-objcopy");
  const symbols = new Set(
    PRIVATE_LIBRARIES.map((name) => path.join(out, archiveFile(name, platform)))
      .filter((file) => existsSync(file))
      .flatMap((file) => definedSymbols(nm, file)),
  );
  const renames = path.join(out, "private-symbols.txt");
  writeFileSync(
    renames,
    [...symbols].map((symbol) => `${symbol} ${privateSymbol(symbol, platform)}\n`).join(""),
  );
  for (const file of readdirSync(out).filter((f) => archiveName(f, platform) !== null))
    run(objcopy, [`--redefine-syms=${renames}`, path.join(out, file)], out);
  console.log(`Skia: ${symbols.size} symbols of its own libraries made private`);
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

function writeConsumer(
  gn: string,
  src: string,
  out: string,
  commit: string,
  platform: SkiaPlatform,
) {
  const defines = [
    ...described(gn, src, out, "defines").filter(
      (d) => !OWN_DEFINE.test(d) && !UNSET_DEFINES.includes(d),
    ),
    ...CHROME_LINUX_DEFINES,
  ];
  const includes = described(gn, src, out, "include_dirs").map((d) => resolved(src, d));
  const libraries = readdirSync(out)
    .map((f) => archiveName(f, platform))
    .filter((name) => name !== null)
    .toSorted();
  writeFileSync(
    path.join(out, "consumer.json"),
    `${JSON.stringify({ commit, defines, includes, libraries }, null, 2)}\n`,
  );
}

export const SKIA_RECIPE = {
  args: ARGS,
  defines: DEFINE_FLAGS,
  unixCflags: UNIX_CFLAGS,
  linuxCxxflags: LINUX_CXXFLAGS,
  windowsCflags: WINDOWS_CFLAGS,
  targets: TARGETS,
  private: { prefix: PRIVATE_PREFIX, libraries: PRIVATE_LIBRARIES },
};

function buildHere(dir: string) {
  const { skia } = readPins();
  const src = path.join(dir, "src");
  const out = path.join(dir, "out");
  fetchGitSource(src, path.join(src, "BUILD.gn"), skia, "Skia");
  syncDependencies(src);
  build(src, out, skia.commit);
  dropBuildOnlySources(src);
  console.log(`Skia: built into ${out}`);
}

function dropBuildOnlySources(src: string) {
  for (const part of BUILD_ONLY_SOURCES) {
    rmSync(path.join(src, part), { recursive: true, force: true, maxRetries: REMOVE_ATTEMPTS });
  }
}

function bunProgram(): string {
  return execFileSync("which", ["bun"], { encoding: "utf8" }).trim();
}

export function buildSkia(dir: string, container: boolean): void {
  if (!container) {
    buildHere(dir);
    return;
  }
  const bun = bunProgram();
  runInTrixie({
    script: `cd ${ROOT}\n${bun} scripts/lib/skia.ts ${INSIDE_FLAG} ${dir}`,
    mounts: [dir, ROOT, path.dirname(bun)],
  });
}

const inside = process.argv.indexOf(INSIDE_FLAG);
if (import.meta.main && inside >= 0) {
  const dir = process.argv[inside + 1];
  if (!dir) throw new Error(`${INSIDE_FLAG} needs the folder to build into.`);
  buildHere(dir);
}
