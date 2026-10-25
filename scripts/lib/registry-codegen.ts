import path from "node:path";
import type { FieldMeta } from "@opsis/core/elements/introspect";
import { AUTHORED_FPS } from "@opsis/core/elements/lengths";
import type { ElementKind } from "@opsis/core/elements/types";
import { humanize } from "@opsis/core/format";
import { isNumber, isRecord, isString } from "@opsis/core/guards";
import { roundTo } from "@opsis/core/math";
import { pick } from "@opsis/core/objects";
import type * as framing from "@opsis/video/registry/framing";
import { REGISTRIES, SECONDS, TITLE_ACRONYMS, type CuratedElement } from "../registry-ingest.list";
import type { Control } from "./registry-controls";
import { VIDEO_DIR } from "./tool-config";

const IDENTIFIER = /^[A-Za-z_$][\w$]*$/;

const CONTROL_META_KEYS = [
  "step",
  "unit",
  "help",
  "advanced",
  "widget",
  "hidden",
  "showIf",
  "labels",
  "themes",
  "framesAt",
] as const;

const controlMeta = (c: Control): FieldMeta =>
  pick(
    c,
    CONTROL_META_KEYS.filter((k) => typeof c[k] === "number" || Boolean(c[k])),
  );

function metaOf(key: string, c: Control, extra: FieldMeta = {}): string {
  const meta: FieldMeta = {
    description: c.description ?? c.label ?? key,
    ...extra,
    ...(c.emptyLabel ? { zeroLabel: c.emptyLabel } : {}),
    ...controlMeta(c),
  };
  const body = Object.entries(meta)
    .map(([k, v]) => `${k}: ${JSON.stringify(v)}`)
    .join(", ");
  return `.meta({ ${body} })`;
}

const COLLECTIONS = {
  strings: { zod: "z.array(z.string())", empty: [] },
  numbers: { zod: "z.array(z.number())", empty: [] },
  object: { zod: "z.record(z.string(), z.unknown())", empty: {} },
} as const;

const lengths = (c: Control) =>
  `${typeof c.min === "number" ? `.min(${c.min})` : ""}${typeof c.max === "number" ? `.max(${c.max})` : ""}`;

function rowsZod(c: Control): string | null {
  const fields = Object.entries(c.columns ?? {}).map(([k, col]) => {
    const schema = zodFor(k, col);
    return schema ? `${IDENTIFIER.test(k) ? k : JSON.stringify(k)}: ${schema}` : null;
  });
  if (fields.length === 0 || fields.includes(null)) return null;
  return `z.array(z.object({ ${fields.join(", ")} }))`;
}

function baseZod(c: Control): { zod: string; fallback: unknown; extra?: FieldMeta } | null {
  const text = String(c.default ?? "");
  switch (c.type) {
    case "text":
    case "text-content":
      return {
        zod: "z.string()",
        fallback: text,
        extra: text.includes("\n") || text.length > 60 ? { widget: "textarea" } : {},
      };
    case "code":
      return { zod: "z.string()", fallback: text, extra: { widget: "code" } };
    case "font":
      return { zod: "z.string()", fallback: "", extra: { widget: "font" } };
    case "image":
    case "asset":
      return {
        zod: "z.string()",
        fallback: text,
        extra: { widget: "asset", ...(c.accept ? { accept: c.accept } : {}) },
      };
    case "gradient":
      return { zod: "z.string()", fallback: text, extra: { widget: "gradient" } };
    case "number": {
      let zod = "z.number()";
      if (typeof c.min === "number") zod += `.min(${c.min})`;
      if (typeof c.max === "number") zod += `.max(${c.max})`;
      return { zod, fallback: typeof c.default === "number" ? c.default : 0 };
    }
    case "color":
      return c.emptyLabel
        ? { zod: "z.string()", fallback: text, extra: { widget: "color" } }
        : { zod: "zColor()", fallback: String(c.default ?? "#000000") };
    case "boolean":
      return { zod: "z.boolean()", fallback: c.default === true };
    case "enum":
    case "select": {
      const opts = c.options ?? Object.keys(c.variants ?? {});
      if (opts.length === 0) return null;
      const def =
        c.default != null && opts.includes(String(c.default)) ? String(c.default) : opts[0];
      return { zod: `z.enum(${JSON.stringify(opts.map(String))})`, fallback: def };
    }
    case "colors":
      return { zod: `z.array(zColor())${lengths(c)}`, fallback: c.default ?? [] };
    case "theme":
      return {
        zod: "themeValueSchema",
        fallback: { mode: c.default === "light" ? "light" : "dark", colors: {} },
        extra: { widget: "theme" },
      };
    case "rows": {
      const zod = rowsZod(c);
      return zod ? { zod: `${zod}${lengths(c)}`, fallback: c.default ?? [] } : null;
    }
    case "strings":
    case "numbers":
    case "object": {
      const { zod, empty } = COLLECTIONS[c.type];
      return { zod: `${zod}${lengths(c)}`, fallback: c.default ?? empty };
    }
  }
}

export function zodFor(key: string, c: Control): string | null {
  const base = baseZod(c);
  if (!base) return null;
  const tail = c.optional ? ".optional()" : `.default(${JSON.stringify(base.fallback)})`;
  return `${base.zod}${tail}${metaOf(key, c, base.extra)}`;
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

type Wrap = { helper: FramingHelper; arg?: unknown };

const WRAPPERS: Array<(item: CuratedElement) => Wrap | null> = [
  (item) => (item.nest ? { helper: "nested", arg: item.nest } : null),
  (item) => (item.lists ? { helper: "listsFromRows", arg: item.lists } : null),
  (item) => (item.marks ? { helper: "marksFromImages", arg: item.marks } : null),
  (item) => (item.ticks ? { helper: "ticksFromWords", arg: item.ticks } : null),
  (item) => (item.tree ? { helper: "treeFrom", arg: item.tree } : null),
  (item) => (item.theme ? { helper: "themed", arg: item.theme.family } : null),
  (item) => (item.screen ? { helper: "fitScreen" } : null),
  (item) => (item.center ? { helper: "centered" } : null),
  (item) => (item.atEnd ? { helper: "playsAtEnd" } : null),
  (item) => (item.paint ? { helper: "painted" } : null),
  (item) => (item.place === "spot" ? { helper: "placedAt" } : null),
  (item) => (item.place === "points" ? { helper: "pointsFromSpots" } : null),
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

const seconds = (frames: number): number => roundTo(frames / AUTHORED_FPS, 4);

const secondsText = (text: string): string =>
  text
    .split(",")
    .map((part) => {
      const frames = Number(part.trim());
      return part.trim() && Number.isFinite(frames) ? String(seconds(frames)) : part.trim();
    })
    .join(", ");

function inSeconds(c: Control, label: string | true): Control {
  const named = label === true ? {} : { description: label };
  if (c.type === "text") {
    return {
      ...c,
      ...named,
      default: isString(c.default) ? secondsText(c.default) : c.default,
      framesAt: AUTHORED_FPS,
    };
  }
  if (c.type !== "number")
    throw new Error(`${label}: only number and text settings can be in seconds`);
  return {
    ...c,
    ...named,
    default: isNumber(c.default) ? seconds(c.default) : c.default,
    ...(isNumber(c.min) ? { min: seconds(c.min) } : {}),
    ...(isNumber(c.max) ? { max: seconds(c.max) } : {}),
    step: 0.01,
    unit: "s",
    framesAt: AUTHORED_FPS,
  };
}

function rowsInSeconds(c: Control, columns: Record<string, string | true>): Control {
  const changed = Object.fromEntries(
    Object.entries(c.columns ?? {}).map(([name, column]) => [
      name,
      columns[name] === undefined ? column : inSeconds(column, columns[name]),
    ]),
  );
  const rows = Array.isArray(c.default)
    ? c.default.map((row: unknown) =>
        isRecord(row)
          ? Object.fromEntries(
              Object.entries(row).map(([name, value]) => [
                name,
                columns[name] !== undefined && isNumber(value) ? seconds(value) : value,
              ]),
            )
          : row,
      )
    : c.default;
  return { ...c, columns: changed, default: rows };
}

export function withSeconds(
  name: string,
  controls: Array<[string, Control]>,
): Array<[string, Control]> {
  const spec = SECONDS[name];
  if (!spec) return controls;
  const used = new Set<string>();
  const out = controls.map(([key, c]): [string, Control] => {
    const columns = Object.fromEntries(
      Object.entries(spec)
        .filter(([setting]) => setting.startsWith(`${key}.`))
        .map(([setting, label]) => [setting.slice(key.length + 1), label]),
    );
    if (Object.keys(columns).length) {
      for (const column of Object.keys(columns)) used.add(`${key}.${column}`);
      return [key, rowsInSeconds(c, columns)];
    }
    const label = spec[key];
    if (label === undefined) return [key, c];
    used.add(key);
    return [key, inSeconds(c, label)];
  });
  const unknown = Object.keys(spec).filter((setting) => !used.has(setting));
  if (unknown.length)
    throw new Error(`${name}: no settings named ${unknown.join(", ")} to show in seconds`);
  return out;
}

export function renderMetaEntry({ item, key, description, controls, duration }: Resolved): string {
  const reg = REGISTRIES[item.registry];
  const fields = withSeconds(item.name, controls)
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
    ...(item.design ? [`    design: ${JSON.stringify(item.design)},`] : []),
    ...(item.optionalClip ? [`    optionalClip: true,`] : []),
    `  },`,
  ].join("\n");
}

function implOf(
  item: CuratedElement,
  exportName: string,
): { impl: string; helpers: FramingHelper[] } {
  const kind = kindOf(item);
  if (kind === "sequencer")
    return {
      impl: `sequencer(m.${exportName}, m.${item.sceneItem ?? exportName})`,
      helpers: ["sequencer"],
    };
  const base = kind === "presentation" ? "presentation" : "component";
  const wraps = WRAPPERS.flatMap((wrap) => wrap(item) ?? []);
  return {
    impl: wraps.reduce(
      (inner, { helper, arg }) =>
        `${helper}(${inner}${arg === undefined ? "" : `, ${JSON.stringify(arg)}`})`,
      `${base}(m.${exportName})`,
    ),
    helpers: [base, ...wraps.map((w) => w.helper)],
  };
}

export function renderLoaderEntry({ item, key, file, exportName }: Resolved): {
  loader: string;
  helpers: FramingHelper[];
} {
  const { impl, helpers } = implOf(item, exportName);
  const rel = `#video/${path.relative(VIDEO_DIR, file).replace(/\\/g, "/")}`;
  return {
    loader: `  ${JSON.stringify(key)}: () => import(${JSON.stringify(rel)}).then((m) => ${impl}),`,
    helpers,
  };
}

export const renderMeta = (entries: string[]): string =>
  [
    'import { zColor } from "@remotion/zod-types";',
    'import { z } from "zod";',
    'import { themeValueSchema } from "#core/elements/theme";',
    'import { defineElementMeta, type ElementMetaInput } from "#core/elements/types";',
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
    `import { ${[...helpers].sort().join(", ")} } from "#video/registry/framing.tsx";`,
    'import type { ElementImpl } from "#video/registry/types.ts";',
    "",
    "export const generatedElementLoaders: Record<string, () => Promise<ElementImpl>> = {",
    ...loaders,
    "};",
    "",
  ].join("\n");
