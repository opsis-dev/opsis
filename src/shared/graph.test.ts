import { describe, expect, it } from "vitest";
import { loginDependencies, runTargets, sessionChain, topoSort, validateWorkflow } from "./graph";
import { todoMvcSample } from "./templates";
import { GraphBuilder } from "./templates/build";

describe("graph", () => {
  it("topologically sorts the sample", () => {
    const wf = todoMvcSample();
    const { order, cyclic } = topoSort(wf);
    expect(cyclic).toEqual([]);
    expect(order.indexOf("session")).toBeLessThan(order.indexOf("record"));
    expect(order.indexOf("record")).toBeLessThan(order.indexOf("camera"));
    expect(order.indexOf("sequence")).toBeLessThan(order.indexOf("render"));
    expect(order).not.toContain("note");
  });

  it("finds the action chain a capture node closes", () => {
    const chain = sessionChain(todoMvcSample(), "record").map((n) => n.id);
    expect(chain).toEqual(["session", "open", "todo1", "todo2", "todo3", "complete", "filter"]);
  });

  it("validates the sample without errors", () => {
    const errors = validateWorkflow(todoMvcSample()).filter((i) => i.level === "error");
    expect(errors).toEqual([]);
  });

  it("rejects port type mismatches and cycles", () => {
    const g = new GraphBuilder();
    const a = g.node("a", "timeline.loop", 0, 0);
    const b = g.node("b", "timeline.loop", 1, 0);
    g.link(a, b).link(b, a);
    const rec = g.node("rec", "capture.record", 2, 0);
    g.link(a, rec, "clip", "session");
    const issues = validateWorkflow(g.build({ id: "x", name: "x" }));
    expect(issues.some((i) => i.message.includes("loops back"))).toBe(true);
    expect(issues.some((i) => i.message === "A clip can't connect to a browser action input")).toBe(
      true,
    );
  });

  it("makes a capture wait for the Login once node that saves its session's login", () => {
    const g = new GraphBuilder();
    const s = g.node("s", "browser.session", 0, 0, { storageState: "{{SHOP}}-login" });
    const nav = g.node("nav", "browser.navigate", 1, 0, { url: "https://shop.test" });
    const rec = g.node("rec", "capture.record", 2, 0);
    g.chain([s, nav, rec]);
    g.node("login", "capture.loginOnce", 0, 1, { secretName: "sauce-login" });
    g.node("other", "capture.loginOnce", 0, 2, { secretName: "admin" });
    const off = g.node("off", "capture.loginOnce", 0, 3, { secretName: "sauce-login" });
    const wf = g.build({ id: "x", name: "x" });
    wf.nodes.find((n) => n.id === off)!.disabled = true;
    expect(loginDependencies(wf, "rec", { SHOP: "sauce" })).toEqual(["login"]);
    expect(loginDependencies(wf, "rec", {})).toEqual([]);
    expect(loginDependencies(todoMvcSample(), "record", {})).toEqual([]);
  });

  it("runs every export and the steps that come after one", () => {
    const g = new GraphBuilder();
    const title = g.node("title", "fx.core.title", 0, 0);
    const film = g.node("film", "output.render", 1, 0);
    g.link(title, film, "clip", "timeline");
    const gif = g.node("gif", "media.gif", 2, 0);
    g.link(film, gif, "media", "media");
    const web = g.node("web", "media.faststart", 3, 0);
    g.link(gif, web, "media", "media");
    g.node("loose", "media.import", 0, 1, { path: "a.mp4" });
    g.node("tidy", "media.faststart", 1, 1);
    const wf = g.build({ id: "x", name: "x" });
    expect(runTargets(wf).toSorted()).toEqual(["film", "gif", "web"]);
    wf.nodes.find((n) => n.id === gif)!.disabled = true;
    expect(runTargets(wf).toSorted()).toEqual(["film", "web"]);
  });

  it("runs everything when there is no export", () => {
    const g = new GraphBuilder();
    g.node("a", "media.import", 0, 0, { path: "a.mp4" });
    g.node("b", "media.faststart", 1, 0);
    expect(runTargets(g.build({ id: "x", name: "x" })).toSorted()).toEqual(["a", "b"]);
  });
});
