import { describe, expect, it } from "vitest";
import { workflowNode } from "../../shared/test-fixtures";
import { landedOnSignIn, plainError, traceGroupTitle } from "./driver-helpers";

describe("traceGroupTitle", () => {
  it("names the step after what it acts on, with variables filled in", () => {
    expect(
      traceGroupTitle("Navigate", workflowNode("browser.navigate", { url: "{{BASE}}/login" }), {
        BASE: "https://x.test",
      }),
    ).toBe("Navigate · https://x.test/login");
    expect(
      traceGroupTitle(
        "Type",
        workflowNode("browser.type", { target: "Username", text: "tom" }),
        {},
      ),
    ).toBe("Type · Username");
    expect(
      traceGroupTitle(
        "Assert",
        workflowNode("browser.assert", { text: "Welcome", target: "" }),
        {},
      ),
    ).toBe("Assert · Welcome");
    expect(traceGroupTitle("Wait", workflowNode("browser.wait", { ms: 500 }), {})).toBe("Wait");
  });
});

describe("plainError", () => {
  const click = workflowNode("browser.click", { target: "css=#add-to-cart" });
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
        workflowNode("browser.navigate", {}),
        {},
      ),
    ).toBe("The page didn't respond within 30 s.");
    expect(plainError(new Error("page.goto: Timeout 90000ms exceeded."), click, {})).toBe(
      "The page didn't respond within 1 min 30 s.",
    );
  });

  it("explains network failures", () => {
    expect(
      plainError(new Error("page.goto: net::ERR_NAME_NOT_RESOLVED at https://x"), click, {}),
    ).toBe("Couldn't reach the site. Check the address.");
    expect(
      plainError(new Error("page.goto: net::ERR_CONNECTION_REFUSED at https://x"), click, {}),
    ).toBe("Couldn't load the page (ERR_CONNECTION_REFUSED).");
  });

  it("drops the internal prefix from other errors", () => {
    expect(plainError(new Error("locator.click: Element is not attached\nmore"), click, {})).toBe(
      "Element is not attached",
    );
  });
});

describe("expired sign-in", () => {
  const signIn = "https://x.test/login";
  const away = "https://x.test/app";
  const root = "https://www.saucedemo.com/";
  const inventory = "https://www.saucedemo.com/inventory.html";
  it.each([
    [away, "https://x.test/login/", signIn, true],
    [away, "https://x.test/login/index.html", signIn, true],
    [away, "https://x.test/login?next=app", signIn, true],
    [away, "https://x.test/app", signIn, false],
    [away, "not a url", signIn, false],
    ["https://x.test/login?next=app", "https://x.test/login", signIn, false],
    [inventory, root, root, true],
    [inventory, inventory, root, false],
    [root, root, root, false],
  ])("asked for %s, landed on %s, sign-in at %s: %s", (requested, current, page, expired) => {
    expect(landedOnSignIn(requested, current, page)).toBe(expired);
  });
});
