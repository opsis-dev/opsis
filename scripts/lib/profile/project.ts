import { type InputName, projectRef } from "./inputs";

type CatalogPort = { type: string; optional: boolean; many: boolean };

export type CatalogNode = {
  type: string;
  executes: boolean;
  virtual: boolean;
  hidden: boolean;
  inputs: Record<string, CatalogPort>;
  outputs: Record<string, CatalogPort>;
  defaults: Record<string, unknown>;
};

export type Catalog = { nodes: CatalogNode[] };

type WorkflowNode = {
  id: string;
  type: string;
  position: { x: number; y: number };
  params: Record<string, unknown>;
  label?: string;
  parentId?: string;
  size?: { width: number; height: number };
};

type WorkflowEdge = {
  id: string;
  source: string;
  sourceHandle: string;
  target: string;
  targetHandle: string;
};

export type Workflow = {
  version: 1;
  id: string;
  name: string;
  settings: Record<string, unknown>;
  variables: Record<string, string>;
  nodes: WorkflowNode[];
  edges: WorkflowEdge[];
  mix: null;
  markers: never[];
};

const SITE_VARIABLE = "Site address";

const COLUMN = 340;
const ROW = 260;
const ROW_LENGTH = 14;

const GROUP_PADDING = 60;

function ref(name: InputName): string {
  return projectRef(name);
}

const FILE_ELEMENTS: Record<string, Record<string, unknown>> = {
  "fx.text-mask-video": { src: ref("clip") },
  "fx.object-3d": { logo: { src: "app:placeholders/logo-mark.svg" } },
};

function workflowBuilder(catalog: Catalog) {
  const definitions = new Map(catalog.nodes.map((node) => [node.type, node]));
  const nodes: WorkflowNode[] = [];
  const edges: WorkflowEdge[] = [];
  let row = 0;
  let column = 0;
  const counts = new Map<string, number>();

  function definition(type: string): CatalogNode {
    const found = definitions.get(type);
    if (!found) throw new Error(`The catalogue has no ${type} node.`);
    return found;
  }

  function nextId(type: string): string {
    const count = (counts.get(type) ?? 0) + 1;
    counts.set(type, count);
    return `${type.replaceAll(/[^a-zA-Z0-9]+/g, "_")}_${count}`;
  }

  function section(title: string): void {
    if (column > 0 || nodes.length > 0) row += 1;
    column = 0;
    nodes.push({
      id: nextId("core.note"),
      type: "core.note",
      position: { x: 0, y: row * ROW },
      params: { ...definition("core.note").defaults, text: title },
    });
    column = 1;
  }

  function add(type: string, settings: Record<string, unknown> = {}): string {
    if (column >= ROW_LENGTH) {
      row += 1;
      column = 1;
    }
    const id = nextId(type);
    const output = type.startsWith("output.") ? { fileName: id } : {};
    nodes.push({
      id,
      type,
      position: { x: column * COLUMN, y: row * ROW },
      params: { ...definition(type).defaults, ...output, ...settings },
    });
    column += 1;
    return id;
  }

  function link(source: string, sourceHandle: string, target: string, targetHandle: string): void {
    edges.push({
      id: `${source}.${sourceHandle}->${target}.${targetHandle}`,
      source,
      sourceHandle,
      target,
      targetHandle,
    });
  }

  function chain(
    first: string,
    handle: string,
    types: [string, Record<string, unknown>?][],
  ): string {
    let previous = first;
    for (const [type, settings] of types) {
      const next = add(type, settings);
      link(previous, handle, next, handle);
      previous = next;
    }
    return previous;
  }

  function group(title: string, ids: string[]): void {
    const children = nodes.filter((node) => ids.includes(node.id));
    const xs = children.map((node) => node.position.x);
    const ys = children.map((node) => node.position.y);
    const origin = { x: Math.min(...xs) - GROUP_PADDING, y: Math.min(...ys) - GROUP_PADDING };
    const id = nextId("core.group");
    nodes.push({
      id,
      type: "core.group",
      position: origin,
      params: { ...definition("core.group").defaults, title },
      size: {
        width: Math.max(...xs) - origin.x + COLUMN,
        height: Math.max(...ys) - origin.y + ROW,
      },
    });
    for (const child of children) {
      child.parentId = id;
      child.position = { x: child.position.x - origin.x, y: child.position.y - origin.y };
    }
    nodes.sort((a, b) => Number(b.type === "core.group") - Number(a.type === "core.group"));
  }

  return { definition, section, add, link, chain, group, nodes, edges };
}

type Builder = ReturnType<typeof workflowBuilder>;

function render(builder: Builder, clipSource: string): string {
  const output = builder.add("output.render");
  builder.link(clipSource, "clip", output, "clip");
  return output;
}

function browserTake(builder: Builder): { recording: string; still: string } {
  builder.section("Browser: a session on the local site, with every action, recorded");
  const open = builder.add("browser.open");
  const session = builder.chain(open, "session", [
    ["browser.navigate"],
    ["browser.startRecording"],
    ["browser.freezeClock"],
    ["browser.mockRoute"],
    ["browser.setState"],
    ["browser.hideElements", { selectors: "#banner" }],
    ["browser.type", { target: "css=#name", text: "Ada" }],
    ["browser.click", { target: "css=#continue" }],
    ["browser.assert", { text: "Saved Ada" }],
    ["browser.wait", { target: "css=#status" }],
    ["browser.hover", { target: "css=#card" }],
    ["browser.drag", { target: "css=#card", to: "css=#drop" }],
    ["browser.select", { target: "css=#size", value: "large" }],
    ["browser.upload", { target: "css=#attachment", file: ref("upload") }],
    ["browser.hoist", { target: "css=#footer" }],
    ["browser.scroll", { target: "css=#footer" }],
    ["browser.press"],
    ["browser.chapter", { description: "The second page" }],
    ["browser.overlayHtml"],
    ["browser.navigate", { url: `{{${SITE_VARIABLE}}}/next` }],
  ]);
  const recording = builder.add("capture.record");
  builder.link(session, "session", recording, "session");
  const stillOpen = builder.add("browser.open");
  const stillPage = builder.chain(stillOpen, "session", [["browser.navigate"]]);
  const still = builder.add("capture.still");
  builder.link(stillPage, "session", still, "session");
  const signInOpen = builder.add("browser.open");
  builder.chain(signInOpen, "session", [
    ["browser.signIn", { url: `{{${SITE_VARIABLE}}}/login`, doneWhen: "url-change" }],
  ]);
  return { recording, still };
}

function recordingStyle(builder: Builder): string {
  const first = builder.add("camera.camera", { follows: "cursor" });
  return builder.chain(first, "style", [
    ["camera.frame", { background: "solid", color: "#0b0b10" }],
    ["camera.idleSpeedup"],
    ["camera.motionBlur"],
    ["cursor.style"],
    ["cursor.clickRipple"],
    ["cursor.keystrokeChips"],
    ["cursor.spotlight"],
    ["cursor.highlightBox"],
    ["text.tooltip", { text: "Saved here" }],
    ["text.calloutArrow", { text: "Drag the card" }],
  ]);
}

function takeFilm(builder: Builder, recording: string, sounds: string[]): string {
  builder.section("Camera, cursor, captions and sound on the browser take");
  const edited = builder.chain(recording, "recording", [
    ["camera.move"],
    ["camera.move", { fromSpot: true }],
    ["camera.speedRamp"],
    ["effect.redactTake"],
  ]);
  const style = recordingStyle(builder);
  const take = builder.add("camera.take");
  builder.link(edited, "recording", take, "recording");
  builder.link(style, "style", take, "style");
  const framed = builder.chain(take, "clip", [["camera.crop"], ["camera.deviceFrame"]]);
  const looks = builder.add("camera.frame", { kind: "device", device: "phone" });
  const styled = builder.add("camera.applyStyle");
  builder.link(framed, "clip", styled, "clip");
  builder.link(looks, "style", styled, "style");
  const script = builder.add("text.captionsFromText", {
    script: "Type a name. Save it. Drag the card. Pick a size.",
  });
  const captioned = builder.add("text.captions");
  builder.link(script, "captions", captioned, "captions");
  builder.link(styled, "clip", captioned, "clip");
  const actionSounds = builder.add("audio.actionSounds");
  builder.link(take, "clip", actionSounds, "clip");
  const transitionSounds = builder.add("audio.transitionSounds");
  builder.link(take, "clip", transitionSounds, "clip");
  const mix = builder.add("audio.mix");
  builder.link(captioned, "clip", mix, "clip");
  for (const sound of [actionSounds, transitionSounds, ...sounds])
    builder.link(sound, "audio", mix, "sounds");
  render(builder, mix);
  const still = builder.add("output.still");
  builder.link(take, "clip", still, "clip");
  const takeFile = builder.add("output.take");
  builder.link(recording, "recording", takeFile, "recording");
  return take;
}

function terminalTakes(builder: Builder): void {
  builder.section("Terminal: a session with every action, a tape and a cast");
  const open = builder.add("terminal.open");
  const session = builder.chain(open, "terminal", [
    ["terminal.startRecording"],
    ["terminal.type", { text: "echo profile run" }],
    ["terminal.keys"],
    ["terminal.waitText", { pattern: "profile run" }],
    ["terminal.run"],
    ["terminal.chapter"],
    ["terminal.click", { text: "profile" }],
    ["terminal.drag", { fromText: "profile", toKind: "text", toText: "run" }],
    ["terminal.wheel", { text: "profile" }],
    ["terminal.clear"],
  ]);
  const capture = builder.add("capture.terminal");
  builder.link(session, "terminal", capture, "terminal");
  const look = builder.add("terminal.look");
  const take = builder.add("camera.take");
  builder.link(capture, "recording", take, "recording");
  builder.link(look, "style", take, "style");
  render(builder, take);
  const tape = builder.add("output.tape");
  builder.link(session, "terminal", tape, "terminal");
  const cast = builder.add("terminal.castFile", { file: ref("cast") });
  const castFile = builder.add("output.take");
  builder.link(cast, "recording", castFile, "recording");
  const played = builder.add("terminal.playTape", { file: ref("tape"), recordLive: false });
  const playedFile = builder.add("output.take");
  builder.link(played, "recording", playedFile, "recording");
}

const MEDIA_ONE_INPUT = [
  "media.trim",
  "media.speed",
  "media.reverse",
  "media.scalePad",
  "media.convert",
  "media.gif",
  "media.loudnorm",
  "media.fixClicks",
  "media.fixClipping",
  "media.hum",
  "media.soundPicture",
  "media.removeNoise",
  "media.softenS",
  "media.pitch",
  "media.voiceEffect",
  "media.thumbnail",
];

function gradeChain(builder: Builder): string {
  builder.section("Grades");
  const first = builder.add("grade.colour");
  const last = builder.chain(first, "grade", [
    ["grade.lookFile", { file: ref("look") }],
    ["grade.mono"],
    ["grade.texture"],
    ["grade.lens"],
    ["grade.autoFix"],
  ]);
  const members = builder.nodes
    .filter((node) => node.type.startsWith("grade."))
    .map((node) => node.id);
  builder.group("Grade chain", members);
  return last;
}

function mediaNodes(builder: Builder, grade: string): { clip: string; music: string } {
  builder.section("Media: every file operation on one generated clip");
  const clip = builder.add("media.import", { path: ref("clip") });
  for (const type of MEDIA_ONE_INPUT) {
    const node = builder.add(type);
    builder.link(clip, "media", node, "media");
  }
  const trimmed = builder.nodes.find((node) => node.type === "media.trim")?.id ?? "";
  const joined = builder.add("media.concat");
  builder.link(clip, "media", joined, "in1");
  builder.link(trimmed, "media", joined, "in2");
  const graded = builder.add("media.applyGrade");
  builder.link(clip, "media", graded, "media");
  builder.link(grade, "grade", graded, "grade");
  const webReady = builder.add("media.convert", { keepFormat: true });
  builder.link(clip, "media", webReady, "media");
  const audio = builder.add("media.extractAudio");
  builder.link(clip, "media", audio, "media");
  const asClip = builder.add("media.toClip");
  builder.link(clip, "media", asClip, "media");
  render(builder, asClip);
  const animation = builder.add("media.import", { path: ref("lottie") });
  const animationClip = builder.add("media.toClip", { loop: true });
  builder.link(animation, "media", animationClip, "media");
  render(builder, animationClip);
  const fitted = builder.add("media.fitMusic");
  builder.link(asClip, "clip", fitted, "clip");
  builder.link(audio, "media", fitted, "media");
  const shortened = builder.add("media.shortenPauses");
  builder.link(audio, "media", shortened, "media");
  const gradedFootage = builder.add("effect.gradeFootage");
  builder.link(asClip, "clip", gradedFootage, "clip");
  builder.link(grade, "grade", gradedFootage, "grade");
  render(builder, gradedFootage);
  return { clip: asClip, music: audio };
}

const IMAGE_ONE_INPUT = [
  "image.resize",
  "image.crop",
  "image.rotate",
  "image.format",
  "image.redact",
  "image.spotlight",
  "image.trim",
  "image.clearColour",
  "image.outline",
  "image.frame",
  "image.magnify",
  "image.annotate",
];

function imageNodes(builder: Builder, still: string): void {
  builder.section("Images: every image operation on a generated picture and the page still");
  const picture = builder.add("media.import", { path: ref("picture") });
  const overlay = builder.add("media.import", { path: ref("overlay") });
  for (const type of IMAGE_ONE_INPUT) {
    const node = builder.add(type);
    builder.link(picture, "media", node, "image");
  }
  const adjusted = builder.nodes.find((node) => node.type === "image.resize")?.id ?? "";
  const tilted = builder.add("image.frame", { kind: "window", angle: "tilted" });
  builder.link(still, "image", tilted, "image");
  const onPhone = builder.add("image.frame", { kind: "device", device: "phone" });
  builder.link(picture, "media", onPhone, "image");
  const placed = builder.add("image.place");
  builder.link(still, "image", placed, "image");
  builder.link(overlay, "media", placed, "overlay");
  const grid = builder.add("image.grid");
  builder.link(picture, "media", grid, "in1");
  builder.link(overlay, "media", grid, "in2");
  for (const type of ["image.beforeAfter", "image.compare"]) {
    const node = builder.add(type);
    builder.link(picture, "media", node, "image");
    builder.link(adjusted, "image", node, "after");
  }
}

function soundNodes(builder: Builder, music: string): string[] {
  builder.section("Sounds made for the browser film's mix");
  const made = [
    builder.add("audio.roomTone"),
    builder.add("audio.sfx", { source: "made", sound: "countdown" }),
    builder.add("audio.sfx", { source: "made", sound: "whoosh", timing: "at-time" }),
    builder.add("audio.sfx", {
      source: "made",
      sound: "riser",
      timing: "ends-with-clip",
      endHit: "ding",
    }),
  ];
  const track = builder.add("audio.music");
  builder.link(music, "media", track, "media");
  return [...made, track];
}

const COVERED_ELSEWHERE = new Set([
  "fx.background",
  "audio.mix",
  "audio.music",
  "audio.sfx",
  "audio.actionSounds",
  "audio.transitionSounds",
  "text.captions",
  "text.captionsFromText",
  "text.tooltip",
  "text.calloutArrow",
  "effect.gradeFootage",
  "effect.redactTake",
]);

function isFilmNode(node: CatalogNode): boolean {
  const outputs = Object.values(node.outputs);
  if (node.executes || node.virtual || node.hidden || COVERED_ELSEWHERE.has(node.type))
    return false;
  if (outputs.length !== 1 || outputs[0].type !== "clip") return false;
  return Object.values(node.inputs).every((port) => port.type === "clip");
}

function filmNodes(builder: Builder, catalog: Catalog, clip: string, music: string): void {
  builder.section("Film nodes: each element, transition and timeline node in a render of its own");
  const first = builder.add("fx.background");
  render(builder, first);
  const second = builder.add("fx.background", { solid: { color: "#1d4ed8" } });
  const feeds = [first, second];
  const shared = builder.add("timeline.transition", { kind: "light-leak" });
  const marked = builder.add("timeline.transition", { kind: "wipe" });
  builder.link(second, "clip", marked, "clip");
  const joined = builder.add("timeline.sequence");
  builder.link(first, "clip", joined, "in1");
  builder.link(marked, "clip", joined, "in2");
  const rejoined = builder.add("timeline.sequence");
  builder.link(joined, "clip", rejoined, "in1");
  builder.link(first, "clip", rejoined, "in2");
  builder.link(shared, "transition", rejoined, "transition");
  render(builder, rejoined);
  const settled = builder.add("timeline.transition", { kind: "spring-settle" });
  const repeated = builder.add("timeline.loop", { times: 3 });
  builder.link(first, "clip", repeated, "clip");
  builder.link(settled, "transition", repeated, "transition");
  render(builder, repeated);
  const film = catalog.nodes.filter(isFilmNode).filter((node) => !node.type.startsWith("camera."));
  for (const node of film) {
    const id = builder.add(node.type, FILE_ELEMENTS[node.type] ?? {});
    Object.keys(node.inputs).forEach((handle, index) => {
      builder.link(feeds[index % feeds.length], "clip", id, handle);
    });
    render(builder, id);
  }
  const fitted = builder.add("timeline.durationFit");
  builder.link(clip, "clip", fitted, "clip");
  builder.link(music, "audio", fitted, "audio");
  render(builder, fitted);
}

export function profileWorkflow(catalog: Catalog, siteAddress: string): Workflow {
  const builder = workflowBuilder(catalog);
  const grade = gradeChain(builder);
  const { clip, music } = mediaNodes(builder, grade);
  const sounds = soundNodes(builder, music);
  const { recording, still } = browserTake(builder);
  takeFilm(builder, recording, sounds);
  imageNodes(builder, still);
  terminalTakes(builder);
  filmNodes(builder, catalog, clip, sounds.at(-1) ?? "");
  return {
    version: 1,
    id: "wf_profile",
    name: "Profile every node",
    settings: {
      width: 1920,
      height: 1080,
      fps: 30,
      theme: "dark",
      fonts: { main: "Inter", heading: "", code: "JetBrains Mono", families: [] },
    },
    variables: { [SITE_VARIABLE]: siteAddress },
    nodes: builder.nodes,
    edges: builder.edges,
    mix: null,
    markers: [],
  };
}

export function missingTypes(catalog: Catalog, workflow: Workflow): string[] {
  const used = new Set(workflow.nodes.map((node) => node.type));
  return catalog.nodes.map((node) => node.type).filter((type) => !used.has(type));
}

export function runOrder(catalog: Catalog, workflow: Workflow): string[] {
  const executes = new Set(catalog.nodes.filter((node) => node.executes).map((node) => node.type));
  const upstream = new Map<string, string[]>();
  for (const edge of workflow.edges)
    upstream.set(edge.target, [...(upstream.get(edge.target) ?? []), edge.source]);
  const order: string[] = [];
  const placed = new Set<string>();
  function visit(id: string): void {
    if (placed.has(id)) return;
    placed.add(id);
    for (const source of upstream.get(id) ?? []) visit(source);
    order.push(id);
  }
  for (const node of workflow.nodes) visit(node.id);
  const types = new Map(workflow.nodes.map((node) => [node.id, node.type]));
  return order.filter((id) => executes.has(types.get(id) ?? ""));
}
