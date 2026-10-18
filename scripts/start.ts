import { spawn } from "node:child_process";
import path from "node:path";
import { ROOT } from "./lib/tool-config";

const forge = path.join(
  ROOT,
  "node_modules",
  ".bin",
  process.platform === "win32" ? "electron-forge.cmd" : "electron-forge",
);
const electronArgs = process.platform === "linux" ? ["--no-sandbox"] : [];
const extra = process.argv.slice(2);
const args = [
  "start",
  ...(electronArgs.length || extra.length ? ["--", ...electronArgs, ...extra] : []),
];
const child = spawn(forge, args, { stdio: "inherit", shell: process.platform === "win32" });
child.on("exit", (code, signal) => process.exit(code ?? (signal ? 1 : 0)));
