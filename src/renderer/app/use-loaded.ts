import { useEffect, useEffectEvent, useState } from "react";
import { reportError } from "@/renderer/lib/toast";

export function useLoaded<T>(load: () => Promise<T>, failure: string, key = ""): T | null {
  const [loaded, setLoaded] = useState<{ key: string; value: T } | null>(null);
  const start = useEffectEvent((forKey: string) =>
    load().then(
      (value) => setLoaded({ key: forKey, value }),
      (e: unknown) => reportError(failure, e),
    ),
  );
  useEffect(() => {
    void start(key);
  }, [key]);
  return loaded?.key === key ? loaded.value : null;
}

export const useProjectLocations = (key = "") =>
  useLoaded(() => window.demoflow.projectLocations(), "Could not find the projects folder", key);
