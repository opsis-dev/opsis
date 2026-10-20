import { describe, expect, it } from "vitest";
import { nodeCard } from "../test-fixtures";
import { assertNode, selectNode, uploadNode, waitNode } from "./browser";
import { parseParams } from "./params";

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

  it("asks for the option to choose", () => {
    expect(parseParams(selectNode.params, { target: "Plan", value: "" })).toEqual({
      ok: false,
      problem: "Choose an option",
    });
    expect(parseParams(selectNode.params, { target: "Plan", value: "pro" }).ok).toBe(true);
  });

  it("refuses to check for blank text, which every page contains", () => {
    expect(parseParams(assertNode.params, { text: " " })).toEqual({
      ok: false,
      problem: "Add the text to look for",
    });
  });
});
