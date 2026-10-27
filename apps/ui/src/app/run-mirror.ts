import { useEffect } from "react";
import { on } from "@/app/tauri";
import { commands, events } from "@/generated/bindings";
import { addLogLine, showRunState, showRunView } from "../store/run";

export function useRunMirror(active = true): void {
  useEffect(() => {
    if (!active) return;
    const offs = [
      on(events.runViewChanged, showRunView),
      on(events.runLogged, addLogLine),
      on(events.runStateReset, showRunState),
    ];
    void commands.runState().then(showRunState);
    return () => {
      for (const off of offs) off();
    };
  }, [active]);
}
