import { useEffect } from "react";
import { on } from "@/app/tauri";
import { commands, events } from "@/generated/bindings";
import { showDocument } from "../store/workflow";

export function useDocumentMirror(): void {
  useEffect(() => {
    const off = on(events.documentChanged, ({ document }) => showDocument(document));
    void commands.documentState().then((state) => {
      if (state) showDocument(state);
    });
    return off;
  }, []);
}
