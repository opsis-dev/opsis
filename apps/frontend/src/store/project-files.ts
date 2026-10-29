import { create } from "zustand";
import { fileUrl } from "../editor/file-url";

const isThere = (ref: string): Promise<boolean> =>
  fetch(fileUrl(ref.trim()), { method: "HEAD" }).then(
    (reply) => reply.ok,
    () => false,
  );

export type FileVersions = Record<string, string>;

const NO_VERSIONS: FileVersions = {};

export const useProjectFiles = create<{
  gone: Partial<Record<string, boolean>>;
  versions: FileVersions;
  check: (refs: readonly string[]) => Promise<void>;
  checkAgain: () => Promise<void>;
  changed: (files: FileVersions) => void;
  forget: () => void;
}>()((set, get) => ({
  gone: {},
  versions: NO_VERSIONS,
  check: async (refs) => {
    const found = await Promise.all(refs.map(async (ref) => [ref, !(await isThere(ref))] as const));
    const changed = found.filter(([ref, gone]) => get().gone[ref] !== gone);
    if (changed.length) set((s) => ({ gone: { ...s.gone, ...Object.fromEntries(changed) } }));
  },
  checkAgain: () => get().check(Object.keys(get().gone)),
  changed: (files) => {
    set((s) => ({ versions: { ...s.versions, ...files } }));
    void get().checkAgain();
  },
  forget: () => set({ gone: {}, versions: NO_VERSIONS }),
}));

export const versionsOf = (refs: readonly string[], versions: FileVersions): FileVersions =>
  Object.fromEntries(refs.flatMap((ref) => (versions[ref] ? [[ref, versions[ref]]] : [])));

function versionedUrl(ref: string, versions: FileVersions): string {
  const version = versions[ref];
  return version ? `${fileUrl(ref)}?v=${encodeURIComponent(version)}` : fileUrl(ref);
}

export const currentFileUrl = (ref: string): string =>
  versionedUrl(ref, useProjectFiles.getState().versions);

export function useFileUrl(ref: string | null): string | null {
  const versions = useProjectFiles((s) => s.versions);
  return ref ? versionedUrl(ref, versions) : null;
}
