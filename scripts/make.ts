import { execFileSync } from "node:child_process";
import { homedir } from "node:os";
import path from "node:path";
import { buildAndPackage } from "./lib/release";
import { ROOT } from "./lib/tool-config";

const OUT = path.join(homedir(), ".cache", "opsis-build");
const LOCAL_FORMATS: Partial<Record<NodeJS.Platform, string>> = {
  linux: "deb",
  darwin: "app,dmg",
  win32: "nsis,msi",
};

function splitArgs(args: string[]): { formats: string | undefined; cargo: string[] } {
  const at = args.indexOf("--formats");
  if (at === -1) return { formats: undefined, cargo: args };
  return { formats: args[at + 1], cargo: [...args.slice(0, at), ...args.slice(at + 2)] };
}

const { formats, cargo } = splitArgs(process.argv.slice(2));
const chosen = formats ?? LOCAL_FORMATS[process.platform];
if (!chosen) throw new Error(`No package formats are known for ${process.platform}.`);
execFileSync("bun", ["run", "stage"], { cwd: ROOT, stdio: "inherit" });
await buildAndPackage({ cargoArgs: cargo, formats: chosen, out: OUT });
