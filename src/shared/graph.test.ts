import { describe, expect, it } from "vitest";
import { connectionProblem, runTargets, topoSort, validateWorkflow } from "./graph";
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
    ).toBe("A clip can't connect to a browser session input.");
  });

  it("asks for the browser where an action chain doesn't start from Open browser", () => {
    const g = new GraphBuilder();
    g.node("go", "browser.navigate", 1, 0, { url: "https://example.com" });
    g.node("rec", "capture.record", 2, 0);
    g.node("lone", "capture.record", 3, 0);
    g.chain(["go", "rec"]);
    const errors = validateWorkflow(build(g)).filter((i) => i.level === "error");
    expect(errors.map((i) => [i.nodeId, i.message])).toEqual([
      ["go", "Connect something to “Browser”"],
      ["lone", "Connect something to “Browser”"],
    ]);
  });

  it("asks for another action when a sound's action is gone", () => {
    const g = new GraphBuilder();
    g.node("save", "browser.click", 0, 0, { target: "Save" });
    g.node("on", "audio.sfx", 1, 0, { src: "a.wav", atAction: "save" });
    g.node("gone", "audio.sfx", 2, 0, { src: "a.wav", atAction: "deleted" });
    const problems = validateWorkflow(build(g)).filter((i) =>
      i.message.includes("action this sound"),
    );
    expect(problems.map((i) => i.nodeId)).toEqual(["gone"]);
  });

  it("ignores a connection from a deleted node when counting a node's connections", () => {
    const g = new GraphBuilder();
    g.node("rec", "capture.record", 0, 0);
    const wf = build(g);
    wf.edges.push({
      id: "e",
      source: "gone",
      sourceHandle: "session",
      target: "rec",
      targetHandle: "session",
    });
    const messages = validateWorkflow(wf)
      .filter((i) => i.nodeId === "rec" || i.edgeId === "e")
      .map((i) => i.message);
    expect(messages).toContain("Connect something to “Browser”");
    expect(messages).toContain("A connection points at a deleted node");
    expect(messages.some((m) => m.includes("only take one connection"))).toBe(false);
  });

  it("warns a node fed by a skipped node that has nothing to pass on", () => {
    const g = new GraphBuilder();
    g.node("bg", "fx.core.solid", 0, 0);
    g.node("trim", "timeline.trim", 1, 0);
    g.node("speed", "timeline.speed", 2, 0);
    g.link("bg", "trim").link("trim", "speed");
    const wf = build(g);
    must(findById(wf.nodes, "bg"), "bg").disabled = true;
    const warn = (w: Workflow) =>
      validateWorkflow(w)
        .filter((i) => i.message.includes("is skipped"))
        .map((i) => i.nodeId);
    expect(warn(wf)).toEqual(["trim"]);
    must(findById(wf.nodes, "bg"), "bg").disabled = false;
    must(findById(wf.nodes, "trim"), "trim").disabled = true;
    expect(warn(wf)).toEqual([]);
  });

  it("stops two Sign in nodes from saving the same sign-in", () => {
    const g = new GraphBuilder();
    g.node("a", "browser.signIn", 0, 0, { url: "https://x.test/login", secretName: "acct" });
    g.node("b", "browser.signIn", 1, 0, { url: "https://x.test/login", secretName: "acct" });
    g.node("c", "browser.signIn", 2, 0, { url: "https://x.test/login", secretName: "other" });
    const clashes = validateWorkflow(build(g)).filter((i) =>
      i.message.includes("also saves the sign-in"),
    );
    expect(clashes.map((i) => i.nodeId)).toEqual(["b"]);
  });

  it("stops two exports from saving the same file", () => {
    const g = new GraphBuilder();
    g.node("one", "output.render", 0, 0, { fileName: "film" });
    g.node("two", "output.render", 1, 0, { fileName: "film" });
    g.node("wide", "output.render", 2, 0, { fileName: "film", variants: ["9:16"] });
    g.node("other", "output.render", 3, 0, { fileName: "teaser" });
    const clashes = validateWorkflow(build(g)).filter((i) => i.message.includes("also saves"));
    expect(clashes.map((i) => [i.nodeId, i.message])).toEqual([
      ["two", "“Export video” also saves “film.mp4”. Give one of them another file name."],
      ["wide", "“Export video” also saves “film.mp4”. Give one of them another file name."],
    ]);
  });

  it("names a variable the project doesn't have", () => {
    const g = new GraphBuilder();
    g.node("session", "browser.open", 0, 0);
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
      g.node("session", "browser.open", 0, 0);
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
    expect(
      issues.some((i) => i.message === "A clip can't connect to a browser session input"),
    ).toBe(true);
  });

  it("runs every export and the nodes that come after one", () => {
    const g = new GraphBuilder();
    const title = g.node("title", "fx.core.title", 0, 0);
    const film = g.node("film", "output.render", 1, 0);
    g.link(title, film);
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

  it("runs Sign in like any node of the chain when there is no export", () => {
    const g = new GraphBuilder();
    g.node("session", "browser.open", 0, 0);
    g.node("login", "browser.signIn", 1, 0, { secretName: "admin" });
    g.node("go", "browser.navigate", 2, 0, { url: "https://example.com" });
    g.node("rec", "capture.record", 3, 0);
    g.chain(["session", "login", "go", "rec"]);
    expect(runTargets(build(g))).toEqual(["login", "rec"]);
  });
});
