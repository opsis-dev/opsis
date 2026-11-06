import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, renameSync, rmSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import path from "node:path";

const QT_VERSION = "6.12.0";
const AQT = "aqtinstall==3.3.0";
const KDDW = {
  version: "2.5.1",
  url: "https://github.com/KDAB/KDDockWidgets/releases/download/v2.5.1/kddockwidgets-2.5.1.tar.gz",
  sha256: "d86c86c7a186a16400e08f78da2c6d0ac792d45822ced5cba29e3537edee952a",
  size: 4492656,
};

type QtTarget = { host: string; arch: string; folder: string };

function cacheDir(): string {
  return path.join(homedir(), ".cache", "opsis-qt");
}

function qtTarget(): QtTarget {
  if (process.platform === "darwin") return { host: "mac", arch: "clang_64", folder: "macos" };
  if (process.platform === "win32") {
    return { host: "windows", arch: "win64_msvc2022_64", folder: "msvc2022_64" };
  }
  if (process.arch === "arm64") {
    return { host: "linux_arm64", arch: "linux_gcc_arm64", folder: "gcc_arm64" };
  }
  return { host: "linux", arch: "linux_gcc_64", folder: "gcc_64" };
}

function installQt(): string {
  const given = process.env.OPSIS_QT;
  if (given && existsSync(path.join(given, "lib"))) {
    console.log(`Qt: using ${given} (OPSIS_QT)`);
    return given;
  }
  const target = qtTarget();
  const prefix = path.join(cacheDir(), QT_VERSION, target.folder);
  if (existsSync(path.join(prefix, "bin"))) {
    console.log(`Qt: ${prefix} is already installed`);
    return prefix;
  }
  execFileSync(
    "uvx",
    [
      "--from",
      AQT,
      "aqt",
      "install-qt",
      target.host,
      "desktop",
      QT_VERSION,
      target.arch,
      "-O",
      cacheDir(),
    ],
    { stdio: "inherit" },
  );
  return prefix;
}

function sha256(bytes: Uint8Array): string {
  return createHash("sha256").update(bytes).digest("hex");
}

async function downloadKddw(): Promise<string> {
  const source = path.join(cacheDir(), `kddockwidgets-${KDDW.version}`);
  if (existsSync(path.join(source, "CMakeLists.txt"))) {
    console.log(`KDDockWidgets: ${source} is already unpacked`);
    return source;
  }
  const downloads = path.join(cacheDir(), "downloads");
  mkdirSync(downloads, { recursive: true });
  const tarball = path.join(downloads, path.basename(KDDW.url));
  if (!existsSync(tarball) || readFileSync(tarball).length !== KDDW.size) {
    const response = await fetch(KDDW.url);
    if (!response.ok) throw new Error(`downloading ${KDDW.url}: ${response.status}`);
    writeFileSync(tarball, new Uint8Array(await response.arrayBuffer()));
  }
  const bytes = readFileSync(tarball);
  if (bytes.length !== KDDW.size || sha256(bytes) !== KDDW.sha256) {
    rmSync(tarball);
    throw new Error(`${tarball} isn't the pinned KDDockWidgets ${KDDW.version}; it was removed`);
  }
  const unpacking = `${source}.partial`;
  rmSync(unpacking, { recursive: true, force: true });
  mkdirSync(unpacking, { recursive: true });
  execFileSync("tar", ["-xzf", tarball, "-C", unpacking, "--strip-components=1"], {
    stdio: "inherit",
  });
  renameSync(unpacking, source);
  return source;
}

async function main(): Promise<void> {
  const qt = installQt();
  const kddw = await downloadKddw();
  console.log(`Qt ${QT_VERSION}: ${qt}`);
  console.log(`KDDockWidgets ${KDDW.version}: ${kddw}`);
}

await main();
