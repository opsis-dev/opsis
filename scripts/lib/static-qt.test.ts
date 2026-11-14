import { readFileSync } from "node:fs";
import path from "node:path";
import * as fc from "fast-check";
import { describe, expect, it } from "vitest";
import {
  buildSteps,
  configureArgs,
  MACOS_MINIMUM,
  parseSetOutput,
  QT_PLATFORMS,
  shellScript,
} from "./static-qt";
import { ROOT } from "./tool-config";

const LINUX_ONLY = ["-xcb", "-fontconfig", "-system-freetype"];
const platform = fc.constantFrom(...QT_PLATFORMS);
const segment = fc.stringMatching(/^[a-z][a-z0-9_]{0,8}$/);
const folder = fc.array(segment, { minLength: 1, maxLength: 4 }).map((parts) => parts.join("/"));

describe("the static Qt build", () => {
  it("is static, without ICU or a static C runtime, on every system", () => {
    fc.assert(
      fc.property(platform, (system) => {
        const args = configureArgs(system);
        expect(args).toEqual(expect.arrayContaining(["-static", "-release", "-no-icu"]));
        expect(args).not.toContain("-static-runtime");
      }),
    );
  });

  it("asks for the Linux display and font switches only on Linux", () => {
    fc.assert(
      fc.property(platform, (system) => {
        const asked = configureArgs(system).filter((arg) => LINUX_ONLY.includes(arg));
        expect(asked.toSorted()).toEqual(system === "linux" ? LINUX_ONLY.toSorted() : []);
      }),
    );
  });

  it("targets the macOS the app package declares", () => {
    const app = readFileSync(path.join(ROOT, "crates", "packager", "src", "app.rs"), "utf8");
    expect(app).toContain(`pub const MACOS_MINIMUM: &str = "${MACOS_MINIMUM}";`);
    const args = configureArgs("darwin");
    expect(args.slice(args.indexOf("--"))).toEqual([
      "--",
      `-DCMAKE_OSX_DEPLOYMENT_TARGET=${MACOS_MINIMUM}`,
    ]);
  });

  it("runs Qt's own scripts as batch files only on Windows, every step inside the build folder", () => {
    fc.assert(
      fc.property(platform, folder, folder, (system, build, prefix) => {
        const steps = buildSteps(
          system,
          "6.12.0",
          ["/t/qtbase.tar.xz", "/t/qtsvg.tar.xz"],
          `/${build}`,
          `/${prefix}`,
        );
        const qtScripts = steps.filter((step) => step.program !== "cmake");
        expect(qtScripts.length).toBe(2);
        for (const step of qtScripts)
          expect(step.program.endsWith(".bat")).toBe(system === "win32");
        for (const step of steps)
          expect(step.dir.replaceAll("\\", "/").startsWith(`/${build}`)).toBe(true);
      }),
    );
  });

  it("writes one shell line per step between clearing the folders and marking the prefix complete", () => {
    fc.assert(
      fc.property(folder, folder, (build, prefix) => {
        const steps = buildSteps(
          "linux",
          "6.12.0",
          ["/t/a.tar.xz", "/t/b.tar.xz"],
          `/${build}`,
          `/${prefix}`,
        );
        const lines = shellScript(steps, `/${build}`, `/${prefix}`).split("\n");
        expect(lines.length).toBe(steps.length + 4);
        expect(lines[0]).toBe(`rm -rf '/${build}' '/${prefix}'`);
        expect(lines.at(-1)).toBe(`touch '/${prefix}/.opsis-complete'`);
        steps.forEach((step, index) => expect(lines[index + 2]).toContain(`> '${step.log}'`));
      }),
    );
  });

  it("reads back every variable the Windows developer prompt sets", () => {
    fc.assert(
      fc.property(
        fc.dictionary(
          fc.stringMatching(/^[A-Za-z_][A-Za-z0-9_()]{0,15}$/),
          fc.stringMatching(/^[^\r\n]{0,30}$/),
        ),
        fc.constantFrom("\n", "\r\n"),
        (variables, newline) => {
          const text = [
            "noise without an equals sign",
            ...Object.entries(variables).map(([name, value]) => `${name}=${value}`),
            "=C:=C:\\",
          ].join(newline);
          expect(parseSetOutput(text)).toEqual(variables);
        },
      ),
    );
  });
});
