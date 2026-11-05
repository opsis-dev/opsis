import { nonBlankString } from "@opsis/video/common/guards";
import { findById } from "@opsis/video/common/lists";
import {
  commands,
  type Category,
  type EditorCatalog,
  type EditorNode,
  type FontFamilyDef,
  type MenuSection,
  type PortType,
  type TemplateInfo,
  type MixerBook,
  type NodeLook,
  type SettingsBook,
  type ShapeChoice,
  type SoundChoice,
  type TerminalTheme,
  type TimelineBook,
  type Theme,
  type TextRules,
} from "@/generated/bindings";
import { FALLBACK_LOOK } from "@/nodes/looks";

type Params = Record<string, unknown>;

type Named = { id?: string; label?: string; type: string };

const loaded: {
  nodes: EditorNode[];
  byType: Map<string, EditorNode>;
  menu: MenuSection[];
  transitions: Record<string, string>;
  connects: Set<string>;
  text: TextRules;
  mixer: MixerBook | null;
  bundledFonts: FontFamilyDef[];
  settings: SettingsBook | null;
  themeBackgrounds: Partial<Record<Theme, string>>;
  shapes: ShapeChoice[];
  qualityPresets: Array<[string, number]>;
  loudnessPresets: Array<[string, number]>;
  timeline: TimelineBook;
  sounds: SoundChoice[];
  music: SoundChoice[];
  terminalThemes: TerminalTheme[];
  templates: TemplateInfo[];
} = {
  nodes: [],
  byType: new Map(),
  menu: [],
  transitions: {},
  connects: new Set(),
  text: { variablePattern: "(?!)", pauseMark: "" },
  mixer: null,
  bundledFonts: [],
  settings: null,
  themeBackgrounds: {},
  shapes: [],
  qualityPresets: [],
  loudnessPresets: [],
  timeline: { eases: [], envelopes: [], cameraDials: [] },
  sounds: [],
  music: [],
  terminalThemes: [],
  templates: [],
};

type CatalogData = { catalog: EditorCatalog; menu: MenuSection[]; templates: TemplateInfo[] };

export async function loadCatalog(): Promise<void> {
  const [catalog, menu, listed] = await Promise.all([
    commands.editorCatalog(),
    commands.appMenuSections(),
    commands.listTemplates(),
  ]);
  applyCatalog({ catalog, menu, templates: listed });
}

export function applyCatalog(data: CatalogData): void {
  const { catalog, menu } = data;
  loaded.templates = data.templates;
  const { nodes, transitions, connects, text, mixer, settings } = catalog;
  loaded.settings = settings;
  loaded.themeBackgrounds = catalog.themeBackgrounds;
  loaded.shapes = catalog.shapes;
  loaded.qualityPresets = catalog.qualityPresets;
  loaded.loudnessPresets = catalog.loudnessPresets;
  loaded.timeline = catalog.timeline;
  loaded.sounds = catalog.sounds;
  loaded.music = catalog.music;
  loaded.terminalThemes = catalog.terminalThemes;
  loaded.text = text;
  loaded.mixer = mixer;
  loaded.bundledFonts = catalog.bundledFonts;
  loaded.nodes = nodes;
  loaded.transitions = transitions;
  loaded.connects = new Set(connects.map((link) => `${link.from}>${link.to}`));
  loaded.byType = new Map(nodes.map((n) => [n.type, n]));
  loaded.menu = menu;
}

export function textRules(): TextRules {
  return loaded.text;
}

export function bundledFonts(): FontFamilyDef[] {
  return loaded.bundledFonts;
}

export function shapes(): ShapeChoice[] {
  return loaded.shapes;
}

export function timelineBook(): TimelineBook {
  return loaded.timeline;
}

export function builtinSounds(): SoundChoice[] {
  return loaded.sounds;
}

export function builtinMusic(): SoundChoice[] {
  return loaded.music;
}

export function terminalThemes(): TerminalTheme[] {
  return loaded.terminalThemes;
}

export function qualityPresets(): Array<[string, number]> {
  return loaded.qualityPresets;
}

export function loudnessPresets(): Array<[string, number]> {
  return loaded.loudnessPresets;
}

export function themeBackground(theme: Theme): string {
  return loaded.themeBackgrounds[theme] ?? "";
}

export function settingsBook(): SettingsBook {
  if (!loaded.settings) throw new Error("The node catalogue hasn't loaded yet.");
  return loaded.settings;
}

export function mixerBook(): MixerBook {
  if (!loaded.mixer) throw new Error("The node catalogue hasn't loaded yet.");
  return loaded.mixer;
}

const NO_TRANSITION = "none";

export function transitionTitle(kind: string): string {
  return loaded.transitions[kind] ?? kind;
}

export function catalogNodes(): readonly EditorNode[] {
  return loaded.nodes;
}

export function nodeLook(type: string): NodeLook {
  return loaded.byType.get(type)?.look ?? FALLBACK_LOOK;
}

export function visibleNodes(): EditorNode[] {
  return loaded.nodes.filter((n) => !n.hidden);
}

export function appMenu(): readonly MenuSection[] {
  return loaded.menu;
}

export function getNodeDef(type: string): EditorNode | undefined {
  return loaded.byType.get(type);
}

function defaultParams(type: string): Params {
  return structuredClone(getNodeDef(type)?.defaults ?? {});
}

function newNodeParams(type: string, params?: Params): Params {
  return {
    ...defaultParams(type),
    ...structuredClone(getNodeDef(type)?.hostDefaults ?? {}),
    ...params,
  };
}

export function customLabel(node: { label?: string }): string | undefined {
  return nonBlankString(node.label);
}

export function nodeName(node: Named, fallback?: string): string {
  return customLabel(node) ?? fallback ?? (getNodeDef(node.type)?.title || node.type);
}

export function nodeNameById(
  nodes: ReadonlyArray<Named & { id: string }>,
  id: string,
  whenMissing = id,
): string {
  const node = findById(nodes, id);
  return node ? nodeName(node) : whenMissing;
}

export function isOutputType(type: string): boolean {
  return getNodeDef(type)?.category === "Output";
}

const CATEGORY_ORDER: Category[] = [
  "Browser",
  "Capture",
  "Camera",
  "Cursor",
  "Text & overlays",
  "Transitions",
  "Backgrounds",
  "Effects",
  "Grade",
  "Elements",
  "Audio",
  "Timeline",
  "Media",
  "Output",
  "Utility",
];

export function categoryLabel(category: Category): string {
  return loaded.nodes.find((n) => n.category === category)?.categoryLabel ?? category;
}

export function nodeCategories(): Category[] {
  return CATEGORY_ORDER.filter((c) => loaded.nodes.some((n) => n.category === c && !n.hidden));
}

export function canConnect(from: PortType, to: PortType): boolean {
  return loaded.connects.has(`${from}>${to}`);
}

export function templates(): readonly TemplateInfo[] {
  return loaded.templates;
}

export function getTemplate(id: string): TemplateInfo | undefined {
  return findById(loaded.templates, id);
}
