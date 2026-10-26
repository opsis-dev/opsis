import { create } from "zustand";
import { commands } from "@/generated/bindings";
import { loadOrReport } from "@/lib/toast";

export const useSecretsStore = create<{
  names: string[] | null;
  refresh: (dir: string | null) => Promise<void>;
}>()((set) => ({
  names: null,
  refresh: async (dir) =>
    set({
      names: dir
        ? await loadOrReport(() => commands.listSecrets(dir), "Couldn't read the saved sign-ins")
        : null,
    }),
}));
