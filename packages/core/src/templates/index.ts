import { findById } from "#core/lists";
import type { NodeIconName } from "#core/nodes/icon-name";
import type { Workflow } from "#core/schema/workflow";
import { GraphBuilder, type TakeStep } from "#core/templates/build";
import { PROJECT_NAME, SITE_ADDRESS, SITE_ADDRESS_VAR, varToken } from "#core/vars";

type TemplateInfo = {
  id: string;
  title: string;
  description: string;
  icon: NodeIconName;
  build: (name: string) => Workflow;
};

function exportVideo(
  g: GraphBuilder,
  from: string,
  x: number,
  y: number,
  params: Record<string, unknown> = {},
): void {
  g.node("render", "output.render", x, y, params);
  g.link(from, "render");
}

const TODO_SITE = "https://demo.playwright.dev/todomvc";
const EXAMPLE_SITE = "https://example.com";
const EXAMPLE_PRODUCT = "Acme";

const PRODUCT_NAME_VAR = "Product name";

const VERSION_VAR = "Version";

const BEFORE_ADDRESS_VAR = "Before address";

const AFTER_ADDRESS_VAR = "After address";

const session: TakeStep = { id: "session", type: "browser.open" };

const open = (url = SITE_ADDRESS, after?: number): TakeStep => ({
  id: "open",
  type: "browser.navigate",
  params: after === undefined ? { url } : { url, after },
});

const addTodo = (id: string, text: string, after: number): TakeStep => ({
  id,
  type: "browser.type",
  params: { target: "What needs to be done?", text, pressEnter: true, cluster: "add", after },
  label: "New todo",
});

function todoMvcSample(name: string): Workflow {
  const g = new GraphBuilder();
  const chips: TakeStep = {
    id: "chips",
    type: "cursor.keystrokeChips",
    params: { position: "bottom", typed: true, chords: true },
  };
  const cam = g.recordTake(
    [
      session,
      open(`${SITE_ADDRESS}/#/`, 900),
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
    ],
    { style: [chips] },
  );

  const bg = g.node("intro-bg", "fx.remocn.shader-mesh-gradient", 4, 2, { clipDuration: 75 });
  const title = g.node("intro-title", "fx.remocn.soft-blur-in", 4, 3, {
    text: "TodoMVC in 20 seconds",
    color: "#fafafa",
    fontSize: 104,
    clipDuration: 75,
  });
  const intro = g.node("intro", "timeline.stack", 5, 2, { duration: "first" });
  g.clipsInto(intro, [bg, title]);

  const outro = g.node("outro", "fx.core.title", 5, 4, {
    eyebrow: "Made with",
    title: "Opsis",
    subtitle: SITE_ADDRESS,
    clipDuration: 75,
  });

  const seq = g.node("sequence", "timeline.sequence", 8, 2, {
    transition: "remocn/whip-pan",
    transitionDuration: 18,
  });
  g.clipsInto(seq, [intro, cam, outro]);

  exportVideo(g, seq, 9, 2, {
    variants: ["16:9", "9:16"],
    codec: "h264",
    reframe: "follow",
    fileName: PROJECT_NAME,
  });
  g.node("note", "core.note", 0, 2, {
    text: "Sample project.\n\n1. Press Run. Opsis records the flow in a hidden browser.\n2. The preview updates with the real footage, camera moves and cursor.\n3. Finished videos appear in Outputs.",
  });

  return g.build({ name, variables: { [SITE_ADDRESS_VAR]: TODO_SITE } });
}

function featureLaunch(name: string): Workflow {
  const g = new GraphBuilder();
  const cam = g.recordTake(
    [
      session,
      open(),
      {
        id: "cta",
        type: "browser.click",
        params: { target: "Get started", after: 1500 },
        label: "Get started",
      },
    ],
    { style: [{ id: "spotlight", type: "cursor.spotlight", params: { match: "Get started" } }] },
  );
  const intro = g.node("headline", "fx.core.kinetic-headline", 1, 2, {
    text: `Introducing *${varToken(PRODUCT_NAME_VAR)}*`,
    background: "#09090b",
  });
  const outro = g.node("outro", "fx.core.title", 2, 3, {
    title: `Try ${varToken(PRODUCT_NAME_VAR)} today`,
    subtitle: SITE_ADDRESS,
  });
  const seq = g.node("sequence", "timeline.sequence", 4, 2, { transition: "remocn/zoom-blur" });
  g.clipsInto(seq, [intro, cam, outro]);
  exportVideo(g, seq, 5, 2);
  return g.build({
    name,
    variables: { [SITE_ADDRESS_VAR]: EXAMPLE_SITE, [PRODUCT_NAME_VAR]: EXAMPLE_PRODUCT },
  });
}

function changelogClip(name: string): Workflow {
  const g = new GraphBuilder();
  const title = g.node("title", "fx.core.title", 0, 0, {
    eyebrow: "Changelog",
    title: `What's new in ${varToken(VERSION_VAR)}`,
  });
  const term = g.node("terminal", "fx.remocn.terminal-simulator", 0, 1, { clipDuration: 150 });
  const chart = g.node("chart", "fx.remocn.animated-bar-chart", 0, 2, { clipDuration: 120 });
  const lower = g.node("credit", "fx.core.lower-third", 1, 3, {
    title: `${varToken(PRODUCT_NAME_VAR)} ${varToken(VERSION_VAR)}`,
    subtitle: "Out now",
  });
  const bg = g.node("bg", "fx.core.gradient", 1, 2);
  const chartScene = g.node("chart-scene", "timeline.stack", 2, 2, { duration: "last" });
  g.clipsInto(chartScene, [bg, chart, lower]);
  const seq = g.node("sequence", "timeline.sequence", 3, 1, { transition: "core/slide" });
  g.clipsInto(seq, [title, term, chartScene]);
  exportVideo(g, seq, 4, 1, { variants: ["16:9", "1:1"] });
  return g.build({
    name,
    variables: { [PRODUCT_NAME_VAR]: EXAMPLE_PRODUCT, [VERSION_VAR]: "v2.4" },
  });
}

function onboarding(name: string): Workflow {
  const g = new GraphBuilder();
  const cam = g.recordTake(
    [
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
    ],
    {
      style: [
        { id: "highlight", type: "cursor.highlightBox" },
        { id: "tooltip", type: "text.tooltip", params: { match: "Done", text: "Tick it off" } },
      ],
    },
  );
  const script = g.node("script", "text.captionsFromText", 4, 3, {
    script: "Create an item. Then tick it off. That's it.",
    startMs: 1500,
  });
  const caps = g.node("captions", "text.captions", 7, 3);
  g.link(script, caps, "captions", "captions").link(cam, caps);
  exportVideo(g, caps, 8, 3);
  return g.build({ name, variables: { [SITE_ADDRESS_VAR]: TODO_SITE } });
}

function socialTeaser(name: string): Workflow {
  const g = new GraphBuilder();
  const cam = g.recordTake(
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
  g.clipsInto(seq, [hook, cam]);
  const music = g.node("music", "audio.music", 4, 3, { volume: 0.4 });
  const mix = g.node("mix", "audio.mix", 5, 2);
  g.link(seq, mix).link(music, mix, "audio", "sounds");
  exportVideo(g, mix, 6, 2, { variants: ["9:16"], reframe: "follow" });
  return g.build({
    name,
    settings: { width: 1080, height: 1920 },
    variables: { [SITE_ADDRESS_VAR]: TODO_SITE },
  });
}

function beforeAfter(name: string): Workflow {
  const g = new GraphBuilder();
  const take = (prefix: string, row: number, url: string) => {
    const cam = g.recordTake([session, open(url, 2500)], {
      camera: { zoom: false },
      row,
      prefix: `${prefix}-`,
    });
    const label = g.node(`${prefix}-label`, "fx.core.lower-third", 3, row + 1, {
      title: prefix === "before" ? "Before" : "After",
      subtitle: url,
    });
    const stack = g.node(`${prefix}-stack`, "timeline.stack", 4, row, { duration: "first" });
    g.clipsInto(stack, [cam, label]);
    return stack;
  };
  const before = take("before", 0, varToken(BEFORE_ADDRESS_VAR));
  const after = take("after", 5, varToken(AFTER_ADDRESS_VAR));
  const wipe = g.node("wipe", "transition.core.wipe", 5, 3, { durationInFrames: 24 });
  g.link(before, wipe, "clip", "a").link(after, wipe, "clip", "b");
  exportVideo(g, wipe, 6, 3);
  return g.build({
    name,
    variables: {
      [BEFORE_ADDRESS_VAR]: EXAMPLE_SITE,
      [AFTER_ADDRESS_VAR]: "https://example.org",
    },
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
