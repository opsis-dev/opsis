import { afterEach, describe, expect, it, vi } from "vitest";
import { addStyles, beforeFirstLoad, hidingCss } from "./page-scripts";

describe("page scripts before the first page", () => {
  it("run once per site, before the page's own code", () => {
    const store = new Map<string, string>();
    const sessionStorage = {
      getItem: (k: string) => store.get(k) ?? null,
      setItem: (k: string, v: string) => void store.set(k, v),
    };
    const ran: string[] = [];
    const run = new Function("sessionStorage", "ran", beforeFirstLoad("n1", "ran.push('seeded');"));
    run(sessionStorage, ran);
    run(sessionStorage, ran);
    expect(ran).toEqual(["seeded"]);
  });
});

describe("hiding clutter", () => {
  it("gives each selector its own rule, so one bad selector hides nothing less", () => {
    expect(hidingCss([".cookie-banner", "div["]).split("\n")).toEqual([
      ".cookie-banner { display: none !important; visibility: hidden !important; }",
      "div[ { display: none !important; visibility: hidden !important; }",
    ]);
  });

  afterEach(() => vi.unstubAllGlobals());

  const fakeDocument = (root: object | null) => {
    let observer: (() => void) | undefined;
    const doc = {
      documentElement: root,
      createElement: () => ({ textContent: "" }),
    };
    class Watch {
      constructor(cb: () => void) {
        observer = cb;
      }
      observe() {}
      disconnect() {
        observer = undefined;
      }
    }
    const run = () => {
      vi.stubGlobal("document", doc);
      vi.stubGlobal("MutationObserver", Watch);
      addStyles(".banner { display: none }");
    };
    return { doc, run, fire: () => observer?.() };
  };

  it("adds the style at once when the page has a document", () => {
    const added: unknown[] = [];
    const { run } = fakeDocument({ appendChild: (s: unknown) => added.push(s) });
    run();
    expect(added).toHaveLength(1);
  });

  it("waits for the document when it runs before the page exists", () => {
    const added: unknown[] = [];
    const page = fakeDocument(null);
    page.run();
    expect(added).toHaveLength(0);
    page.doc.documentElement = { appendChild: (s: unknown) => added.push(s) };
    page.fire();
    page.fire();
    expect(added).toHaveLength(1);
  });
});
