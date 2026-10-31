import { useEffect } from "react";
import { create } from "zustand";
import { on } from "@/app/tauri";
import { commands, events, type EngineStatus, type EnvStatus } from "@/generated/bindings";
import { loadOrReport } from "@/lib/toast";

type EnvironmentState = {
  env: EnvStatus | null;
  refresh: () => Promise<EnvStatus | null>;
};

export function needsEngine(engine: EngineStatus): boolean {
  return !engine.browser.path || !!engine.browser.problem;
}

export const useEnvironment = create<EnvironmentState>()((set) => ({
  env: null,
  refresh: async () => {
    const env = await loadOrReport(
      () => commands.getEnvironment(),
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
    return on(events.engineChanged, (engine) =>
      useEnvironment.setState((s) => (s.env ? { env: { ...s.env, engine } } : s)),
    );
  }, []);
  return env;
}
