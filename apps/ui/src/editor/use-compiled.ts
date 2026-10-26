import { createContext, useCallback, useContext, useEffect, useRef, useState } from "react";
import type { EditorCompile } from "@/generated/bindings";
import { attempt } from "@/lib/toast";
import { singleSelected, useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";

export type Compiled = Omit<EditorCompile, "seq" | "result"> & {
  result: EditorCompile["result"] | null;
};

const EMPTY: Compiled = {
  result: null,
  issues: [],
  preview: null,
  flat: null,
  plan: null,
  levelled: null,
  cards: {},
  inspector: null,
  clocks: {},
};

function useLatest(): [Compiled, (compiled: EditorCompile) => void] {
  const [compiled, setCompiled] = useState<Compiled>(EMPTY);
  const newest = useRef(0);
  const take = useCallback((next: EditorCompile) => {
    if (next.seq < newest.current) return;
    newest.current = next.seq;
    setCompiled(next);
  }, []);
  return [compiled, take];
}

export function useCompiled({ drive }: { drive: boolean }): Compiled {
  const workflow = useWorkflowStore((s) => s.workflow);
  const selectedId = useUiStore((s) => s.pinnedSource ?? singleSelected(s.selectedIds));
  const inspectedId = useUiStore((s) => singleSelected(s.selectedIds));
  const [compiled, take] = useLatest();
  useEffect(() => window.opsis.onCompiled(take), [take]);
  useEffect(() => {
    if (drive) return;
    void window.opsis.latestCompile().then((latest) => {
      if (latest) take(latest);
    });
  }, [drive, take]);
  useEffect(() => {
    if (!drive || !workflow) return;
    void attempt(
      async () => take(await window.opsis.compile(workflow, selectedId, inspectedId)),
      "Couldn't work out the film",
    );
  }, [drive, workflow, selectedId, inspectedId, take]);
  return workflow ? compiled : EMPTY;
}

export const CompiledContext = createContext<Compiled>(EMPTY);
export const useCompiledContext = () => useContext(CompiledContext);
