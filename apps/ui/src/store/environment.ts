import { useEffect } from "react";
import { create } from "zustand";
import type { EngineStatus, EnvStatus } from "@/generated/bindings";
import { loadOrReport } from "@/lib/toast";

type EnvironmentState = {
  env: EnvStatus | null;
  refresh: () => Promise<EnvStatus | null>;
};

export const needsEngine = (engine: EngineStatus): boolean =>
  [engine.browser, engine.ffmpeg, engine.ffprobe].some((tool) => !tool.path || !!tool.problem);

export const useEnvironment = create<EnvironmentState>()((set) => ({
  env: null,
  refresh: async () => {
    const env = await loadOrReport(
      () => window.opsis.getEnvironment(),
      "Couldn't read the engine status",
    );
    if (env) set({ env });
    return env;
  },
}));

export function useFreshEnvironment(): EnvStatus | null {
  const env = useEnvironment((s) => s.env);
  useEffect(() => {
    void useEnvironment.getState().refresh();
    return window.opsis.onEngineChanged((engine) =>
      useEnvironment.setState((s) => (s.env ? { env: { ...s.env, engine } } : s)),
    );
  }, []);
  return env;
}
