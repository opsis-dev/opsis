import { existsSync, readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { parseColor } from "@opsis/core/color";
import { humanize } from "@opsis/core/format";
import { isNumber, isRecord, isString } from "@opsis/core/guards";
import { Node, type SourceFile, type TypeNode } from "ts-morph";
import type { ControlSpec } from "../registry-ingest.list";
import { VIDEO_DIR } from "./tool-config";
import { scratchFile } from "./ts-literal";

type Member = { type: string; optional: boolean };

const HEX = /^#[0-9a-f]{3,8}$/i;
const QUOTED = /"([^"]*)"/g;
const STRING_UNION = /^\s*"[^"]*"(\s*\|\s*"[^"]*")+\s*$/;
const OPEN_STRING_UNION = /^\s*"[^"]*"(\s*\|\s*("[^"]*"|string))+\s*$/;

export const isColorProp = (name: string): boolean =>
  /colou?r$/i.test(name) || name === "stroke" || name === "fill";

const isColorListProp = (name: string): boolean => /colou?rs$/i.test(name);

export const isColorList = (name: string, value: unknown): value is string[] =>
  isColorListProp(name) &&
  Array.isArray(value) &&
  value.length > 0 &&
  value.every((v) => isString(v) && parseColor(v) !== null);

export function unionOptions(type: string, { open = false } = {}): string[] | null {
  if (!(open ? OPEN_STRING_UNION : STRING_UNION).test(type)) return null;
  return [...type.matchAll(QUOTED)].map((m) => m[1]);
}

function typeSources(file: string): SourceFile[] {
  const dir = path.dirname(file);
  const libDir = path.join(VIDEO_DIR, "lib", "remotion-ui");
  const files = [
    file,
    ...readdirSync(dir)
      .filter((f) => /\.tsx?$/.test(f))
      .map((f) => path.join(dir, f)),
    ...(existsSync(libDir) ? readdirSync(libDir).map((f) => path.join(libDir, f)) : []),
  ];
  return [...new Set(files)].map((f, i) =>
    scratchFile(`types-${i}`, "tsx", readFileSync(f, "utf8")),
  );
}

function aliasText(sources: SourceFile[], name: string): string | undefined {
  for (const sf of sources) {
    const alias = sf.getTypeAlias(name);
    if (alias) return alias.getTypeNode()?.getText();
  }
  return undefined;
}

function membersOf(node: TypeNode | undefined): Map<string, Member> {
  const out = new Map<string, Member>();
  if (!node || !Node.isTypeLiteral(node)) return out;
  for (const m of node.getProperties())
    out.set(m.getName(), {
      type: m.getTypeNode()?.getText() ?? "",
      optional: m.hasQuestionToken(),
    });
  return out;
}

export function rowMembers(file: string, typeName: string | undefined): Map<string, Member> {
  if (!typeName) return new Map();
  const sources = typeSources(file);
  for (const sf of sources) {
    const alias = sf.getTypeAlias(typeName);
    if (alias) return resolveAliases(sources, membersOf(alias.getTypeNode()));
    const iface = sf.getInterface(typeName);
    if (iface) {
      const out = new Map<string, Member>();
      for (const m of iface.getProperties())
        out.set(m.getName(), {
          type: m.getTypeNode()?.getText() ?? "",
          optional: m.hasQuestionToken(),
        });
      return resolveAliases(sources, out);
    }
  }
  return new Map();
}

function resolveAliases(sources: SourceFile[], members: Map<string, Member>): Map<string, Member> {
  for (const [key, m] of members) {
    const named = /^[A-Z]\w*$/.test(m.type) ? aliasText(sources, m.type) : undefined;
    if (named && unionOptions(named)) members.set(key, { ...m, type: named });
  }
  return members;
}

export const rowTypeName = (docType: string | undefined): string | undefined =>
  /^\s*(?:Array<)?([A-Z]\w*)(?:>|\[\])\s*$/.exec(docType ?? "")?.[1];

function columnFor(key: string, value: unknown, member: Member | undefined): ControlSpec | null {
  const description = humanize(key);
  const options = member ? unionOptions(member.type) : null;
  if (options)
    return { type: "enum", options, default: isString(value) ? value : options[0], description };
  const type = member?.type ?? "";
  if (isString(value) || type === "string") {
    const text = isString(value) ? value : "";
    if (isColorProp(key) || HEX.test(text)) return { type: "color", default: text, description };
    return { type: "text", default: text, description };
  }
  if (isNumber(value) || type === "number")
    return { type: "number", default: isNumber(value) ? value : 0, description };
  if (typeof value === "boolean" || type === "boolean")
    return { type: "boolean", default: value === true, description };
  if (isColorList(key, value)) return { type: "colors", min: 1, default: [], description };
  if (Array.isArray(value) && value.every(isString))
    return { type: "strings", default: [], description };
  if (Array.isArray(value) && value.every(isNumber))
    return { type: "numbers", default: [], description };
  return null;
}

export type Columns = { columns: Record<string, ControlSpec>; problems: string[] };

export function inferColumns(
  rows: unknown[],
  members: Map<string, Member>,
  explicit: Record<string, ControlSpec> = {},
): Columns {
  const records = rows.filter(isRecord);
  const keys = [
    ...new Set([
      ...Object.keys(explicit),
      ...records.flatMap((r) => Object.keys(r)),
      ...members.keys(),
    ]),
  ];
  const columns: Record<string, ControlSpec> = {};
  const problems: string[] = [];
  for (const key of keys) {
    const given = explicit[key];
    const sample = records.findLast((r) => r[key] !== undefined)?.[key];
    const member = members.get(key);
    const column = given ?? columnFor(key, sample, member);
    if (!column) {
      if (sample !== undefined) problems.push(`column "${key}" has a value the editor can't show`);
      continue;
    }
    const missing = records.some((r) => r[key] === undefined);
    const optional = column.optional ?? (member?.optional === true || missing);
    columns[key] = {
      ...column,
      ...(optional ? { optional: true } : {}),
      ...(optional && column.type === "color" ? { emptyLabel: column.emptyLabel ?? "None" } : {}),
    };
  }
  return { columns, problems };
}
