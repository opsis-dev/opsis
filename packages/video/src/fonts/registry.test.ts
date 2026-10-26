import { afterEach, describe, expect, it, vi } from "vitest";
import { registerFaces, unregisterFaces } from "#video/common/font-faces.ts";
import type { FontFamilyDef } from "#video/generated/spec.ts";

class UnreadableFace {
  load(): Promise<never> {
    return Promise.reject(new Error("Not a font."));
  }
}

const broken: FontFamilyDef = {
  id: "broken-sans",
  family: "Broken Sans",
  category: "sans-serif",
  source: "file",
  faces: [
    { file: "project:assets/fonts/broken.woff2", weight: "400", style: "normal", format: "woff2" },
  ],
};

afterEach(() => {
  unregisterFaces("df-test");
  vi.unstubAllGlobals();
});

describe("registering font faces", () => {
  it("reports an unreadable font once, however often the same font is registered", async () => {
    vi.stubGlobal("document", { fonts: { add: () => undefined, delete: () => true } });
    vi.stubGlobal("FontFace", UnreadableFace);
    const warnings: string[] = [];
    const warn = () => warnings.push(broken.family);
    for (let i = 0; i < 3; i++) await registerFaces("df-test", broken, (ref) => ref, warn);
    expect(warnings).toEqual(["Broken Sans"]);
    const changed = { ...broken, faces: [{ ...broken.faces[0], weight: "700" }] };
    await registerFaces("df-test", changed, (ref) => ref, warn);
    expect(warnings).toEqual(["Broken Sans", "Broken Sans"]);
  });
});
