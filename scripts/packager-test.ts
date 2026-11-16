import { execFileSync } from "node:child_process";
import { appImageTools } from "./lib/appimage-tools";
import { hostTarget } from "./lib/ffmpeg";
import { ROOT } from "./lib/tool-config";

const tools = await appImageTools(hostTarget());

execFileSync(
  "cargo",
  ["test", "-p", "opsis-packager", "--", "--include-ignored", ...process.argv.slice(2)],
  {
    cwd: ROOT,
    stdio: "inherit",
    env: {
      ...process.env,
      CARGO_BUILD_JOBS: process.env.CARGO_BUILD_JOBS ?? "8",
      OPSIS_APPIMAGETOOL: tools.appimagetool,
      OPSIS_APPIMAGE_RUNTIME: tools.runtime,
      APPIMAGE_EXTRACT_AND_RUN: "1",
    },
  },
);
