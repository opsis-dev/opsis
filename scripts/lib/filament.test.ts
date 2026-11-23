import * as fc from "fast-check";
import { describe, expect, it } from "vitest";
import { FILAMENT_LIBRARY_DIR, type FilamentPlatform, configureOptions } from "./filament";
import { tagCommit } from "./setup/pin";

const PLATFORMS: FilamentPlatform[] = ["linux", "darwin", "win32"];
const platform = fc.constantFrom(...PLATFORMS);
const commit = fc.stringMatching(/^[0-9a-f]{40}$/);
const tag = fc.stringMatching(/^v[0-9]{1,2}\.[0-9]{1,3}\.[0-9]{1,3}$/);

function option(args: string[], name: string): string | undefined {
  return args.find((arg) => arg.startsWith(`-D${name}=`))?.slice(`-D${name}=`.length);
}

describe("the Filament build", () => {
  it("has Vulkan, no OpenGL or WebGPU and its libraries in one known folder on every system", () => {
    fc.assert(
      fc.property(platform, (system) => {
        const args = configureOptions(system);
        expect(option(args, "FILAMENT_SUPPORTS_VULKAN")).toBe("ON");
        expect(option(args, "FILAMENT_SUPPORTS_WEBGPU")).toBe("OFF");
        expect(option(args, "FILAMENT_SUPPORTS_OPENGL")).toBe("OFF");
        expect(option(args, "DIST_DIR")).toBe(FILAMENT_LIBRARY_DIR);
        expect(option(args, "CMAKE_BUILD_TYPE")).toBe("Release");
      }),
    );
  });

  it("uses Windows' dynamic C runtime, as Rust does", () => {
    expect(option(configureOptions("win32"), "USE_STATIC_CRT")).toBe("OFF");
  });

  it("opens the Vulkan loader the engine ships on macOS", () => {
    expect(option(configureOptions("darwin"), "CMAKE_CXX_FLAGS")).toContain("libvulkan.dylib");
  });
});

describe("a tag's commit", () => {
  it("is the peeled commit of an annotated tag, else the tag's own", () => {
    fc.assert(
      fc.property(tag, commit, commit, fc.boolean(), (name, own, peeled, annotated) => {
        const lines = [`${own}\trefs/tags/${name}`];
        if (annotated) lines.push(`${peeled}\trefs/tags/${name}^{}`);
        expect(tagCommit(lines.join("\n"), name)).toBe(annotated ? peeled : own);
      }),
    );
  });

  it("is missing when the tag is", () => {
    fc.assert(
      fc.property(tag, tag, commit, (name, other, own) => {
        fc.pre(name !== other);
        expect(tagCommit(`${own}\trefs/tags/${other}\n`, name)).toBeNull();
      }),
    );
  });
});
