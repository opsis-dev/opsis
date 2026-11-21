import { z } from "zod";

export const REGISTRY_URL = "https://cdn.agentclientprotocol.com/registry/v1/latest/registry.json";

const launch = z.object({ args: z.array(z.string()).optional() });
const binary = launch.extend({ cmd: z.string() });
const packaged = launch.extend({ package: z.string() });

const registryAgent = z.object({
  id: z.string().min(1),
  name: z.string().min(1),
  version: z.string(),
  description: z.string().optional(),
  repository: z.string().optional(),
  website: z.string().optional(),
  license: z.string().optional(),
  distribution: z.object({
    binary: z.record(z.string(), binary).optional(),
    npx: packaged.optional(),
    uvx: packaged.optional(),
  }),
});

export const registry = z.object({
  version: z.string(),
  agents: z.array(registryAgent),
});

export type Registry = z.infer<typeof registry>;

export type Launch = { program: string; args: string[] };

export type Preset = {
  id: string;
  name: string;
  version: string;
  description: string;
  website: string;
  license: string;
  launches: Launch[];
};

function programOf(path: string): string {
  const name = path.split(/[\\/]/).pop() ?? "";
  return name.replace(/\.exe$/i, "");
}

function packageProgram(spec: string): string {
  const withoutVersion = spec.replace(/(.)@[^/@]*$/, "$1");
  return withoutVersion.split("/").pop() ?? "";
}

function launches(agent: z.infer<typeof registryAgent>): Launch[] {
  const found: Launch[] = [];
  const binaries = Object.values(agent.distribution.binary ?? {});
  const first = binaries[0];
  if (first) found.push({ program: programOf(first.cmd), args: first.args ?? [] });
  for (const tool of [agent.distribution.npx, agent.distribution.uvx]) {
    if (tool) found.push({ program: packageProgram(tool.package), args: tool.args ?? [] });
  }
  const seen = new Set<string>();
  return found.filter(
    (launch) =>
      /^[\w.+-]+$/.test(launch.program) && !seen.has(launch.program) && seen.add(launch.program),
  );
}

export function presets(source: Registry): Preset[] {
  return source.agents
    .map((agent) => ({
      id: agent.id,
      name: agent.name,
      version: agent.version,
      description: agent.description ?? "",
      website: agent.website ?? agent.repository ?? "",
      license: agent.license ?? "",
      launches: launches(agent),
    }))
    .filter((preset) => preset.launches.length > 0)
    .sort((a, b) => a.name.localeCompare(b.name) || a.id.localeCompare(b.id));
}
