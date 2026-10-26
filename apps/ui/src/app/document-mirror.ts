import { useEffect } from "react";
import { showDocument } from "../store/workflow";

export function useDocumentMirror(): void {
  useEffect(() => {
    const off = window.opsis.onDocumentChanged(({ document }) => showDocument(document));
    void window.opsis.documentState().then((state) => {
      if (state) showDocument(state);
    });
    return off;
  }, []);
}
