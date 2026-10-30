import * as fc from "fast-check";
import { describe, expect, it } from "vitest";
import {
  listsFrom,
  nestedFrom,
  ticksFromWords,
  treeFromPaths,
  type TreeNode,
} from "#video/lib/settings/shape.ts";

const word = fc.stringMatching(/^[a-z]{1,5}$/);
const value = fc.oneof(
  fc.string({ maxLength: 6 }),
  fc.integer(),
  fc.boolean(),
  fc.array(fc.integer(), { maxLength: 3 }),
);

function at(target: unknown, path: string[]): unknown {
  return path.reduce<unknown>(
    (node, key) => (node as Record<string, unknown> | undefined)?.[key],
    target,
  );
}

describe("nestedFrom", () => {
  it("puts every set value at its path, keeps the other settings and leaves unset ones out", () => {
    fc.assert(
      fc.property(
        fc.uniqueArray(fc.stringMatching(/^[a-y][a-z]{0,4}$/), { minLength: 1, maxLength: 6 }),
        fc.array(fc.option(value, { nil: undefined }), { minLength: 6, maxLength: 6 }),
        fc.dictionary(fc.stringMatching(/^z[a-z]{1,4}$/), value, { maxKeys: 3 }),
        (keys, values, others) => {
          const paths = Object.fromEntries(keys.map((k) => [k, `group.${k}.value`]));
          const props = { ...others, ...Object.fromEntries(keys.map((k, i) => [k, values[i]])) };
          const nested = nestedFrom(props, paths);
          keys.forEach((k, i) => {
            expect(Object.hasOwn(nested, k)).toBe(false);
            expect(at(nested, ["group", k, "value"])).toEqual(values[i]);
            expect(values[i] === undefined ? at(nested, ["group", k]) : undefined).toBeUndefined();
          });
          for (const [k, v] of Object.entries(others)) expect(nested[k]).toEqual(v);
        },
      ),
    );
  });
});

describe("listsFrom", () => {
  const lists = { bars: { label: "labels", value: "data" } };
  const row = fc.record({ label: fc.oneof(fc.constant(""), word), value: fc.integer() });

  it("splits rows into one list per column and leaves out a column empty in every row", () => {
    fc.assert(
      fc.property(fc.array(row, { maxLength: 8 }), fc.integer(), (bars, kept) => {
        const out = listsFrom({ bars, kept }, lists);
        expect(Object.hasOwn(out, "bars")).toBe(false);
        expect(out.kept).toBe(kept);
        expect(out.data).toEqual(bars.length > 0 ? bars.map((b) => b.value) : undefined);
        const labels = bars.map((b) => b.label);
        expect(out.labels).toEqual(labels.some(Boolean) ? labels : undefined);
      }),
    );
  });
});

describe("ticksFromWords", () => {
  it("turns yes and no in any case into ticks and dashes and keeps every other cell", () => {
    const cell = fc.oneof(
      fc.constantFrom("Yes", "yes", " YES ", "No", "no ", "NO"),
      fc.stringMatching(/^[0-9a-z ]{0,4}$/),
      fc.integer(),
    );
    fc.assert(
      fc.property(
        fc.array(fc.record({ label: word, cells: fc.array(cell, { maxLength: 5 }) }), {
          maxLength: 5,
        }),
        (rows) => {
          const ticked = ticksFromWords(rows);
          expect(ticked.map((r) => r.label)).toEqual(rows.map((r) => r.label));
          ticked.forEach((r, i) => {
            rows[i].cells.forEach((c, j) => {
              const plain = typeof c === "string" ? c.trim().toLowerCase() : null;
              const want = plain === "yes" ? true : plain === "no" ? false : c;
              expect((r.cells as unknown[])[j]).toEqual(want);
            });
          });
        },
      ),
    );
  });
});

describe("treeFromPaths", () => {
  function files(nodes: TreeNode[], prefix = ""): string[] {
    return nodes.flatMap((n) =>
      n.children ? files(n.children, `${prefix}${n.name}/`) : [`${prefix}${n.name}`],
    );
  }

  it("builds folders from paths with every file once where its path puts it", () => {
    fc.assert(
      fc.property(
        fc.array(fc.array(word, { minLength: 1, maxLength: 4 }), { maxLength: 10 }),
        (lists) => {
          const paths = lists.map((parts) => parts.join("/"));
          const tree = treeFromPaths(paths);
          const folders = new Set(
            lists.flatMap((parts) =>
              parts.slice(0, -1).map((_, i) => parts.slice(0, i + 1).join("/")),
            ),
          );
          const expected = [...new Set(paths)].filter((p) => !folders.has(p)).toSorted();
          expect(files(tree).toSorted()).toEqual(expected);
          function unique(nodes: TreeNode[]): boolean {
            return (
              new Set(nodes.map((n) => n.name)).size === nodes.length &&
              nodes.every((n) => unique(n.children ?? []))
            );
          }
          expect(unique(tree)).toBe(true);
        },
      ),
    );
    expect(treeFromPaths(["docs/", "README.md"])).toEqual([
      { name: "docs", children: [] },
      { name: "README.md" },
    ]);
  });
});
