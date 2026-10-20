import { describe, expect, it } from "vitest";
import { must, nodeCard } from "../test-fixtures";
import { PAUSE_MARK } from "../vars";
import {
  assertNode,
  browserNodes,
  dragNode,
  mockRouteNode,
  navigateNode,
  selectNode,
  typeNode,
  uploadNode,
  waitNode,
} from "./browser";
import { parseNodeParams } from "./define";
import { parseParams } from "./params";
import { defaultParams, getNodeDef } from "./registry";

describe("browser action cards", () => {
  it("shows what the Type text node does", () => {
    const out = nodeCard("browser.type", {
      target: "Username",
      targetRole: "textbox",
      text: "tom¦smith",
      pressEnter: true,
      after: 500,
      cluster: "login",
    });
    expect(out.map((r) => [r.icon, r.text])).toEqual([
      ["crosshair", "“Username” field"],
      ["keyboard", "“tom … smith”, then Enter"],
      ["timer", "Pause 0.5 s"],
      ["zoom-in", "Zoom group “login”"],
    ]);
  });

  it("asks for the missing element in the warning tone", () => {
    expect(nodeCard("browser.click", { target: "" })[0]).toMatchObject({
      text: "Pick an element",
      tone: "warn",
    });
  });

  it("shows what a Drag picks up and where it goes", () => {
    const texts = (params: Record<string, unknown>) =>
      nodeCard("browser.drag", { target: "Deal card", after: 0, ...params }).map((r) => r.text);
    expect(texts({ mode: "element", to: "Won" })).toEqual(["“Deal card”", "Onto “Won”"]);
    expect(texts({ mode: "element" })).toEqual(["“Deal card”", "Pick where to drop it"]);
    expect(texts({ mode: "by", dx: 300, dy: -40 })).toEqual(["“Deal card”", "By 300, -40 px"]);
    expect(texts({ mode: "path", path: "0, 0\n40, 40\n120, -80", cluster: "Board" })).toEqual([
      "“Deal card”",
      "Along 3 points",
      "Zoom group “Board”",
    ]);
    expect(texts({ mode: "path", path: "" })).toEqual(["“Deal card”", "Add points to the path"]);
    expect(texts({ mode: "path", path: "100, 0" })).toEqual(["“Deal card”", "Along 1 point"]);
    expect(texts({ mode: "path", path: "100, 0\nnot a point" })).toEqual([
      "“Deal card”",
      "Line 2 of the path isn't two numbers",
    ]);
    expect(texts({ mode: "path", path: "0, 0" })).toEqual([
      "“Deal card”",
      "Add a point away from where the drag starts",
    ]);
  });

  it("doesn't call passing over an element a 0 s hover", () => {
    const texts = (hoverMs: number) =>
      nodeCard("browser.hover", { target: "Menu", hoverMs, after: 0 }).map((r) => r.text);
    expect(texts(600)).toEqual(["“Menu”", "Hover 0.6 s"]);
    expect(texts(0)).toEqual(["“Menu”", "Passes over it"]);
  });

  it("asks for the address a faked response answers", () => {
    expect(nodeCard("browser.mockRoute", { url: " " })[0]).toMatchObject({
      text: "Add an address pattern",
      tone: "warn",
    });
    expect(parseParams(mockRouteNode.params, { url: "" })).toEqual({
      ok: false,
      problem: "Add an address pattern",
    });
    expect(parseParams(mockRouteNode.params, {}).ok).toBe(true);
  });

  it("asks for a script on an empty Run page script", () => {
    expect(nodeCard("browser.setState", { script: " " })[0]).toMatchObject({ tone: "warn" });
  });
});

describe("browser action settings", () => {
  it("never gives up after 0 s, which would wait forever", () => {
    for (const node of [waitNode, assertNode]) {
      expect(node.params.safeParse({ timeoutMs: 0 }).success).toBe(false);
      expect(
        node.params.safeParse({ mode: "network-idle", text: "Saved", timeoutMs: 500 }).success,
      ).toBe(true);
    }
  });

  it("uploads only a chosen file inside the project", () => {
    const problem = (file: string) => {
      const parsed = parseParams(uploadNode.params, { target: "Avatar", file });
      return parsed.ok ? null : parsed.problem;
    };
    expect(problem("assets/photo.png")).toBeNull();
    expect(problem("project:assets/photo.png")).toBeNull();
    expect(problem("")).toBe("Choose a file");
    expect(problem("../secret.txt")).toBe("Choose a file inside the project folder");
    expect(problem("assets/../../secret.txt")).toBe("Choose a file inside the project folder");
    expect(problem("/etc/passwd")).toBe("Choose a file inside the project folder");
    expect(problem("C:\\Users\\me\\a.txt")).toBe("Choose a file inside the project folder");
  });

  it("asks what to wait for instead of waiting on nothing", () => {
    const problem = (params: Record<string, unknown>) => {
      const parsed = parseParams(waitNode.params, params);
      return parsed.ok ? null : parsed.problem;
    };
    expect(problem({ mode: "element", target: " " })).toBe("Pick what to wait for");
    expect(problem({ mode: "url", url: "" })).toBe("Add an address pattern");
    expect(problem({ mode: "url", url: "*/done*" })).toBeNull();
    expect(problem({ mode: "network-idle" })).toBeNull();
    expect(problem({ mode: "ms", ms: 500 })).toBeNull();
  });

  it("checks a drag path before the run", () => {
    const problem = (path: string) => {
      const parsed = parseParams(dragNode.params, { target: "Card", mode: "path", path });
      return parsed.ok ? null : parsed.problem;
    };
    expect(problem("0, 0\n40, 40")).toBeNull();
    expect(problem("100, 0\nnot a point")).toBe("Line 2 of the path isn't two numbers");
    expect(problem("")).toBe("Add points to the path");
  });

  it("asks for the option to choose", () => {
    expect(parseParams(selectNode.params, { target: "Plan", value: "" })).toEqual({
      ok: false,
      problem: "Choose an option",
    });
    expect(parseParams(selectNode.params, { target: "Plan", value: "pro" }).ok).toBe(true);
  });

  it("asks for the text to type instead of clearing the field", () => {
    const problem = (text: string) => {
      const parsed = parseParams(typeNode.params, { target: "Name", text });
      return parsed.ok ? null : parsed.problem;
    };
    expect(problem("")).toBe("Add text to type");
    expect(problem(`${PAUSE_MARK}${PAUSE_MARK}`)).toBe("Add text to type");
    expect(problem(`tom${PAUSE_MARK}smith`)).toBeNull();
  });

  it("refuses to check for blank text, which every page contains", () => {
    expect(parseParams(assertNode.params, { text: " " })).toEqual({
      ok: false,
      problem: "Add the text to look for",
    });
  });
});

describe("every browser action", () => {
  const firstProblem = (type: string, params: Record<string, unknown> = {}) => {
    const def = must(getNodeDef(type), type);
    const parsed = parseParams(def.params, { ...defaultParams(type), ...params });
    return parsed.ok ? null : parsed.problem;
  };

  it("asks a new action only for what the user has to fill in", () => {
    expect(Object.fromEntries(browserNodes.map((n) => [n.type, firstProblem(n.type)]))).toEqual({
      "browser.session": null,
      "browser.navigate": null,
      "browser.click": "Element is missing",
      "browser.type": "Element is missing",
      "browser.hover": "Element is missing",
      "browser.drag": "Element to drag is missing",
      "browser.scroll": "Pick an element",
      "browser.select": "Element is missing",
      "browser.upload": "File field is missing",
      "browser.press": null,
      "browser.wait": "Pick what to wait for",
      "browser.assert": "Add the text to look for",
      "browser.hoist": "Element is missing",
      "browser.setState": null,
      "browser.mockRoute": null,
      "browser.freezeClock": null,
      "browser.hideElements": null,
      "browser.chapter": null,
      "browser.overlayHtml": null,
    });
  });

  it("asks where to scroll to in element mode", () => {
    expect(firstProblem("browser.scroll", { mode: "element", target: "" })).toBe("Pick an element");
    expect(firstProblem("browser.scroll", { mode: "element", target: "Pricing" })).toBeNull();
    expect(firstProblem("browser.scroll", { mode: "by", target: "" })).toBeNull();
  });

  it("keeps numbers inside their limits", () => {
    expect(firstProblem("browser.scroll", { mode: "by", deltaY: 20_001 })).toBe(
      "Distance is too large",
    );
    expect(firstProblem("browser.wait", { mode: "ms", ms: 600_001 })).toBe("Time is too large");
    expect(firstProblem("browser.drag", { target: "Card", mode: "by", dx: -10_001 })).toBe(
      "Across is too small",
    );
    expect(firstProblem("browser.session", { viewportWidth: 10 })).toMatch(/is too small$/);
    expect(firstProblem("browser.click", { target: "Save", after: -1 })).toMatch(/is too small$/);
  });

  it("fills variables into text settings before a run", () => {
    const go = parseNodeParams(
      navigateNode,
      { params: { url: "{{Site address}}/pricing" } },
      { "Site address": "https://example.com" },
    );
    expect(go.url).toBe("https://example.com/pricing");
    const typed = parseNodeParams(
      typeNode,
      { params: { target: "{{Field}}", text: "Hello {{Name}}" } },
      { Field: "Name", Name: "Ada" },
    );
    expect([typed.target, typed.text]).toEqual(["Name", "Hello Ada"]);
  });

  it("draws a card for every action with its defaults", () => {
    for (const n of browserNodes) {
      const rows = nodeCard(n.type);
      expect({ type: n.type, rows: rows.length }).not.toEqual({ type: n.type, rows: 0 });
      expect(rows.filter((r) => !r.text.trim())).toEqual([]);
    }
  });
});
