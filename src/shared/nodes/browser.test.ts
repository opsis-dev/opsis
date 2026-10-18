import { describe, expect, it } from "vitest";
import { nodeCard } from "../test-fixtures";

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
