import { describe, expect, it } from "vitest";
import {
  connectionProblem,
  loginDependencies,
  runTargets,
  sessionChain,
  topoSort,
  validateWorkflow,
} from "./graph";
import { findById } from "./lists";
import type { Workflow } from "./schema/workflow";
import { getTemplate } from "./templates";
import { GraphBuilder } from "./templates/build";
import { must } from "./test-fixtures";

const todoMvc = () => getTemplate("todomvc").build("TodoMVC demo");

const build = (g: GraphBuilder) => g.build({ id: "x", name: "x" });

const disable = (wf: Workflow, id: string) => {
  must(findById(wf.nodes, id), id).disabled = true;
};

describe("graph", () => {
  it("topologically sorts the sample", () => {
    const wf = todoMvc();
    const { sorted, cyclic } = topoSort(wf);
    const order = sorted.map((n) => n.id);
    expect(cyclic).toEqual([]);
    expect(order.indexOf("session")).toBeLessThan(order.indexOf("record"));
    expect(order.indexOf("record")).toBeLessThan(order.indexOf("camera"));
    expect(order.indexOf("sequence")).toBeLessThan(order.indexOf("render"));
    expect(order).not.toContain("note");
  });

  it("finds the action chain a capture node closes", () => {
    const chain = sessionChain(todoMvc(), "record").map((n) => n.id);
    expect(chain).toEqual(["session", "open", "todo1", "todo2", "todo3", "complete", "filter"]);
  });

  it("validates the sample without errors", () => {
    const errors = validateWorkflow(todoMvc()).filter((i) => i.level === "error");
    expect(errors).toEqual([]);
  });

  it("checks a new connection the same way for the canvas and the API", () => {
    const wf = todoMvc();
    const edge = { source: "camera", sourceHandle: "clip", target: "camera", targetHandle: "clip" };
    expect(connectionProblem(wf, edge)?.message).toBe("A node can't connect to itself.");
    expect(connectionProblem(wf, { ...edge, source: "gone" })?.notFound).toBe(true);
    expect(connectionProblem(wf, { ...edge, target: "record", targetHandle: "nope" })).toEqual({
      notFound: false,
      message: expect.stringContaining("has no input “nope”"),
    });
    expect(
      connectionProblem(wf, { ...edge, target: "open", targetHandle: "session" })?.message,
    ).toBe("A clip can't connect to a browser action input.");
  });

  it("explains an action chain that doesn't start from Open browser", () => {
    const g = new GraphBuilder();
    g.node("go", "browser.navigate", 1, 0, { url: "https://example.com" });
    g.node("rec", "capture.record", 2, 0);
    g.node("lone", "capture.record", 3, 0);
    g.chain(["go", "rec"]);
    const wf = build(g);
    expect(() => sessionChain(wf, "rec")).toThrow(
      "The actions starting at “Go to page” don't start from an Open browser node.",
    );
    expect(() => sessionChain(wf, "lone")).toThrow(
      "Connect Open browser and its actions to this node's “Previous” input.",
    );
  });

  it("names a variable the project doesn't have", () => {
    const g = new GraphBuilder();
    g.node("session", "browser.session", 0, 0);
    g.node("go", "browser.navigate", 1, 0, { url: "{{Site address}}/pricing" });
    g.node("rec", "capture.record", 2, 0);
    g.chain(["session", "go", "rec"]);
    const wf = build(g);
    const problems = () => validateWorkflow(wf).filter((i) => i.nodeId === "go");
    expect(problems().map((i) => i.message)).toEqual([
      "“Site address” isn't one of the project's variables. Add it under Variables in the project settings, or change this setting.",
    ]);
    wf.variables = { "Site address": "https://example.com" };
    expect(problems()).toEqual([]);
  });

  it("warns that a reload before the first page does nothing", () => {
    const reloads = (beforePage: boolean) => {
      const g = new GraphBuilder();
      g.node("session", "browser.session", 0, 0);
      g.node("script", "browser.setState", 1, 0, { script: "1", reload: true });
      g.node("go", "browser.navigate", 2, 0, { url: "https://example.com" });
      g.node("rec", "capture.record", 3, 0);
      g.chain(beforePage ? ["session", "script", "go", "rec"] : ["session", "go", "script", "rec"]);
      return validateWorkflow(build(g)).filter((i) => i.nodeId === "script");
    };
    expect(reloads(true).map((i) => i.message)).toEqual([
      expect.stringContaining("there's nothing to reload"),
    ]);
    expect(reloads(false)).toEqual([]);
  });

  it("reports group and note settings that can't be read", () => {
    const g = new GraphBuilder();
    g.node("note", "core.note", 0, 0, { text: 5 });
    g.node("group", "core.group", 1, 0);
    const issues = validateWorkflow(build(g));
    expect(issues.map((i) => i.nodeId)).toEqual(["note"]);
  });

  it("rejects port type mismatches and cycles", () => {
    const g = new GraphBuilder();
    const a = g.node("a", "timeline.loop", 0, 0);
    const b = g.node("b", "timeline.loop", 1, 0);
    g.link(a, b).link(b, a);
    const rec = g.node("rec", "capture.record", 2, 0);
    g.link(a, rec, "clip", "session");
    const issues = validateWorkflow(build(g));
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
    const wf = build(g);
    disable(wf, off);
    expect(loginDependencies(wf, "rec", { SHOP: "sauce" })).toEqual(["login"]);
    expect(loginDependencies(wf, "rec", {})).toEqual([]);
    expect(loginDependencies(todoMvc(), "record", {})).toEqual([]);
  });

  it("matches sign-in names the way they are saved", () => {
    const g = new GraphBuilder();
    const s = g.node("s", "browser.session", 0, 0, { storageState: " default " });
    const rec = g.node("rec", "capture.record", 1, 0);
    g.chain([s, rec]);
    g.node("login", "capture.loginOnce", 0, 1, { secretName: "" });
    expect(loginDependencies(build(g), "rec", {})).toEqual(["login"]);
  });

  it("runs every export and the nodes that come after one", () => {
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
    const wf = build(g);
    expect(runTargets(wf).toSorted()).toEqual(["film", "gif", "web"]);
    disable(wf, gif);
    expect(runTargets(wf).toSorted()).toEqual(["film", "web"]);
  });

  it("runs everything when there is no export", () => {
    const g = new GraphBuilder();
    g.node("a", "media.import", 0, 0, { path: "a.mp4" });
    g.node("b", "media.faststart", 1, 0);
    expect(runTargets(build(g)).toSorted()).toEqual(["a", "b"]);
  });
});
