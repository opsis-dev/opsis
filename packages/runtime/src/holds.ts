import { useEffectEvent, useLayoutEffect, useRef, useState } from "react";

const open = new Map<number, string>();
const waiting = new Set<() => void>();
let nextHold = 0;
let failure: Error | null = null;

function wake() {
  if (open.size > 0 && failure === null) return;
  for (const done of waiting) done();
  waiting.clear();
}

export function failRender(error: unknown): void {
  failure ??= error instanceof Error ? error : new Error(String(error));
  wake();
}

export function holdRender(label: string): () => void {
  const id = nextHold;
  nextHold += 1;
  open.set(id, label);
  return () => {
    if (open.delete(id)) wake();
  };
}

export function openHolds(): string[] {
  return [...open.values()];
}

export function renderFailure(): Error | null {
  return failure;
}

export function settled(): Promise<void> {
  return new Promise((resolve, reject) => {
    function done() {
      if (failure) reject(failure);
      else resolve();
    }
    if (open.size === 0 || failure) done();
    else waiting.add(done);
  });
}

type Loaded<T> = { key: string; value: T };

export function useHeldLoad<T>(
  label: string,
  key: string | null,
  load: (key: string) => Promise<T>,
): Loaded<T> | null {
  const [loaded, setLoaded] = useState<Loaded<T> | null>(null);
  const start = useEffectEvent(load);
  const held = useRef<{ key: string; release: () => void } | null>(null);
  useLayoutEffect(() => {
    if (key === null) return;
    const release = holdRender(label);
    held.current = { key, release };
    let alive = true;
    start(key)
      .then((value) => {
        if (alive) setLoaded({ key, value });
      })
      .catch((error: unknown) => failRender(error));
    return () => {
      alive = false;
      held.current = null;
      release();
    };
  }, [key, label]);
  useLayoutEffect(() => {
    if (loaded && held.current?.key === loaded.key) held.current.release();
  }, [loaded]);
  return loaded;
}
