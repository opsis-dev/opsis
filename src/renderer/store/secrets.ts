import { create } from "zustand";

export const secretKey = (name: string) => name.trim().replace(/[^\w.-]+/g, "-");

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
    } catch {
      set({ names: null });
    }
  },
}));
