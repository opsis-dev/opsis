import { create } from "zustand";

type StillsState = {
  projectDir: string | null;
  index: Record<string, string>;
  wanted: Record<string, true>;
  engine: boolean | null;
  load: (projectDir: string | null) => Promise<void>;
  checkEngine: () => Promise<void>;
  want: (nodeIds: string[]) => void;
  ready: (projectDir: string, nodeId: string, hash: string) => void;
};

export const useStillsStore = create<StillsState>((set, get) => ({
  projectDir: null,
  index: {},
  wanted: {},
  engine: null,
  load: async (projectDir) => {
    set({ projectDir, index: {}, wanted: {} });
    if (!projectDir) return;
    const index = await window.demoflow.listStills(projectDir);
    if (get().projectDir === projectDir) set({ index: { ...index, ...get().index } });
  },
  checkEngine: async () => {
    const env = await window.demoflow.getEnvironment();
    set({ engine: env.engine.complete });
  },
  want: (nodeIds) => {
    const missing = nodeIds.filter((id) => !get().wanted[id]);
    if (!missing.length) return;
    set({ wanted: { ...get().wanted, ...Object.fromEntries(missing.map((id) => [id, true])) } });
  },
  ready: (projectDir, nodeId, hash) => {
    if (get().projectDir !== projectDir) return;
    set({ index: { ...get().index, [nodeId]: hash } });
  },
}));
