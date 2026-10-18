import { act, cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useDecimalInput } from "./form/decimal";
import { useAutosave } from "./use-autosave";

const editor = vi.hoisted(() => {
  const state: { reports: boolean[]; flush: (() => Promise<void>) | null } = {
    reports: [],
    flush: null,
  };
  Object.assign(window, {
    demoflow: {
      platform: "linux",
      reportUnsaved: (unsaved: boolean) => state.reports.push(unsaved),
      onFlushSave: (listener: () => Promise<void>) => {
        state.flush = listener;
        return () => undefined;
      },
    },
  });
  return state;
});

afterEach(cleanup);

function Editor({ apply }: { apply: (text: string) => void }) {
  useAutosave();
  const input = useDecimalInput("1", "blur", apply);
  return <input aria-label="Seconds" {...input} />;
}

describe("closing the window with a value still being typed", () => {
  it("reports the typed value as unsaved, then commits it before saving", async () => {
    const applied: string[] = [];
    const view = render(<Editor apply={(text) => applied.push(text)} />);
    expect(editor.reports.at(-1)).toBe(false);
    fireEvent.change(view.getByLabelText("Seconds"), { target: { value: "2.5" } });
    expect(editor.reports.at(-1)).toBe(true);
    expect(applied).toEqual([]);
    await act(async () => {
      await editor.flush?.();
    });
    expect(applied).toEqual(["2.5"]);
    expect(editor.reports.at(-1)).toBe(false);
  });
});
