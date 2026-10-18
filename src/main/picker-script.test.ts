import path from "node:path";
import { build, type Rolldown } from "vite";
import { describe, expect, it } from "vitest";
import { pickerScript } from "./picker-script";

type ScriptFor = (prefix: string) => string;

const PREFIX = "pick:";

class FakeElement {
  id = "";
  textContent = "";
  innerText = "";
  parentElement: FakeElement | null = null;
  children: FakeElement[] = [];
  style: Record<string, string> = {};

  constructor(
    readonly tagName: string,
    private readonly attributes: Record<string, string> = {},
  ) {}

  getAttribute(name: string): string | null {
    return this.attributes[name] ?? null;
  }

  hasAttribute(name: string): boolean {
    return name in this.attributes;
  }

  closest(): FakeElement {
    return this;
  }

  getBoundingClientRect() {
    return { left: 10, top: 40, width: 80, height: 24 };
  }
}

function installInFakePage(script: ScriptFor = pickerScript) {
  const listeners = new Map<string, (event: unknown) => void>();
  const logged: string[] = [];
  const document = {
    documentElement: { appendChild: () => undefined },
    createElement: (tag: string) => new FakeElement(tag.toUpperCase()),
    querySelector: () => null,
    addEventListener: (type: string, listener: (event: unknown) => void) =>
      listeners.set(type, listener),
  };
  const page = {
    window: {},
    document,
    Element: FakeElement,
    HTMLElement: FakeElement,
    CSS: { escape: (s: string) => s.replaceAll('"', '\\"') },
    console: { log: (line: string) => logged.push(line) },
  };
  const run = new Function(...Object.keys(page), script(PREFIX));
  run(...Object.values(page));
  const click = (event: {
    target: FakeElement | null;
    shiftKey: boolean;
    clientX?: number;
    clientY?: number;
  }) =>
    listeners.get("click")?.({
      clientX: 0,
      clientY: 0,
      preventDefault: () => undefined,
      stopPropagation: () => undefined,
      ...event,
    });
  const picks = () =>
    logged
      .filter((line) => line.startsWith(PREFIX))
      .map((line) => JSON.parse(line.slice(PREFIX.length)));
  return { click, picks };
}

describe("pickerScript", () => {
  it("runs on its own in a page and reports a clicked element by its accessible name", () => {
    const page = installInFakePage();
    page.click({
      target: new FakeElement("BUTTON", { "aria-label": "Save draft" }),
      shiftKey: false,
    });
    expect(page.picks()).toEqual([
      { target: "Save draft", css: "button", role: "button", selectorUsed: "accessible name" },
    ]);
  });

  it("reports a Shift-click as a spot", () => {
    const page = installInFakePage();
    page.click({ target: null, shiftKey: true, clientX: 12.4, clientY: 33.6 });
    expect(page.picks()).toEqual([{ target: "12, 34", css: "", role: "", selectorUsed: "spot" }]);
  });

  it("quotes a test id it falls back to", () => {
    const page = installInFakePage();
    page.click({ target: new FakeElement("DIV", { "data-testid": 'say "hi"' }), shiftKey: false });
    expect(page.picks()).toEqual([
      {
        target: 'say "hi"',
        css: '[data-testid="say \\"hi\\""]',
        role: "",
        selectorUsed: "accessible name",
      },
    ]);
  });
});

async function minifiedPickerScript(): Promise<ScriptFor> {
  const output = await build({
    configFile: false,
    logLevel: "silent",
    build: {
      write: false,
      minify: true,
      lib: {
        entry: path.resolve(import.meta.dirname, "picker-script.ts"),
        formats: ["cjs"],
        fileName: "picker-script",
      },
    },
  });
  const [bundle] = Array.isArray(output) ? output : [output];
  if (!("output" in bundle)) throw new Error("The picker script didn't build.");
  const chunk = bundle.output.find((o): o is Rolldown.OutputChunk => o.type === "chunk");
  if (!chunk) throw new Error("The picker script build has no code.");
  const module: { exports: { pickerScript?: ScriptFor } } = { exports: {} };
  new Function("module", "exports", chunk.code)(module, module.exports);
  if (!module.exports.pickerScript) throw new Error("The built picker script has no export.");
  return module.exports.pickerScript;
}

describe("pickerScript once minified like the packaged app", () => {
  it("still runs on its own in a page", async () => {
    const page = installInFakePage(await minifiedPickerScript());
    page.click({
      target: new FakeElement("BUTTON", { "aria-label": "Save draft" }),
      shiftKey: false,
    });
    expect(page.picks()).toEqual([
      { target: "Save draft", css: "button", role: "button", selectorUsed: "accessible name" },
    ]);
  });
});
