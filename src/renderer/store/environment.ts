import { useEffect } from "react";
import { create } from "zustand";
import { reportError } from "@/renderer/lib/toast";
import type { EnvStatus } from "@/shared/ipc";

type EnvironmentState = {
  env: EnvStatus | null;
  refresh: () => Promise<EnvStatus | null>;
};

export const useEnvironment = create<EnvironmentState>()((set) => ({
  env: null,
  refresh: async () => {
    try {
      const env = await window.demoflow.getEnvironment();
      set({ env });
      return env;
    } catch (e) {
      reportError("Could not read the engine status", e);
      return null;
    }
  },
}));

export function useFreshEnvironment(): EnvStatus | null {
  const env = useEnvironment((s) => s.env);
  useEffect(() => {
    void useEnvironment.getState().refresh();
  }, []);
  return env;
}
