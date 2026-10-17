import { useState } from "react";
import { randomId } from "@/shared/ids";

export function useItemKeys(count: number) {
  const [keys, setKeys] = useState(() => Array.from({ length: count }, () => randomId()));
  if (keys.length !== count)
    setKeys(Array.from({ length: count }, (_, i) => keys[i] ?? randomId()));
  return {
    keys,
    add: () => setKeys([...keys, randomId()]),
    remove: (index: number) => setKeys(keys.toSpliced(index, 1)),
    reorder: (order: readonly number[]) => setKeys(order.map((i) => keys[i] ?? randomId())),
  };
}
