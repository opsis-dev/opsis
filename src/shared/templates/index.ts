import type { Workflow } from "../schema/workflow";
import { GraphBuilder } from "./build";

export type TemplateInfo = {
  id: string;
  title: string;
  description: string;
  build: (name: string) => Workflow;
};

const id = () => `wf_${Math.random().toString(36).slice(2, 10)}`;

export function todoMvcSample(name = "TodoMVC demo"): Workflow {
  const g = new GraphBuilder();
  const s = g.node("session", "browser.session", 0, 0, {
    viewportWidth: 1920,
    viewportHeight: 1080,
    dpr: 2,
  });
  const nav = g.node("open", "browser.navigate", 1, 0, { url: "{{Site address}}/#/", after: 900 });
  const t1 = g.node("todo1", "browser.type", 2, 0, {
    target: "What needs to be done?",
    text: "Record a real product flow",
    pressEnter: true,
    cluster: "add",
    after: 500,
    label: "New todo",
  });
  const t2 = g.node("todo2", "browser.type", 3, 0, {
    target: "What needs to be done?",
    text: "Auto-zoom on every click",
    pressEnter: true,
    cluster: "add",
    after: 500,
    label: "New todo",
  });
  const t3 = g.node("todo3", "browser.type", 4, 0, {
    target: "What needs to be done?",
    text: "Render 16:9 and 9:16",
    pressEnter: true,
    cluster: "add",
    after: 1200,
    label: "New todo",
  });
  const done = g.node("complete", "browser.click", 5, 0, {
    target: "Toggle Todo",
    targetRole: "checkbox",
    label: "Complete",
    cluster: "done",
    after: 1100,
  });
  const filter = g.node("filter", "browser.click", 6, 0, {
    target: "Completed",
    targetRole: "link",
    label: "Completed filter",
    cluster: "filter",
    after: 1500,
  });
  const rec = g.node("record", "capture.record", 7, 0, { fps: 30 });
  g.chain([s, nav, t1, t2, t3, done, filter, rec]);

  const cam = g.node("camera", "camera.autoZoom", 7, 1, {
    maxScale: 1.74,
    hold: 1300,
    backdrop: "glaze",
    motionBlur: true,
  });
  g.link(rec, cam, "recording", "recording");
  const chips = g.node("chips", "cursor.keystrokeChips", 7, 2, {
    position: "bottom",
    typed: true,
    chords: true,
  });
  g.link(cam, chips);

  const bg = g.node("intro-bg", "fx.remocn.shader-mesh-gradient", 4, 2, { clipDuration: 75 });
  const title = g.node("intro-title", "fx.remocn.soft-blur-in", 4, 3, {
    text: "TodoMVC in 20 seconds",
    color: "#fafafa",
    fontSize: 104,
    clipDuration: 75,
  });
  const intro = g.node("intro", "timeline.stack", 5, 2, { duration: "first" });
  g.link(bg, intro, "clip", "in1").link(title, intro, "clip", "in2");

  const outro = g.node("outro", "fx.core.title", 5, 4, {
    eyebrow: "Made with",
    title: "Demoflow",
    subtitle: "{{Site address}}",
    clipDuration: 75,
  });

  const seq = g.node("sequence", "timeline.sequence", 8, 2, {
    transition: "remocn/whip-pan",
    transitionDuration: 18,
  });
  g.link(intro, seq, "clip", "in1").link(chips, seq, "clip", "in2").link(outro, seq, "clip", "in3");

  const out = g.node("render", "output.render", 9, 2, {
    variants: ["16:9", "9:16"],
    codec: "h264",
    reframe: "follow",
    fileName: "{{Project name}}",
  });
  g.link(seq, out, "clip", "timeline");
  g.node("note", "core.note", 0, 2, {
    text: "Sample project.\n\n1. Press Run. Demoflow records the flow in a hidden browser.\n2. The preview updates with the real footage, camera moves and cursor.\n3. Finished videos appear in Outputs.",
  });

  return g.build({
    id: id(),
    name,
    variables: { "Site address": "https://demo.playwright.dev/todomvc" },
  });
}

function featureLaunch(name: string): Workflow {
  const g = new GraphBuilder();
  const s = g.node("session", "browser.session", 0, 0);
  const nav = g.node("open", "browser.navigate", 1, 0, { url: "{{Site address}}" });
  const click = g.node("cta", "browser.click", 2, 0, {
    target: "Get started",
    label: "Get started",
    after: 1500,
  });
  const rec = g.node("record", "capture.record", 3, 0);
  g.chain([s, nav, click, rec]);
  const cam = g.node("camera", "camera.autoZoom", 3, 1);
  g.link(rec, cam, "recording", "recording");
  const spot = g.node("spotlight", "cursor.spotlight", 3, 2, { match: "Get started" });
  g.link(cam, spot);
  const intro = g.node("headline", "fx.core.kinetic-headline", 1, 2, {
    text: "Introducing *{{Product name}}*",
    background: "#09090b",
  });
  const outro = g.node("outro", "fx.core.title", 2, 3, {
    title: "Try {{Product name}} today",
    subtitle: "{{Site address}}",
  });
  const seq = g.node("sequence", "timeline.sequence", 4, 2, { transition: "remocn/zoom-blur" });
  g.link(intro, seq, "clip", "in1").link(spot, seq, "clip", "in2").link(outro, seq, "clip", "in3");
  const out = g.node("render", "output.render", 5, 2);
  g.link(seq, out, "clip", "timeline");
  return g.build({
    id: id(),
    name,
    variables: { "Site address": "https://example.com", "Product name": "Acme" },
  });
}

function changelogClip(name: string): Workflow {
  const g = new GraphBuilder();
  const title = g.node("title", "fx.core.title", 0, 0, {
    eyebrow: "Changelog",
    title: "What's new in {{Version}}",
  });
  const term = g.node("terminal", "fx.remocn.terminal-simulator", 0, 1, { clipDuration: 150 });
  const chart = g.node("chart", "fx.remocn.animated-bar-chart", 0, 2, { clipDuration: 120 });
  const lower = g.node("credit", "fx.core.lower-third", 1, 3, {
    title: "{{Product name}} {{Version}}",
    subtitle: "Out now",
  });
  const bg = g.node("bg", "fx.core.gradient", 1, 2);
  const chartScene = g.node("chart-scene", "timeline.stack", 2, 2, { duration: "last" });
  g.link(bg, chartScene, "clip", "in1")
    .link(chart, chartScene, "clip", "in2")
    .link(lower, chartScene, "clip", "in3");
  const seq = g.node("sequence", "timeline.sequence", 3, 1, { transition: "core/slide" });
  g.link(title, seq, "clip", "in1")
    .link(term, seq, "clip", "in2")
    .link(chartScene, seq, "clip", "in3");
  const out = g.node("render", "output.render", 4, 1, { variants: ["16:9", "1:1"] });
  g.link(seq, out, "clip", "timeline");
  return g.build({ id: id(), name, variables: { "Product name": "Acme", Version: "v2.4" } });
}

function onboarding(name: string): Workflow {
  const g = new GraphBuilder();
  const s = g.node("session", "browser.session", 0, 0);
  const nav = g.node("open", "browser.navigate", 1, 0, { url: "{{Site address}}" });
  const ch1 = g.node("chapter1", "browser.chapter", 2, 0, { title: "Create your first item" });
  const type = g.node("type", "browser.type", 3, 0, {
    target: "What needs to be done?",
    text: "Invite the team",
    pressEnter: true,
    label: "New item",
  });
  const ch2 = g.node("chapter2", "browser.chapter", 4, 0, { title: "Mark it done" });
  const done = g.node("done", "browser.click", 5, 0, {
    target: "css=.todo-list li:nth-child(1) .toggle",
    label: "Done",
    after: 1400,
  });
  const rec = g.node("record", "capture.record", 6, 0);
  g.chain([s, nav, ch1, type, ch2, done, rec]);
  const cam = g.node("camera", "camera.autoZoom", 6, 1);
  g.link(rec, cam, "recording", "recording");
  const hl = g.node("highlight", "cursor.highlightBox", 6, 2);
  g.link(cam, hl);
  const tip = g.node("tooltip", "text.tooltip", 6, 3, { match: "Done", text: "Tick it off" });
  g.link(hl, tip);
  const script = g.node("script", "text.captionsFromText", 4, 3, {
    script: "Create an item. Then tick it off. That's it.",
    startMs: 1500,
  });
  const caps = g.node("captions", "text.captions", 7, 3);
  g.link(script, caps, "captions", "captions").link(tip, caps, "clip", "clip");
  const out = g.node("render", "output.render", 8, 3);
  g.link(caps, out, "clip", "timeline");
  return g.build({
    id: id(),
    name,
    variables: { "Site address": "https://demo.playwright.dev/todomvc" },
  });
}

function socialTeaser(name: string): Workflow {
  const g = new GraphBuilder();
  const s = g.node("session", "browser.session", 0, 0, {
    viewportWidth: 1440,
    viewportHeight: 900,
  });
  const nav = g.node("open", "browser.navigate", 1, 0, { url: "{{Site address}}" });
  const t = g.node("type", "browser.type", 2, 0, {
    target: "What needs to be done?",
    text: "Ship the teaser",
    pressEnter: true,
    after: 1500,
  });
  const rec = g.node("record", "capture.record", 3, 0);
  g.chain([s, nav, t, rec]);
  const cam = g.node("camera", "camera.autoZoom", 3, 1, { speed: 1.5 });
  g.link(rec, cam, "recording", "recording");
  const hook = g.node("hook", "fx.remocn.kinetic-center-build", 1, 2, { clipDuration: 60 });
  const seq = g.node("sequence", "timeline.sequence", 4, 2, { transition: "remocn/push-through" });
  g.link(hook, seq, "clip", "in1").link(cam, seq, "clip", "in2");
  const music = g.node("music", "audio.music", 4, 3, { volume: 0.4 });
  const mix = g.node("mix", "audio.mix", 5, 2);
  g.link(seq, mix).link(music, mix, "audio", "sounds");
  const out = g.node("render", "output.render", 6, 2, { variants: ["9:16"], reframe: "follow" });
  g.link(mix, out, "clip", "timeline");
  return g.build({
    id: id(),
    name,
    settings: { width: 1080, height: 1920 },
    variables: { "Site address": "https://demo.playwright.dev/todomvc" },
  });
}

function beforeAfter(name: string): Workflow {
  const g = new GraphBuilder();
  const take = (prefix: string, row: number, url: string) => {
    const s = g.node(`${prefix}-session`, "browser.session", 0, row);
    const nav = g.node(`${prefix}-open`, "browser.navigate", 1, row, { url, after: 2500 });
    const rec = g.node(`${prefix}-record`, "capture.record", 2, row);
    g.chain([s, nav, rec]);
    const cam = g.node(`${prefix}-camera`, "camera.autoZoom", 3, row, { zoom: false });
    g.link(rec, cam, "recording", "recording");
    const label = g.node(`${prefix}-label`, "fx.core.lower-third", 3, row + 1, {
      title: prefix === "before" ? "Before" : "After",
      subtitle: url,
    });
    const stack = g.node(`${prefix}-stack`, "timeline.stack", 4, row, { duration: "first" });
    g.link(cam, stack, "clip", "in1").link(label, stack, "clip", "in2");
    return stack;
  };
  const before = take("before", 0, "{{Before address}}");
  const after = take("after", 2, "{{After address}}");
  const wipe = g.node("wipe", "transition.core.wipe", 5, 1, { durationInFrames: 24 });
  g.link(before, wipe, "clip", "a").link(after, wipe, "clip", "b");
  const out = g.node("render", "output.render", 6, 1);
  g.link(wipe, out, "clip", "timeline");
  return g.build({
    id: id(),
    name,
    variables: { "Before address": "https://example.com", "After address": "https://example.org" },
  });
}

function blank(name: string): Workflow {
  const g = new GraphBuilder();
  g.node("note", "core.note", 0, 0, {
    text: "Drag nodes in from the list on the left, or press Ctrl+K.",
  });
  return g.build({ id: id(), name });
}

export const TEMPLATES: TemplateInfo[] = [
  {
    id: "blank",
    title: "Blank",
    description: "An empty canvas to build from scratch.",
    build: blank,
  },
  {
    id: "todomvc",
    title: "Sample: to-do app",
    description:
      "Records a sample to-do app with auto zoom and an opening title, in wide and tall shapes.",
    build: todoMvcSample,
  },
  {
    id: "feature-launch",
    title: "Feature launch",
    description: "A bold headline, a recorded flow with a spotlight, then a call to action.",
    build: featureLaunch,
  },
  {
    id: "changelog",
    title: "Changelog clip",
    description: "A title, a terminal session and a chart with a name label.",
    build: changelogClip,
  },
  {
    id: "onboarding",
    title: "Onboarding walkthrough",
    description: "Chapters, highlight boxes, a pinned tooltip and captions.",
    build: onboarding,
  },
  {
    id: "social-teaser",
    title: "Social teaser (9:16)",
    description: "A vertical opener, a recording that follows the action, and background music.",
    build: socialTeaser,
  },
  {
    id: "before-after",
    title: "Before / after",
    description: "Two takes joined with a wipe.",
    build: beforeAfter,
  },
];

export const getTemplate = (templateId: string) =>
  TEMPLATES.find((t) => t.id === templateId) ?? TEMPLATES[0];
