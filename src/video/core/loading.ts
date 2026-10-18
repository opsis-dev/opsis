import { useEffectEvent, useLayoutEffect, useState } from "react";
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
  useLayoutEffect(() => {
    if (key === null) return;
    const release = holdRender(label);
    let alive = true;
    start(key)
      .then((value) => {
        if (!alive) return;
        setLoaded({ key, value });
        release();
      })
      .catch((error: unknown) => cancelRender(error));
    return () => {
      alive = false;
      release();
    };
  }, [key, label]);
  return loaded;
}
