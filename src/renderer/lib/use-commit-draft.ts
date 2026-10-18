import { useCallback, useEffect, useRef, useState, useSyncExternalStore } from "react";
import { listenerSet } from "@/shared/listener-set";

type Pending = { text: string; commit: (text: string) => void };

type PendingRef = { current: Pending | null };

const held = new Set<PendingRef>();

const heldChanges = listenerSet();

function hold(ref: PendingRef, next: Pending | null): void {
  ref.current = next;
  const before = held.size;
  if (next) held.add(ref);
  else held.delete(ref);
  if (held.size !== before) heldChanges.emit();
}

function release(ref: PendingRef): void {
  const unsaved = ref.current;
  hold(ref, null);
  unsaved?.commit(unsaved.text);
}

export function commitHeldDrafts(): void {
  for (const ref of held) release(ref);
}

const anyHeld = () => held.size > 0;

export const useHeldDrafts = (): boolean => useSyncExternalStore(heldChanges.add, anyHeld);

export function useCommitDraft(
  value: string,
  onCommit: (text: string) => void,
  applyOn: "edit" | "blur" = "blur",
) {
  const [draft, setDraft] = useState<string | null>(null);
  const pending = useRef<Pending | null>(null);
  const flush = useCallback(() => release(pending), []);
  useEffect(() => {
    window.addEventListener("pagehide", flush);
    return () => {
      window.removeEventListener("pagehide", flush);
      flush();
    };
  }, [flush]);
  const commitText = (text: string) => {
    setDraft(null);
    hold(pending, null);
    if (text !== value) onCommit(text);
  };
  return {
    text: draft ?? value,
    change: (text: string) => {
      setDraft(text);
      if (applyOn === "edit") onCommit(text);
      else hold(pending, text === value ? null : { text, commit: onCommit });
    },
    commit: () => {
      if (applyOn === "blur" && draft !== null) commitText(draft);
      else setDraft(null);
    },
    commitText,
  };
}
