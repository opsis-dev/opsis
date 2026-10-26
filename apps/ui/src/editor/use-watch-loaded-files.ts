import { useEffect, useMemo, useRef } from "react";
import { projectRefsIn } from "@/core/named-files";
import type { CompileResult } from "@/generated/bindings";
import { useWorkflowStore } from "../store/workflow";

export function useWatchLoadedFiles(projectDir: string, result: CompileResult | null): void {
  const fonts = useWorkflowStore((s) => s.workflow?.settings.fonts.families);
  const refs = useMemo(() => projectRefsIn([result?.values, fonts]).toSorted(), [result, fonts]);
  const sent = useRef("");
  useEffect(() => {
    const key = [projectDir, ...refs].join("\n");
    if (key === sent.current) return;
    sent.current = key;
    void window.opsis.watchProjectFiles(projectDir, refs);
  }, [projectDir, refs]);
}
