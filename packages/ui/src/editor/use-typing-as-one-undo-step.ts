import { useCallback, useEffect, useRef, type FocusEvent } from "react";
import { isEditingElement } from "@/lib/dom";
import { useWorkflowStore } from "../store/workflow";

type TypingHandlers = { onFocus: (e: FocusEvent) => void; onBlur: (e: FocusEvent) => void };

export function useTypingAsOneUndoStep(): TypingHandlers {
  const typing = useRef(false);
  const stop = useCallback(() => {
    if (!typing.current) return;
    typing.current = false;
    useWorkflowStore.getState().endGesture();
  }, []);
  useEffect(() => stop, [stop]);
  return {
    onFocus: (e: FocusEvent) => {
      if (typing.current || !isEditingElement(e.target)) return;
      typing.current = true;
      useWorkflowStore.getState().beginGesture();
    },
    onBlur: (e: FocusEvent) => {
      if (isEditingElement(e.target)) stop();
    },
  };
}
