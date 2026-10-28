import { isRecord, isString } from "#video/common/guards.ts";
import type { ElementProps } from "#video/common/spec.ts";

function setPath(target: ElementProps, path: string[], value: unknown): void {
  const [head, ...rest] = path;
  if (!head) return;
  if (rest.length === 0) {
    target[head] = value;
    return;
  }
  const inner = isRecord(target[head]) ? { ...target[head] } : {};
  setPath(inner, rest, value);
  target[head] = inner;
}

export function nestedFrom<P extends ElementProps>(props: P, paths: Record<string, string>): ElementProps {
  const out: ElementProps = {};
  for (const [key, value] of Object.entries(props)) if (!Object.hasOwn(paths, key)) out[key] = value;
  for (const [key, path] of Object.entries(paths))
    if (props[key] !== undefined) setPath(out, path.split("."), props[key]);
  return out;
}

export const rowsOf = (value: unknown): ElementProps[] =>
  Array.isArray(value) ? value.filter(isRecord) : [];

export function listsFrom<P extends ElementProps>(
  props: P,
  lists: Record<string, Record<string, string>>,
): ElementProps {
  const out: ElementProps = { ...props };
  for (const [key, columns] of Object.entries(lists)) {
    const rows = rowsOf(props[key]);
    delete out[key];
    for (const [column, prop] of Object.entries(columns)) {
      const values = rows.map((r) => r[column]);
      if (values.some((v) => v !== undefined && v !== "")) out[prop] = values;
    }
  }
  return out;
}

const TICK_WORDS: Partial<Record<string, boolean>> = { yes: true, no: false };

export const ticksFromWords = (rows: unknown): ElementProps[] =>
  rowsOf(rows).map((row) => ({
    ...row,
    cells: Array.isArray(row.cells)
      ? row.cells.map((cell) =>
          isString(cell) ? (TICK_WORDS[cell.trim().toLowerCase()] ?? cell) : cell,
        )
      : row.cells,
  }));

export type TreeNode = { name: string; children?: TreeNode[] };

export function treeFromPaths(paths: readonly string[]): TreeNode[] {
  const root: TreeNode = { name: "", children: [] };
  for (const path of paths) {
    const parts = path.split("/").map((p) => p.trim());
    const folder = parts.at(-1) === "";
    const names = parts.filter(Boolean);
    let node = root;
    names.forEach((name, i) => {
      node.children ??= [];
      let child = node.children.find((c) => c.name === name);
      if (!child) {
        child = { name, ...(i < names.length - 1 || folder ? { children: [] } : {}) };
        node.children.push(child);
      } else if (i < names.length - 1) child.children ??= [];
      node = child;
    });
  }
  return root.children ?? [];
}
