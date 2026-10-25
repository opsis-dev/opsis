import type { CompileResult } from "@opsis/core/compile/compile";
import { projectRefsIn } from "@opsis/core/named-files";
import { useEffect, useMemo, useRef } from "react";
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
