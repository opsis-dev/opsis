import path from "node:path";
import type { ElementKind } from "../../src/shared/elements/types";
import { humanize } from "../../src/shared/format";
import type * as framing from "../../src/video/registry/framing";
import { REGISTRIES, TITLE_ACRONYMS, type CuratedElement } from "../registry-ingest.list";
import type { Control } from "./registry-controls";
import { VIDEO_DIR } from "./tool-config";

function metaOf(key: string, c: Control, extra: Record<string, unknown> = {}): string {
  const meta: Record<string, unknown> = {
    description: c.description ?? c.label ?? key,
    ...extra,
    ...(typeof c.step === "number" ? { step: c.step } : {}),
    ...(c.unit ? { unit: c.unit } : {}),
    ...(c.help ? { help: c.help } : {}),
    ...(c.advanced ? { advanced: true } : {}),
    ...(c.widget ? { widget: c.widget } : {}),
    ...(c.hidden ? { hidden: true } : {}),
  };
  const body = Object.entries(meta)
    .map(([k, v]) => `${k}: ${JSON.stringify(v)}`)
    .join(", ");
  return `.meta({ ${body} })`;
}

const COLLECTIONS = {
  strings: { zod: "z.array(z.string())", empty: [] },
  numbers: { zod: "z.array(z.number())", empty: [] },
  rows: { zod: "z.array(z.record(z.string(), z.unknown()))", empty: [] },
  object: { zod: "z.record(z.string(), z.unknown())", empty: {} },
} as const;

const stringField = (text: string, meta: string) =>
  `z.string().default(${JSON.stringify(text)})${meta}`;

function zodFor(key: string, c: Control): string | null {
  const withMeta = (extra: Record<string, unknown> = {}) => metaOf(key, c, extra);
  const text = String(c.default ?? "");
  switch (c.type) {
    case "text":
    case "text-content":
      return stringField(
        text,
        withMeta(text.includes("\n") || text.length > 60 ? { widget: "textarea" } : {}),
      );
    case "code":
      return stringField(text, withMeta({ widget: "code" }));
    case "font":
      return stringField("", withMeta({ widget: "font" }));
    case "image":
    case "asset":
      return stringField(
        text,
        withMeta({ widget: "asset", ...(c.accept ? { accept: c.accept } : {}) }),
      );
    case "number": {
      let s = "z.number()";
      if (typeof c.min === "number") s += `.min(${c.min})`;
      if (typeof c.max === "number") s += `.max(${c.max})`;
      s += `.default(${typeof c.default === "number" ? c.default : 0})`;
      return s + withMeta();
    }
    case "color":
      return `zColor().default(${JSON.stringify(String(c.default ?? "#000000"))})${withMeta()}`;
    case "boolean":
      return `z.boolean().default(${c.default === true})${withMeta()}`;
    case "enum":
    case "select": {
      const opts = c.options ?? Object.keys(c.variants ?? {});
      if (opts.length === 0) return null;
      const def =
        c.default != null && opts.includes(String(c.default)) ? String(c.default) : opts[0];
      return `z.enum(${JSON.stringify(opts.map(String))}).default(${JSON.stringify(def)})${withMeta()}`;
    }
    case "strings":
    case "numbers":
    case "rows":
    case "object": {
      const { zod, empty } = COLLECTIONS[c.type];
      return `${zod}.default(${JSON.stringify(c.default ?? empty)})${withMeta()}`;
    }
  }
}

const titleOf = (name: string): string =>
  humanize(name)
    .split(" ")
    .map((word) => (TITLE_ACRONYMS.has(word.toUpperCase()) ? word.toUpperCase() : word))
    .join(" ");

const CATEGORY_KINDS: Partial<Record<CuratedElement["category"], ElementKind>> = {
  transition: "presentation",
  effect: "wrapper",
};

const kindOf = (item: CuratedElement): ElementKind =>
  item.kind ?? CATEGORY_KINDS[item.category] ?? "component";

type FramingHelper = keyof typeof framing;

const WRAPPERS: Array<[applies: (item: CuratedElement) => boolean, wrapper: FramingHelper]> = [
  [(item) => !!item.screen, "fitScreen"],
  [(item) => !!item.center, "centered"],
  [(item) => !!item.atEnd, "playsAtEnd"],
  [(item) => !!item.paint, "painted"],
  [(item) => item.place === "spot", "placedAt"],
  [(item) => item.place === "points", "pointsFromSpots"],
];

export type Resolved = {
  item: CuratedElement;
  key: string;
  file: string;
  exportName: string;
  description?: string;
  controls: Array<[string, Control]>;
  duration: number;
};

const IDENTIFIER = /^[A-Za-z_$][\w$]*$/;

export function renderMetaEntry({ item, key, description, controls, duration }: Resolved): string {
  const reg = REGISTRIES[item.registry];
  const fields = controls
    .map(([k, c]) => [k, zodFor(k, c)] as const)
    .filter((f): f is readonly [string, string] => f[1] != null)
    .map(([k, schema]) => `      ${IDENTIFIER.test(k) ? k : JSON.stringify(k)}: ${schema},`);
  return [
    `  {`,
    `    key: ${JSON.stringify(key)},`,
    `    title: ${JSON.stringify(item.title ?? titleOf(item.name))},`,
    ...(description ? [`    description: ${JSON.stringify(description)},`] : []),
    `    category: ${JSON.stringify(item.category)},`,
    `    kind: ${JSON.stringify(kindOf(item))},`,
    `    source: { registry: ${JSON.stringify(reg.label)}, url: ${JSON.stringify(`${reg.docs}/${item.name}`)}, license: ${JSON.stringify(reg.license)} },`,
    `    schema: z.object({`,
    ...fields,
    `    }),`,
    `    fixedProps: ${JSON.stringify(item.fixedProps ?? {})},`,
    `    defaultDurationInFrames: ${duration},`,
    `  },`,
  ].join("\n");
}

export function renderLoaderEntry({ item, key, file, exportName }: Resolved): {
  loader: string;
  helpers: FramingHelper[];
} {
  const base = kindOf(item) === "presentation" ? "presentation" : "component";
  const wrappers = WRAPPERS.filter(([applies]) => applies(item)).map(([, wrapper]) => wrapper);
  const impl = wrappers.reduce(
    (inner, wrapper) => `${wrapper}(${inner})`,
    `${base}(m.${exportName})`,
  );
  const rel =
    "../" +
    path
      .relative(VIDEO_DIR, file)
      .replace(/\\/g, "/")
      .replace(/(\/index)?\.tsx$/, "");
  return {
    loader: `  ${JSON.stringify(key)}: () => import(${JSON.stringify(rel)}).then((m) => ${impl}),`,
    helpers: [base, ...wrappers],
  };
}

export const renderMeta = (entries: string[]): string =>
  [
    'import { zColor } from "@remotion/zod-types";',
    'import { z } from "zod";',
    'import { defineElementMeta, type ElementMetaInput } from "./types";',
    "",
    "const defs: ElementMetaInput[] = [",
    ...entries,
    "];",
    "",
    "export const generatedElementMeta = defs.map(defineElementMeta);",
    "",
  ].join("\n");

export const renderLoaders = (loaders: string[], helpers: Set<FramingHelper>): string =>
  [
    `import { ${[...helpers].sort().join(", ")} } from "./framing";`,
    'import type { ElementImpl } from "./types";',
    "",
    "export const generatedElementLoaders: Record<string, () => Promise<ElementImpl>> = {",
    ...loaders,
    "};",
    "",
  ].join("\n");
