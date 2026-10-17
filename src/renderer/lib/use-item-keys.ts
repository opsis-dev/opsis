import { useState } from "react";
import { randomId } from "@/shared/ids";

export type KeyedItem<T> = { item: T; key: string };

export const newKeyedItem = <T>(item: T): KeyedItem<T> => ({ item, key: randomId() });

export function useItemKeys<T>(items: readonly T[]) {
  const [keys, setKeys] = useState(() => items.map(() => randomId()));
  const rows = items.map((item, i): KeyedItem<T> => ({ item, key: keys[i] ?? randomId() }));
  if (keys.length !== items.length) setKeys(rows.map((r) => r.key));
  return {
    rows,
    commit: (next: readonly KeyedItem<T>[]): T[] => {
      setKeys(next.map((r) => r.key));
      return next.map((r) => r.item);
    },
  };
}
