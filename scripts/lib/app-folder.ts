import { copyFileSync, cpSync, existsSync, mkdirSync, rmSync } from "node:fs";
import path from "node:path";
import { stagedFfmpeg } from "./ffmpeg";
import { AGENT_PROGRAM, MCP_PROGRAM, RENDERER_PROGRAM, releaseBinary } from "./release";
import { ROOT } from "./tool-config";

const FFMPEG_PROGRAM = "opsis-ffmpeg";
const FOLDERS = ["public", "resources"];
const FILES = ["LICENSE", "THIRD_PARTY_NOTICES.md", "QT_NOTICES.generated.md"];

export function assembleAppFolder(folder: string): string {
  const program = releaseBinary();
  const renderer = releaseBinary(RENDERER_PROGRAM);
  const mcp = releaseBinary(MCP_PROGRAM);
  const agent = releaseBinary(AGENT_PROGRAM);
  const missing = [
    program,
    renderer,
    mcp,
    agent,
    stagedFfmpeg(),
    ...FOLDERS.map((name) => path.join(ROOT, name)),
  ].filter((needed) => !existsSync(needed));
  if (missing.length > 0)
    throw new Error(
      `Run bun run make first; missing ${missing.map((file) => path.relative(ROOT, file)).join(", ")}.`,
    );
  rmSync(folder, { recursive: true, force: true });
  mkdirSync(folder, { recursive: true });
  copyFileSync(program, path.join(folder, "opsis"));
  copyFileSync(stagedFfmpeg(), path.join(folder, FFMPEG_PROGRAM));
  copyFileSync(renderer, path.join(folder, path.basename(renderer)));
  copyFileSync(mcp, path.join(folder, path.basename(mcp)));
  copyFileSync(agent, path.join(folder, path.basename(agent)));
  for (const name of FOLDERS)
    cpSync(path.join(ROOT, name), path.join(folder, name), { recursive: true });
  for (const name of FILES) copyFileSync(path.join(ROOT, name), path.join(folder, name));
  return path.join(folder, "opsis");
}
