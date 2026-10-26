import path from "node:path";
import { humanize } from "@opsis/video/common/format";
import { isNumber, isRecord, isString } from "@opsis/video/common/guards";
import { roundTo } from "@opsis/video/common/math";
import { pick } from "@opsis/video/common/objects";
import { AUTHORED_FPS } from "@opsis/video/common/spec";
import type * as framing from "@opsis/video/registry/framing";
import { REGISTRIES, SECONDS, TITLE_ACRONYMS, type CuratedElement } from "../registry-ingest.list";
import type { ElementEntry, Json } from "./element-json";
import { META_KEYWORDS, THEME_MODES, type ElementKind, type FieldMeta } from "./element-meta";
import type { Control } from "./registry-controls";
import { VIDEO_DIR } from "./tool-config";

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

const keywordsOf = ({ description, ...meta }: FieldMeta): Json => ({
  description,
  ...Object.fromEntries(
    Object.entries(meta).map(([k, v]) => [META_KEYWORDS[k as keyof typeof META_KEYWORDS], v]),
  ),
});

const fieldMeta = (key: string, c: Control, extra: FieldMeta = {}): FieldMeta => ({
  description: c.description ?? c.label ?? key,
  ...extra,
  ...(c.emptyLabel ? { zeroLabel: c.emptyLabel } : {}),
  ...controlMeta(c),
});

const COLOUR: Json = { type: "string", "x-check": { rule: "colour", message: "Pick a colour" } };

const THEME_VALUE: Json = {
  type: "object",
  properties: {
    mode: { type: "string", enum: THEME_MODES },
    colors: {
      type: "object",
      propertyNames: { type: "string" },
      additionalProperties: { type: "string" },
    },
  },
  required: ["mode", "colors"],
};

const listOf = (c: Control, items: Json): Json => ({
  ...(typeof c.min === "number" ? { minItems: c.min } : {}),
  ...(typeof c.max === "number" ? { maxItems: c.max } : {}),
  type: "array",
  items,
});

const COLLECTIONS = {
  strings: { items: { type: "string" }, empty: [] },
  numbers: { items: { type: "number" }, empty: [] },
} as const;

function rowSchema(c: Control): Json | null {
  const fields = Object.entries(c.columns ?? {}).map(
    ([k, col]) => [k, fieldSchema(k, col)] as const,
  );
  if (fields.length === 0 || fields.some(([, schema]) => schema === null)) return null;
  return { type: "object", properties: Object.fromEntries(fields) };
}

type Base = { schema: Json; fallback: unknown; extra?: FieldMeta };

function baseSchema(c: Control): Base | null {
  const text = String(c.default ?? "");
  const string = { type: "string" };
  switch (c.type) {
    case "text":
    case "text-content":
      return {
        schema: string,
        fallback: text,
        extra: text.includes("\n") || text.length > 60 ? { widget: "textarea" } : {},
      };
    case "code":
      return { schema: string, fallback: text, extra: { widget: "code" } };
    case "font":
      return { schema: string, fallback: "", extra: { widget: "font" } };
    case "image":
    case "asset":
      return {
        schema: string,
        fallback: text,
        extra: { widget: "asset", ...(c.accept ? { accept: c.accept } : {}) },
      };
    case "gradient":
      return { schema: string, fallback: text, extra: { widget: "gradient" } };
    case "number":
      return {
        schema: {
          type: "number",
          ...(typeof c.min === "number" ? { minimum: c.min } : {}),
          ...(typeof c.max === "number" ? { maximum: c.max } : {}),
        },
        fallback: typeof c.default === "number" ? c.default : 0,
      };
    case "color":
      return c.emptyLabel
        ? { schema: string, fallback: text, extra: { widget: "color" } }
        : { schema: COLOUR, fallback: String(c.default ?? "#000000") };
    case "boolean":
      return { schema: { type: "boolean" }, fallback: c.default === true };
    case "enum":
    case "select": {
      const opts = (c.options ?? Object.keys(c.variants ?? {})).map(String);
      if (opts.length === 0) return null;
      const def =
        c.default != null && opts.includes(String(c.default)) ? String(c.default) : opts[0];
      return { schema: { type: "string", enum: opts }, fallback: def };
    }
    case "colors":
      return { schema: listOf(c, COLOUR), fallback: c.default ?? [] };
    case "theme":
      return {
        schema: THEME_VALUE,
        fallback: { mode: c.default === "light" ? "light" : "dark", colors: {} },
        extra: { widget: "theme" },
      };
    case "rows": {
      const row = rowSchema(c);
      return row ? { schema: listOf(c, row), fallback: c.default ?? [] } : null;
    }
    case "strings":
    case "numbers": {
      const { items, empty } = COLLECTIONS[c.type];
      return { schema: listOf(c, items), fallback: c.default ?? empty };
    }
    case "object":
      return {
        schema: { type: "object", propertyNames: { type: "string" }, additionalProperties: {} },
        fallback: c.default ?? {},
      };
  }
}

export function fieldSchema(key: string, c: Control): Json | null {
  const base = baseSchema(c);
  if (!base) return null;
  return {
    ...(c.optional ? {} : { default: base.fallback }),
    ...keywordsOf(fieldMeta(key, c, base.extra)),
    ...base.schema,
  };
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

export function elementEntry({
  item,
  key,
  description,
  controls,
  duration,
}: Resolved): ElementEntry {
  const reg = REGISTRIES[item.registry];
  const properties = Object.fromEntries(
    withSeconds(item.name, controls).flatMap(([k, c]) => {
      const schema = fieldSchema(k, c);
      return schema ? [[k, schema]] : [];
    }),
  );
  const defaultProps = Object.fromEntries(
    Object.entries(properties).flatMap(([k, schema]) =>
      "default" in schema ? [[k, schema.default]] : [],
    ),
  );
  return {
    key,
    title: item.title ?? titleOf(item.name),
    ...(description ? { description } : {}),
    category: item.category,
    kind: kindOf(item),
    source: { registry: reg.label, url: `${reg.docs}/${item.name}`, license: reg.license },
    schema: { type: "object", properties },
    defaultProps,
    fixedProps: item.fixedProps ?? {},
    defaultDurationInFrames: duration,
    ...(item.design ? { design: item.design } : {}),
    ...(item.optionalClip ? { optionalClip: true } : {}),
  };
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
