import { describe, expect, it } from "vitest";
import type { WorkflowNode } from "../../shared/schema/workflow";
import { landedOnSignIn, plainError, setupLength, traceGroupTitle } from "./driver-helpers";

const node = (type: string, params: Record<string, unknown> = {}): WorkflowNode => ({
  id: "n",
  type,
  position: { x: 0, y: 0 },
  params,
});

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
  it("compares pages by address, ignoring index files, queries and trailing slashes", () => {
    const signIn = "https://x.test/login";
    const away = "https://x.test/app";
    expect(landedOnSignIn(away, "https://x.test/login/", signIn)).toBe(true);
    expect(landedOnSignIn(away, "https://x.test/login/index.html", signIn)).toBe(true);
    expect(landedOnSignIn(away, "https://x.test/login?next=app", signIn)).toBe(true);
    expect(landedOnSignIn(away, "https://x.test/app", signIn)).toBe(false);
    expect(landedOnSignIn(away, "not a url", signIn)).toBe(false);
    expect(landedOnSignIn("https://x.test/login?next=app", "https://x.test/login", signIn)).toBe(
      false,
    );
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

describe("what happens before the camera rolls", () => {
  it("keeps waits and invisible steps right after the first page off camera", () => {
    const steps = [
      node("browser.setState", { script: "1" }),
      node("browser.navigate", { url: "https://example.com" }),
      node("browser.wait", { mode: "element", target: "Board" }),
      node("browser.hideElements"),
      node("browser.click", { target: "Open" }),
      node("browser.wait", { mode: "element", target: "Panel" }),
    ];
    expect(setupLength(steps, {})).toBe(4);
  });

  it("films a fixed pause after the first page", () => {
    const steps = [
      node("browser.navigate", { url: "https://example.com" }),
      node("browser.wait", { mode: "ms", ms: 1500 }),
    ];
    expect(setupLength(steps, {})).toBe(1);
  });

  it("has no setup without a page to open", () => {
    expect(setupLength([node("browser.press", { keys: "a" })], {})).toBe(0);
  });
});
