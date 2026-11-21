import fc from "fast-check";
import { describe, expect, it } from "vitest";
import { type Registry, presets } from "./agent-registry";

const name = fc.stringMatching(/^[a-z][a-z0-9-]{0,12}$/);
const version = fc.stringMatching(/^\d{1,2}\.\d{1,2}\.\d{1,2}$/);

const agent = fc.record({
  id: name,
  name,
  version,
  distribution: fc.record(
    {
      binary: fc.dictionary(
        fc.constantFrom("linux-x86_64", "darwin-aarch64", "windows-x86_64"),
        fc.record({
          cmd: fc.oneof(
            name.map((n) => `./${n}`),
            name.map((n) => `bin\\${n}.exe`),
          ),
        }),
      ),
      npx: fc.record({
        package: fc.oneof(
          name,
          fc.tuple(name, name, version).map(([scope, pkg, v]) => `@${scope}/${pkg}@${v}`),
        ),
      }),
      uvx: fc.record({ package: name }),
    },
    { requiredKeys: [] },
  ),
});

describe("the agent registry snapshot", () => {
  it("names each agent's programs plainly, once each, and keeps only agents it can start", () => {
    fc.assert(
      fc.property(fc.array(agent, { maxLength: 8 }), (agents) => {
        const source: Registry = { version: "1.0.0", agents };
        for (const preset of presets(source)) {
          expect(preset.launches.length).toBeGreaterThan(0);
          const programs = preset.launches.map((launch) => launch.program);
          expect(new Set(programs).size).toBe(programs.length);
          for (const program of programs) expect(program).toMatch(/^[\w.+-]+$/);
          expect(programs.some((p) => p.includes("@") || p.endsWith(".exe"))).toBe(false);
        }
      }),
    );
  });

  it("drops the version and scope from an npm package's program", () => {
    const source: Registry = {
      version: "1.0.0",
      agents: [
        {
          id: "claude-acp",
          name: "Claude Agent",
          version: "0.109.0",
          distribution: {
            binary: { "linux-x86_64": { cmd: "./claude-agent-acp" } },
            npx: { package: "@agentclientprotocol/claude-agent-acp@0.109.0" },
          },
        },
      ],
    };
    expect(presets(source)[0]?.launches).toEqual([{ program: "claude-agent-acp", args: [] }]);
  });
});
