import { create } from "zustand";
import { reportError } from "@/renderer/lib/toast";

export const useSecretsStore = create<{
  names: string[] | null;
  refresh: (dir: string | null) => Promise<void>;
}>()((set) => ({
  names: null,
  refresh: async (dir) => {
    if (!dir) {
      set({ names: null });
      return;
    }
    try {
      set({ names: await window.demoflow.listSecrets(dir) });
    } catch (e) {
      set({ names: null });
      reportError("Could not read the saved sign-ins", e);
    }
  },
}));
