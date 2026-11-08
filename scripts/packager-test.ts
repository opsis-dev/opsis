import { homedir } from "node:os";
import path from "node:path";
import { APPIMAGE_TOOLS_CACHE, appImageTools } from "./lib/appimage-tools";
import { hostTarget } from "./lib/ffmpeg";
import { ROOT } from "./lib/tool-config";
import { runInTrixie } from "./lib/trixie";

const SYSTEM_PATH = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";

const tools = await appImageTools(hostTarget());
const cargoHome = process.env.CARGO_HOME ?? path.join(homedir(), ".cargo");
const rustupHome = process.env.RUSTUP_HOME ?? path.join(homedir(), ".rustup");
const target = path.join(ROOT, "target", "trixie");

runInTrixie({
  script: [
    `cd ${ROOT}`,
    `cargo test -p opsis-packager -- --include-ignored ${process.argv.slice(2).join(" ")}`,
  ].join("\n"),
  mounts: [ROOT, cargoHome, rustupHome, APPIMAGE_TOOLS_CACHE],
  env: {
    PATH: `${path.join(cargoHome, "bin")}:${SYSTEM_PATH}`,
    CARGO_HOME: cargoHome,
    RUSTUP_HOME: rustupHome,
    CARGO_TARGET_DIR: target,
    CARGO_BUILD_JOBS: process.env.CARGO_BUILD_JOBS ?? "8",
    OPSIS_APPIMAGETOOL: tools.appimagetool,
    OPSIS_APPIMAGE_RUNTIME: tools.runtime,
    APPIMAGE_EXTRACT_AND_RUN: "1",
  },
});
