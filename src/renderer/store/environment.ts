import { useEffect } from "react";
import { create } from "zustand";
import { loadOrReport } from "@/renderer/lib/toast";
import type { EnvStatus } from "@/shared/ipc";

type EnvironmentState = {
  env: EnvStatus | null;
  refresh: () => Promise<EnvStatus | null>;
};

export const useEnvironment = create<EnvironmentState>()((set) => ({
  env: null,
  refresh: async () => {
    const env = await loadOrReport(
      () => window.demoflow.getEnvironment(),
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
  }, []);
  return env;
}
