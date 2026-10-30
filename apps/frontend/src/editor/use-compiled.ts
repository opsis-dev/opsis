import { createContext, useCallback, useContext, useEffect, useRef, useState } from "react";
import { on } from "@/app/tauri";
import { commands, events, type EditorCompile } from "@/generated/bindings";
import { attempt } from "@/lib/toast";
import { singleSelected, useUiStore } from "../store/ui";
import { useWorkflowStore } from "../store/workflow";

export type Compiled = Omit<EditorCompile, "seq" | "result" | "mix"> & {
  result: EditorCompile["result"] | null;
  mix: EditorCompile["mix"] | null;
};

const EMPTY: Compiled = {
  result: null,
  issues: [],
  film: null,
  flat: null,
  plan: null,
  levelled: null,
  cards: {},
  inputs: {},
  inspector: null,
  localFrames: {},
  mix: null,
  solo: false,
  outdated: [],
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
  useEffect(() => on(events.compiled, take), [take]);
  useEffect(() => {
    void attempt(async () => {
      const latest = await commands.latestCompile();
      if (latest) take(latest);
    }, "Couldn't work out the film");
  }, [take]);
  useEffect(() => {
    if (drive) void commands.setCompileFocus(selectedId, inspectedId);
  }, [drive, selectedId, inspectedId]);
  return workflow ? compiled : EMPTY;
}

export const CompiledContext = createContext<Compiled>(EMPTY);
export function useCompiledContext() {
  return useContext(CompiledContext);
}
