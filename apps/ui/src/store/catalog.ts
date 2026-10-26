import { nonBlankString } from "@opsis/video/common/guards";
import { findById } from "@opsis/video/common/lists";
import {
  commands,
  type Category,
  type EditorCatalog,
  type EditorNode,
  type MenuSection,
  type PortType,
  type TemplateInfo,
} from "@/generated/bindings";

type Params = Record<string, unknown>;

type Named = { id?: string; label?: string; type: string };

const loaded: {
  nodes: EditorNode[];
  byType: Map<string, EditorNode>;
  menu: MenuSection[];
  transitions: Record<string, string>;
  connects: Set<string>;
  templates: TemplateInfo[];
} = {
  nodes: [],
  byType: new Map(),
  menu: [],
  transitions: {},
  connects: new Set(),
  templates: [],
};

type CatalogData = { catalog: EditorCatalog; menu: MenuSection[]; templates: TemplateInfo[] };

export async function loadCatalog(): Promise<void> {
  const [catalog, menu, templates] = await Promise.all([
    commands.editorCatalog(),
    commands.appMenuSections(),
    commands.listTemplates(),
  ]);
  applyCatalog({ catalog, menu, templates });
}

export function applyCatalog({ catalog, menu, templates }: CatalogData): void {
  loaded.templates = templates;
  const { nodes, transitions, connects } = catalog;
  loaded.nodes = nodes;
  loaded.transitions = transitions;
  loaded.connects = new Set(connects.map((link) => `${link.from}>${link.to}`));
  loaded.byType = new Map(nodes.map((n) => [n.type, n]));
  loaded.menu = menu;
}

export const NO_TRANSITION = "none";

export const transitionTitle = (kind: string): string => loaded.transitions[kind] ?? kind;

export const catalogNodes = (): readonly EditorNode[] => loaded.nodes;

export const visibleNodes = (): EditorNode[] => loaded.nodes.filter((n) => !n.hidden);

export const appMenu = (): readonly MenuSection[] => loaded.menu;

export const getNodeDef = (type: string): EditorNode | undefined => loaded.byType.get(type);

export const defaultParams = (type: string): Params =>
  structuredClone(getNodeDef(type)?.defaults ?? {});

export const newNodeParams = (type: string, params?: Params): Params => ({
  ...defaultParams(type),
  ...structuredClone(getNodeDef(type)?.hostDefaults ?? {}),
  ...params,
});

export const customLabel = (node: { label?: string }): string | undefined =>
  nonBlankString(node.label);

export const nodeName = (node: Named, fallback?: string): string =>
  customLabel(node) ?? fallback ?? (getNodeDef(node.type)?.title || node.type);

export function nodeNameById(
  nodes: ReadonlyArray<Named & { id: string }>,
  id: string,
  whenMissing = id,
): string {
  const node = findById(nodes, id);
  return node ? nodeName(node) : whenMissing;
}

export const isOutputType = (type: string): boolean => getNodeDef(type)?.category === "Output";

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

export const categoryLabel = (category: Category): string =>
  loaded.nodes.find((n) => n.category === category)?.categoryLabel ?? category;

export const nodeCategories = (): Category[] =>
  CATEGORY_ORDER.filter((c) => loaded.nodes.some((n) => n.category === c && !n.hidden));

export const canConnect = (from: PortType, to: PortType): boolean =>
  loaded.connects.has(`${from}>${to}`);

export const templates = (): readonly TemplateInfo[] => loaded.templates;

export const getTemplate = (id: string): TemplateInfo | undefined => findById(loaded.templates, id);
