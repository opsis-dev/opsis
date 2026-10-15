import { describe, expect, it } from "vitest";
import type { WorkflowNode } from "../../shared/schema/workflow";
import { landedOnSignIn, plainError, samePage, traceGroupTitle } from "./driver";

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
