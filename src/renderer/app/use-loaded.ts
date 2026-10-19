import { useEffect, useEffectEvent, useState } from "react";
import { reportError } from "@/renderer/lib/toast";

export function useLoaded<T>(load: () => Promise<T>, failure: string, key = ""): T | null {
  const [loaded, setLoaded] = useState<{ key: string; value: T } | null>(null);
  const start = useEffectEvent((forKey: string, current: () => boolean) =>
    load().then(
      (value) => {
        if (current()) setLoaded({ key: forKey, value });
      },
      (e: unknown) => {
        if (current()) reportError(failure, e);
      },
    ),
  );
  useEffect(() => {
    let current = true;
    void start(key, () => current);
    return () => {
      current = false;
    };
  }, [key]);
  return loaded?.key === key ? loaded.value : null;
}

export const useProjectLocations = (key = "") =>
  useLoaded(() => window.opsis.projectLocations(), "Couldn't find the projects folder", key);
