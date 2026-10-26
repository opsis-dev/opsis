import { useEffect } from "react";
import { useRunStore } from "../store/run";
import { showDocument } from "../store/workflow";

export function useDocumentMirror(): void {
  useEffect(() => {
    const off = window.opsis.onDocumentChanged(({ document, edited }) => {
      showDocument(document);
      if (edited.length) useRunStore.getState().forgetErrors(edited);
    });
    void window.opsis.documentState().then((state) => {
      if (state) showDocument(state);
    });
    return off;
  }, []);
}
