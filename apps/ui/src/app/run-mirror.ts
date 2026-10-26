import { useEffect } from "react";
import { addLogLine, showRunState, showRunView } from "../store/run";

export function useRunMirror(): void {
  useEffect(() => {
    const offs = [
      window.opsis.onRunViewChanged(showRunView),
      window.opsis.onRunLogged(addLogLine),
      window.opsis.onRunStateReset(showRunState),
    ];
    void window.opsis.runState().then(showRunState);
    return () => {
      for (const off of offs) off();
    };
  }, []);
}
