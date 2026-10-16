import { describe, expect, it } from "vitest";
import type { WorkflowNode } from "../../shared/schema/workflow";
import {
  beforeFirstLoad,
  hideScript,
  landedOnSignIn,
  setupLength,
  plainError,
  samePage,
  traceGroupTitle,
} from "./driver";

const node = (type: string, params: Record<string, unknown>) =>
  ({ id: "n", type, position: { x: 0, y: 0 }, params }) as unknown as WorkflowNode;

describe("traceGroupTitle", () => {
  it("names the step after what it acts on, with variables filled in", () => {
    expect(
      traceGroupTitle("Navigate", node("browser.navigate", { url: "{{BASE}}/login" }), {
        BASE: "https://x.test",
      }),
    ).toBe("Navigate · https://x.test/login");
    expect(
      traceGroupTitle("Type", node("browser.type", { target: "Username", text: "tom" }), {}),
    ).toBe("Type · Username");
    expect(
      traceGroupTitle("Assert", node("browser.assert", { text: "Welcome", target: "" }), {}),
    ).toBe("Assert · Welcome");
    expect(traceGroupTitle("Wait", node("browser.wait", { ms: 500 }), {})).toBe("Wait");
  });
});

describe("plainError", () => {
  const click = node("browser.click", { target: "css=#add-to-cart" });
  it("explains timeouts in words", () => {
    expect(
      plainError(
        new Error(
          "locator.waitFor: Timeout 15000ms exceeded.\nCall log:\n\u001b[2m  - waiting for locator('#add-to-cart').first() to be visible\u001b[22m\n",
        ),
        click,
        {},
      ),
    ).toBe("Couldn't find “#add-to-cart” on the page within 15 s.");
    expect(
      plainError(
        new Error("page.goto: Timeout 30000ms exceeded."),
        node("browser.navigate", {}),
        {},
      ),
    ).toBe("The page didn't respond within 30 s.");
  });

  it("explains network failures", () => {
    expect(
      plainError(new Error("page.goto: net::ERR_NAME_NOT_RESOLVED at https://x"), click, {}),
    ).toBe("Couldn't reach the site. Check the address.");
  });

  it("drops the internal prefix from other errors", () => {
    expect(plainError(new Error("locator.click: Element is not attached\nmore"), click, {})).toBe(
      "Element is not attached",
    );
  });
});

describe("expired sign-in", () => {
  it("compares pages by address, ignoring index files and trailing slashes", () => {
    expect(samePage("https://x.test/", "https://x.test")).toBe(true);
    expect(samePage("https://x.test/app/index.html", "https://x.test/app")).toBe(true);
    expect(samePage("https://x.test/app?a=1", "https://x.test/app")).toBe(true);
    expect(samePage("https://x.test/app", "https://x.test/login")).toBe(false);
    expect(samePage("not a url", "https://x.test")).toBe(false);
  });

  it("notices a redirect back to the sign-in page", () => {
    const signIn = "https://www.saucedemo.com/";
    expect(
      landedOnSignIn(
        "https://www.saucedemo.com/inventory.html",
        "https://www.saucedemo.com/",
        signIn,
      ),
    ).toBe(true);
    expect(
      landedOnSignIn(
        "https://www.saucedemo.com/inventory.html",
        "https://www.saucedemo.com/inventory.html",
        signIn,
      ),
    ).toBe(false);
    expect(landedOnSignIn("https://www.saucedemo.com/", "https://www.saucedemo.com/", signIn)).toBe(
      false,
    );
  });
});

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
  const fakeDocument = (root: object | null) => {
    let observer: (() => void) | undefined;
    const doc = {
      documentElement: root,
      head: null,
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
    const run = () =>
      new Function("document", "MutationObserver", hideScript(".banner { display: none }"))(
        doc,
        Watch,
      );
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

describe("what happens before the camera rolls", () => {
  const step = (type: string, params: Record<string, unknown> = {}) => ({
    id: type,
    type,
    position: { x: 0, y: 0 },
    params,
  });

  it("keeps waits and invisible steps right after the first page off camera", () => {
    const steps = [
      step("browser.setState", { script: "1" }),
      step("browser.navigate", { url: "https://example.com" }),
      step("browser.wait", { mode: "element", target: "Board" }),
      step("browser.hideElements"),
      step("browser.click", { target: "Open" }),
      step("browser.wait", { mode: "element", target: "Panel" }),
    ];
    expect(setupLength(steps, {})).toBe(4);
  });

  it("films a fixed pause after the first page", () => {
    const steps = [
      step("browser.navigate", { url: "https://example.com" }),
      step("browser.wait", { mode: "ms", ms: 1500 }),
    ];
    expect(setupLength(steps, {})).toBe(1);
  });

  it("has no setup without a page to open", () => {
    expect(setupLength([step("browser.press", { keys: "a" })], {})).toBe(0);
  });
});
