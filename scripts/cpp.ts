import { execFileSync, spawnSync } from "node:child_process";
import { existsSync, readdirSync, statSync } from "node:fs";
import path from "node:path";
import { ROOT } from "./lib/tool-config";

const CLANG_FORMAT = "clang-format==23.1.5";
const CLANG_TIDY = "clang-tidy==23.1.4";
const STYLE = "{BasedOnStyle: Google, FixNamespaceComments: false}";
const SOURCES = /^crates\/qt\/cpp\/.*\.(cc|h)$/;
const CHECKS = [
  "-*",
  "bugprone-*",
  "modernize-*",
  "cppcoreguidelines-*",
  "-modernize-use-trailing-return-type",
  "-modernize-use-nodiscard",
  "-cppcoreguidelines-owning-memory",
  "-cppcoreguidelines-avoid-magic-numbers",
  "-cppcoreguidelines-pro-type-static-cast-downcast",
  "-cppcoreguidelines-pro-bounds-array-to-pointer-decay",
  "-cppcoreguidelines-pro-type-vararg",
  "-cppcoreguidelines-avoid-do-while",
  "-bugprone-easily-swappable-parameters",
  "-cppcoreguidelines-prefer-member-initializer",
].join(",");

function sources(): string[] {
  return execFileSync("git", ["ls-files", "--cached", "--others", "--exclude-standard"], {
    cwd: ROOT,
    encoding: "utf8",
  })
    .split("\n")
    .filter((file) => SOURCES.test(file) && existsSync(path.join(ROOT, file)));
}

function run(tool: string, args: string[]): number {
  const result = spawnSync("uvx", ["-q", "--from", tool, ...args], { cwd: ROOT, stdio: "inherit" });
  return result.status ?? 1;
}

function format(): number {
  return run(CLANG_FORMAT, ["clang-format", `--style=${STYLE}`, "-i", ...sources()]);
}

function check(): number {
  return run(CLANG_FORMAT, [
    "clang-format",
    `--style=${STYLE}`,
    "--dry-run",
    "--Werror",
    ...sources(),
  ]);
}

function newestCompileCommands(): string {
  const build = path.join(ROOT, "target", "debug", "build", "opsis-qt");
  const found = readdirSync(build)
    .map((dir) => path.join(build, dir, "out", "compile_commands.json"))
    .filter((file) => existsSync(file))
    .toSorted((a, b) => statSync(b).mtimeMs - statSync(a).mtimeMs);
  const [newest] = found;
  if (!newest) throw new Error("opsis-qt wrote no compile_commands.json; build it first");
  return path.dirname(newest);
}

function lint(): number {
  const built = spawnSync("cargo", ["check", "-q", "-p", "opsis-qt"], {
    cwd: ROOT,
    stdio: "inherit",
  });
  if (built.status !== 0) return built.status ?? 1;
  const commands = newestCompileCommands();
  const files = sources().filter((file) => file.endsWith(".cc"));
  return run(CLANG_TIDY, [
    "clang-tidy",
    "-p",
    commands,
    `--checks=${CHECKS}`,
    `--header-filter=${path.join(ROOT, "crates", "qt", "cpp")}/.*`,
    "--system-headers=false",
    "--warnings-as-errors=*",
    "--quiet",
    ...files,
  ]);
}

const MODES: Record<string, () => number> = { format, check, lint };
const mode = MODES[process.argv[2] ?? ""];
if (!mode) {
  console.error("usage: bun scripts/cpp.ts format|check|lint");
  process.exit(2);
}
process.exit(mode());
