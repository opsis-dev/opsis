import { execFileSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, readdirSync, statSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { parseArgs } from "node:util";
import { stagedFfmpeg } from "./lib/ffmpeg";
import { CONFIG_FILE, type InsideConfig, PROFILERS, type Profiler } from "./lib/profile/config";
import { writeInputs } from "./lib/profile/inputs";
import { type Catalog, missingTypes, profileWorkflow, runOrder } from "./lib/profile/project";
import { writeReport } from "./lib/profile/report";
import { ROOT } from "./lib/tool-config";
import { DEV_ENGINE, seedUserData } from "./lib/user-data";

const PROFILE_DIR = path.join(os.homedir(), ".cache", "opsis-profile");
const DEB_DIR = path.join(os.homedir(), ".cache", "opsis-build", "release", "bundle", "deb");
const IMAGE = "opsis-profile";
const SAMPLY_VERSION = "0.13.1";
const SAMPLY_SHA256 = "61875daad67888798690dea3cb2748279df6ac299c5c6a857d67eed7642473d9";
const SITE_PORT = 4321;
const APP = "/usr/bin/opsis-desktop";
const EDITOR_DATA = path.join(ROOT, "target", "release", "editor-data");
const RUN_LAST = new Set(["browser.signIn"]);

const { values } = parseArgs({
  options: {
    profiler: { type: "string", default: "none" },
    only: { type: "string" },
    cpus: { type: "string", default: "0-3" },
    memory: { type: "string", default: "6g" },
    "sample-ms": { type: "string", default: "250" },
    "timeout-s": { type: "string", default: "900" },
    deb: { type: "string" },
    gpu: { type: "boolean", default: false },
    "render-gl": { type: "string" },
    "app-env": { type: "string", multiple: true, default: [] },
  },
});

const DRI = "/dev/dri";

function gpuArgs(): string[] {
  if (!values.gpu) return [];
  const groups = new Set(
    readdirSync(DRI)
      .filter((name) => /^(card|renderD)\d+$/.test(name))
      .map((name) => String(statSync(path.join(DRI, name)).gid)),
  );
  return ["--device", DRI, ...[...groups].flatMap((group) => ["--group-add", group])];
}

function isProfiler(value: string): value is Profiler {
  return (PROFILERS as readonly string[]).includes(value);
}

function run(command: string, args: string[], cwd = ROOT): void {
  execFileSync(command, args, { cwd, stdio: "inherit" });
}

function catalog(): Catalog {
  run("cargo", ["build", "-q", "--release", "-p", "opsis-compile", "--bin", "editor-data"]);
  return JSON.parse(
    execFileSync(EDITOR_DATA, ["catalog"], { encoding: "utf8", maxBuffer: 1 << 26 }),
  );
}

function debFile(): string {
  if (values.deb) return path.resolve(values.deb);
  const debs = existsSync(DEB_DIR)
    ? readdirSync(DEB_DIR).filter((name) => name.endsWith(".deb"))
    : [];
  if (debs.length !== 1)
    throw new Error(
      `Expected one deb in ${DEB_DIR}, found ${debs.length}. Build it with bun run make, or pass --deb.`,
    );
  return path.join(DEB_DIR, debs[0]);
}

function buildImage(deb: string): void {
  const context = path.join(PROFILE_DIR, "image");
  mkdirSync(context, { recursive: true });
  copyFileSync(
    path.join(ROOT, "scripts", "lib", "profile", "Dockerfile"),
    path.join(context, "Dockerfile"),
  );
  copyFileSync(deb, path.join(context, "opsis.deb"));
  run("docker", [
    "build",
    "-t",
    IMAGE,
    "--build-arg",
    `SAMPLY_VERSION=${SAMPLY_VERSION}`,
    "--build-arg",
    `SAMPLY_SHA256=${SAMPLY_SHA256}`,
    context,
  ]);
}

function stamp(): string {
  return new Date().toISOString().slice(0, 19).replaceAll(":", "-");
}

function bunProgram(): string {
  return execFileSync("which", ["bun"], { encoding: "utf8" }).trim();
}

function runContainer(runDir: string): void {
  const uid = os.userInfo().uid;
  const gid = os.userInfo().gid;
  const mounts = [
    [ROOT, ROOT, "ro"],
    [DEV_ENGINE, DEV_ENGINE, "ro"],
    [runDir, runDir, "rw"],
    [bunProgram(), "/usr/local/bin/bun", "ro"],
  ];
  run("docker", [
    "run",
    "--rm",
    "--init",
    "--name",
    `opsis-profile-${path.basename(runDir)}`,
    "--cpuset-cpus",
    values.cpus,
    "--memory",
    values.memory,
    "--memory-swap",
    values.memory,
    "--shm-size",
    "2g",
    "--network",
    "none",
    "--cap-add",
    "PERFMON",
    "--cap-add",
    "IPC_LOCK",
    "--user",
    `${uid}:${gid}`,
    "-e",
    "HOME=/tmp/home",
    ...gpuArgs(),
    ...mounts.flatMap(([from, to, mode]) => ["-v", `${from}:${to}:${mode}`]),
    IMAGE,
    "bun",
    path.join(ROOT, "scripts", "profile-inside.ts"),
    path.join(runDir, CONFIG_FILE),
  ]);
}

function main(): void {
  const profiler = values.profiler;
  if (!isProfiler(profiler)) throw new Error(`--profiler is one of ${PROFILERS.join(", ")}.`);
  const nodes = catalog();
  const workflow = profileWorkflow(nodes, `http://127.0.0.1:${SITE_PORT}`);
  const renderGl = values["render-gl"];
  if (renderGl)
    for (const node of workflow.nodes)
      if (node.type === "output.render") node.params = { ...node.params, gl: renderGl };
  const missing = missingTypes(nodes, workflow);
  if (missing.length > 0) throw new Error(`The profile project leaves out ${missing.join(", ")}.`);
  const only = values.only ? new RegExp(values.only) : null;
  const types = new Map(workflow.nodes.map((node) => [node.id, node.type]));
  const targets = runOrder(nodes, workflow)
    .map((id) => ({ id, type: types.get(id) ?? "" }))
    .filter((target) => !only || only.test(`${target.type} ${target.id}`))
    .sort((a, b) => Number(RUN_LAST.has(a.type)) - Number(RUN_LAST.has(b.type)));
  const runDir = path.join(
    PROFILE_DIR,
    "runs",
    `${stamp()}-${profiler}${values.gpu ? "-gpu" : ""}`,
  );
  const userData = path.join(runDir, "user-data");
  const project = path.join(runDir, "project");
  const out = path.join(runDir, "out");
  mkdirSync(out, { recursive: true });
  writeInputs(project, stagedFfmpeg());
  writeFileSync(path.join(project, "project.opsis.json"), `${JSON.stringify(workflow, null, 2)}\n`);
  seedUserData(userData);
  const config: InsideConfig = {
    userData,
    project,
    out,
    sitePort: SITE_PORT,
    sampleMs: Number(values["sample-ms"]),
    nodeTimeoutMs: Number(values["timeout-s"]) * 1000,
    profiler,
    app: APP,
    appEnv: Object.fromEntries(
      values["app-env"].map((pair) => {
        const [name, ...value] = pair.split("=");
        return [name, value.join("=")];
      }),
    ),
    targets,
  };
  writeFileSync(path.join(runDir, CONFIG_FILE), JSON.stringify(config, null, 2));
  buildImage(debFile());
  console.log(`Profiling ${targets.length} runs in ${runDir}`);
  try {
    runContainer(runDir);
  } finally {
    if (existsSync(path.join(out, "runs.json"))) {
      const costs = writeReport(out, `Opsis profile ${path.basename(runDir)}`);
      const failed = costs.filter((cost) => !cost.ok);
      console.log(
        `Report: ${path.join(out, "report.html")} (${costs.length} runs, ${failed.length} failed)`,
      );
    }
  }
}

main();
