import type { Workflow } from "../schema/workflow";
import { GraphBuilder } from "./build";
import { findById } from "../lists";

type TemplateInfo = {
  id: string;
  title: string;
  description: string;
  icon: string;
  build: (name: string) => Workflow;
};

type Step = { id: string; type: string; params?: Record<string, unknown>; label?: string };

type TakeOptions = { camera?: Record<string, unknown>; row?: number; prefix?: string };

function recordTake(
  g: GraphBuilder,
  steps: Step[],
  { camera = {}, row = 0, prefix = "" }: TakeOptions = {},
): string {
  const ids = steps.map((step, x) =>
    g.node(`${prefix}${step.id}`, step.type, x, row, step.params, step.label),
  );
  const rec = g.node(`${prefix}record`, "capture.record", steps.length, row);
  g.chain([...ids, rec]);
  const cam = g.node(`${prefix}camera`, "camera.autoZoom", steps.length, row + 1, camera);
  g.link(rec, cam, "recording", "recording");
  return cam;
}

function exportVideo(
  g: GraphBuilder,
  from: string,
  x: number,
  y: number,
  params: Record<string, unknown> = {},
): void {
  g.node("render", "output.render", x, y, params);
  g.link(from, "render", "clip", "timeline");
}

const session: Step = { id: "session", type: "browser.session" };

const open = (url = "{{Site address}}", after?: number): Step => ({
  id: "open",
  type: "browser.navigate",
  params: after === undefined ? { url } : { url, after },
});

const addTodo = (id: string, text: string, after: number): Step => ({
  id,
  type: "browser.type",
  params: { target: "What needs to be done?", text, pressEnter: true, cluster: "add", after },
  label: "New todo",
});

function todoMvcSample(name: string): Workflow {
  const g = new GraphBuilder();
  const cam = recordTake(g, [
    session,
    open("{{Site address}}/#/", 900),
    addTodo("todo1", "Record a real product flow", 500),
    addTodo("todo2", "Auto-zoom on every click", 500),
    addTodo("todo3", "Render 16:9 and 9:16", 1200),
    {
      id: "complete",
      type: "browser.click",
      params: { target: "Toggle Todo", targetRole: "checkbox", cluster: "done", after: 1100 },
      label: "Complete",
    },
    {
      id: "filter",
      type: "browser.click",
      params: { target: "Completed", targetRole: "link", cluster: "filter", after: 1500 },
      label: "Completed filter",
    },
  ]);
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

  exportVideo(g, seq, 9, 2, {
    variants: ["16:9", "9:16"],
    codec: "h264",
    reframe: "follow",
    fileName: "{{Project name}}",
  });
  g.node("note", "core.note", 0, 2, {
    text: "Sample project.\n\n1. Press Run. Demoflow records the flow in a hidden browser.\n2. The preview updates with the real footage, camera moves and cursor.\n3. Finished videos appear in Outputs.",
  });

  return g.build({ name, variables: { "Site address": "https://demo.playwright.dev/todomvc" } });
}

function featureLaunch(name: string): Workflow {
  const g = new GraphBuilder();
  const cam = recordTake(g, [
    session,
    open(),
    {
      id: "cta",
      type: "browser.click",
      params: { target: "Get started", after: 1500 },
      label: "Get started",
    },
  ]);
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
  exportVideo(g, seq, 5, 2);
  return g.build({
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
  exportVideo(g, seq, 4, 1, { variants: ["16:9", "1:1"] });
  return g.build({ name, variables: { "Product name": "Acme", Version: "v2.4" } });
}

function onboarding(name: string): Workflow {
  const g = new GraphBuilder();
  const cam = recordTake(g, [
    session,
    open(),
    { id: "chapter1", type: "browser.chapter", params: { title: "Create your first item" } },
    {
      id: "type",
      type: "browser.type",
      params: { target: "What needs to be done?", text: "Invite the team", pressEnter: true },
      label: "New item",
    },
    { id: "chapter2", type: "browser.chapter", params: { title: "Mark it done" } },
    {
      id: "done",
      type: "browser.click",
      params: { target: "css=.todo-list li:nth-child(1) .toggle", after: 1400 },
      label: "Done",
    },
  ]);
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
  exportVideo(g, caps, 8, 3);
  return g.build({ name, variables: { "Site address": "https://demo.playwright.dev/todomvc" } });
}

function socialTeaser(name: string): Workflow {
  const g = new GraphBuilder();
  const cam = recordTake(
    g,
    [
      { ...session, params: { viewportWidth: 1440, viewportHeight: 900 } },
      open(),
      {
        id: "type",
        type: "browser.type",
        params: {
          target: "What needs to be done?",
          text: "Ship the teaser",
          pressEnter: true,
          after: 1500,
        },
      },
    ],
    { camera: { speed: 1.5 } },
  );
  const hook = g.node("hook", "fx.remocn.kinetic-center-build", 1, 2, { clipDuration: 60 });
  const seq = g.node("sequence", "timeline.sequence", 4, 2, { transition: "remocn/push-through" });
  g.link(hook, seq, "clip", "in1").link(cam, seq, "clip", "in2");
  const music = g.node("music", "audio.music", 4, 3, { volume: 0.4 });
  const mix = g.node("mix", "audio.mix", 5, 2);
  g.link(seq, mix).link(music, mix, "audio", "sounds");
  exportVideo(g, mix, 6, 2, { variants: ["9:16"], reframe: "follow" });
  return g.build({
    name,
    settings: { width: 1080, height: 1920 },
    variables: { "Site address": "https://demo.playwright.dev/todomvc" },
  });
}

function beforeAfter(name: string): Workflow {
  const g = new GraphBuilder();
  const take = (prefix: string, row: number, url: string) => {
    const cam = recordTake(g, [session, open(url, 2500)], {
      camera: { zoom: false },
      row,
      prefix: `${prefix}-`,
    });
    const label = g.node(`${prefix}-label`, "fx.core.lower-third", 3, row + 1, {
      title: prefix === "before" ? "Before" : "After",
      subtitle: url,
    });
    const stack = g.node(`${prefix}-stack`, "timeline.stack", 4, row, { duration: "first" });
    g.link(cam, stack, "clip", "in1").link(label, stack, "clip", "in2");
    return stack;
  };
  const before = take("before", 0, "{{Before address}}");
  const after = take("after", 5, "{{After address}}");
  const wipe = g.node("wipe", "transition.core.wipe", 5, 3, { durationInFrames: 24 });
  g.link(before, wipe, "clip", "a").link(after, wipe, "clip", "b");
  exportVideo(g, wipe, 6, 3);
  return g.build({
    name,
    variables: { "Before address": "https://example.com", "After address": "https://example.org" },
  });
}

function blank(name: string): Workflow {
  const g = new GraphBuilder();
  g.node("note", "core.note", 0, 0, {
    text: "Drag nodes in from the list on the left, or press Ctrl+K.",
  });
  return g.build({ name });
}

export const TEMPLATES: TemplateInfo[] = [
  {
    id: "blank",
    title: "Blank",
    description: "An empty canvas to build from scratch.",
    icon: "file",
    build: blank,
  },
  {
    id: "todomvc",
    title: "Sample: to-do app",
    description:
      "Records a sample to-do app with auto zoom and an opening title, in wide and tall shapes.",
    icon: "list-checks",
    build: todoMvcSample,
  },
  {
    id: "feature-launch",
    title: "Feature launch",
    description: "A bold headline, a recorded flow with a spotlight, then a call to action.",
    icon: "rocket",
    build: featureLaunch,
  },
  {
    id: "changelog",
    title: "Changelog clip",
    description: "A title, a terminal session and a chart with a name label.",
    icon: "scroll-text",
    build: changelogClip,
  },
  {
    id: "onboarding",
    title: "Onboarding walkthrough",
    description: "Chapters, highlight boxes, a pinned tooltip and captions.",
    icon: "map",
    build: onboarding,
  },
  {
    id: "social-teaser",
    title: "Social teaser (9:16)",
    description: "A vertical opener, a recording that follows the action, and background music.",
    icon: "smartphone",
    build: socialTeaser,
  },
  {
    id: "before-after",
    title: "Before / after",
    description: "Two takes joined with a wipe.",
    icon: "columns-2",
    build: beforeAfter,
  },
];

export function getTemplate(templateId: string): TemplateInfo {
  const template = findById(TEMPLATES, templateId);
  if (!template) throw new Error(`There's no template called “${templateId}”.`);
  return template;
}
