import { useState } from "react";
import { randomId } from "@/shared/ids";

type KeyedItem<T> = { item: T; key: string };

export function useItemKeys<T>(items: readonly T[], order?: (a: T, b: T) => number) {
  const [keys, setKeys] = useState(() => items.map(() => randomId()));
  const rows = items.map((item, i): KeyedItem<T> => ({ item, key: keys[i] ?? randomId() }));
  if (keys.length !== items.length) setKeys(rows.map((r) => r.key));
  const commit = (next: KeyedItem<T>[]): T[] => {
    const arranged = order ? next.toSorted((a, b) => order(a.item, b.item)) : next;
    setKeys(arranged.map((r) => r.key));
    return arranged.map((r) => r.item);
  };
  return {
    rows,
    add: (item: T): T[] => commit([...rows, { item, key: randomId() }]),
    update: (index: number, item: T): T[] =>
      commit(rows.map((r, i) => (i === index ? { ...r, item } : r))),
    remove: (index: number): T[] => commit(rows.toSpliced(index, 1)),
    insert: (index: number, added: T[]): T[] =>
      commit(rows.toSpliced(index, 0, ...added.map((item) => ({ item, key: randomId() })))),
    reorder: (keyOrder: string[]): T[] =>
      commit(keyOrder.flatMap((key) => rows.find((r) => r.key === key) ?? [])),
  };
}
