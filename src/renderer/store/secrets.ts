import { create } from "zustand";
import { loadOrReport } from "@/renderer/lib/toast";

export const useSecretsStore = create<{
  names: string[] | null;
  refresh: (dir: string | null) => Promise<void>;
}>()((set) => ({
  names: null,
  refresh: async (dir) =>
    set({
      names: dir
        ? await loadOrReport(
            () => window.opsis.listSecrets(dir),
            "Couldn't read the saved sign-ins",
          )
        : null,
    }),
}));
