import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import path from "node:path";
import { runInTrixie } from "./trixie";

export const QT_VERSION = "6.12.0";
const BUILD_JOBS = "8";
const COMPLETE = ".opsis-complete";
const SOURCES = [
  {
    module: "qtbase",
    sha256: "a951bd163c7b80fc6b8c88d7668fb56abf91c152373e13c10666763238131307",
    size: 51220268,
  },
  {
    module: "qtsvg",
    sha256: "e4ab39534ec97987b1b9b60ef7f4d3253d5912a69f8c713a01753174a4029331",
    size: 2341876,
  },
];
const CONFIGURE = [
  "-static",
  "-release",
  "-no-icu",
  "-no-opengl",
  "-nomake",
  "examples",
  "-nomake",
  "tests",
  "-no-feature-sql",
  "-qt-zlib",
  "-qt-libpng",
  "-qt-libjpeg",
  "-qt-harfbuzz",
  "-qt-pcre",
  "-system-freetype",
  "-fontconfig",
  "-xcb",
];

function cacheDir(): string {
  return path.join(homedir(), ".cache", "opsis-qt-static");
}

function folder(): string {
  return process.arch === "arm64" ? "gcc_arm64" : "gcc_64";
}

export function staticQtPrefix(): string {
  return path.join(cacheDir(), QT_VERSION, folder());
}

export function staticQtReady(): boolean {
  return existsSync(path.join(staticQtPrefix(), COMPLETE));
}

function tarballName(module: string): string {
  return `${module}-everywhere-src-${QT_VERSION}.tar.xz`;
}

function tarballUrl(module: string): string {
  const minor = QT_VERSION.split(".").slice(0, 2).join(".");
  return `https://download.qt.io/official_releases/qt/${minor}/${QT_VERSION}/submodules/${tarballName(module)}`;
}

function sha256(bytes: Uint8Array): string {
  return createHash("sha256").update(bytes).digest("hex");
}

async function download(source: (typeof SOURCES)[number]): Promise<string> {
  const downloads = path.join(cacheDir(), "downloads");
  mkdirSync(downloads, { recursive: true });
  const tarball = path.join(downloads, tarballName(source.module));
  if (!existsSync(tarball) || readFileSync(tarball).length !== source.size) {
    const response = await fetch(tarballUrl(source.module));
    if (!response.ok)
      throw new Error(`downloading ${tarballUrl(source.module)}: ${response.status}`);
    writeFileSync(tarball, new Uint8Array(await response.arrayBuffer()));
  }
  const bytes = readFileSync(tarball);
  if (bytes.length !== source.size || sha256(bytes) !== source.sha256) {
    rmSync(tarball);
    throw new Error(`${tarball} isn't the pinned ${source.module} ${QT_VERSION}; it was removed`);
  }
  return tarball;
}

function buildScript(tarballs: string[], build: string, prefix: string): string {
  const [qtbase, qtsvg] = tarballs;
  const sources = path.join(build, "src");
  return [
    `rm -rf ${build} ${prefix}`,
    `mkdir -p ${sources} ${build}/qtbase ${build}/qtsvg`,
    `tar -xf ${qtbase} -C ${sources}`,
    `tar -xf ${qtsvg} -C ${sources}`,
    `cd ${build}/qtbase`,
    `${sources}/qtbase-everywhere-src-${QT_VERSION}/configure -prefix ${prefix} ${CONFIGURE.join(" ")} > configure.log`,
    `cmake --build . --parallel ${BUILD_JOBS} > build.log`,
    "cmake --install . > install.log",
    `cd ${build}/qtsvg`,
    `${prefix}/bin/qt-configure-module ${sources}/qtsvg-everywhere-src-${QT_VERSION} > configure.log`,
    `cmake --build . --parallel ${BUILD_JOBS} > build.log`,
    "cmake --install . > install.log",
    `rm -rf ${build}`,
    `touch ${prefix}/${COMPLETE}`,
  ].join("\n");
}

export function buildsNatively(): boolean {
  return process.env.CI === "true";
}

export async function ensureStaticQt(): Promise<string> {
  const prefix = staticQtPrefix();
  if (staticQtReady()) return prefix;
  if (process.platform !== "linux")
    throw new Error("The static Qt is built here only on Linux; macOS and Windows come with CI.");
  const tarballs = await Promise.all(SOURCES.map(download));
  const build = path.join(cacheDir(), "build");
  const script = buildScript(tarballs, build, prefix);
  if (buildsNatively()) execFileSync("bash", ["-c", `set -eu\n${script}`], { stdio: "inherit" });
  else runInTrixie({ script, mounts: [cacheDir()] });
  if (!staticQtReady()) throw new Error(`The static Qt build didn't finish in ${prefix}.`);
  return prefix;
}
