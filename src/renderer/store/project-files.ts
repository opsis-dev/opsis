import { create } from "zustand";
import { fileUrl } from "../editor/file-url";

const isThere = (ref: string): Promise<boolean> =>
  fetch(fileUrl(ref.trim()), { method: "HEAD" }).then(
    (reply) => reply.ok,
    () => false,
  );

export const useProjectFiles = create<{
  gone: Partial<Record<string, boolean>>;
  check: (refs: readonly string[]) => Promise<void>;
  checkAgain: () => Promise<void>;
  forget: () => void;
}>()((set, get) => ({
  gone: {},
  check: async (refs) => {
    const found = await Promise.all(refs.map(async (ref) => [ref, !(await isThere(ref))] as const));
    const changed = found.filter(([ref, gone]) => get().gone[ref] !== gone);
    if (changed.length) set((s) => ({ gone: { ...s.gone, ...Object.fromEntries(changed) } }));
  },
  checkAgain: () => get().check(Object.keys(get().gone)),
  forget: () => set({ gone: {} }),
}));
