import { useEffectEvent, useLayoutEffect, useRef, useState } from "react";
import { cancelRender, continueRender, delayRender } from "remotion";

export function holdRender(label: string): () => void {
  const handle = delayRender(label);
  let held = true;
  return () => {
    if (!held) return;
    held = false;
    continueRender(handle);
  };
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
      .catch((error: unknown) => cancelRender(error));
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
