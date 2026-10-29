import { act, cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useDecimalInput } from "./form/decimal";
import { useHandOverDrafts } from "./use-hand-over-drafts";

const editor = vi.hoisted(() => ({
  reports: [] as boolean[],
  saves: 0,
  flushed: 0,
  flush: null as (() => void) | null,
}));

vi.mock("@/generated/bindings", () => ({
  events: { flushSave: "flush-save" },
  commands: {
    reportDrafts: (held: boolean) => Promise.resolve(void editor.reports.push(held)),
    saveFlushed: () => Promise.resolve(void (editor.flushed += 1)),
    saveProject: () => {
      editor.saves += 1;
      return Promise.resolve(true);
    },
  },
}));

vi.mock("@/app/tauri", () => ({
  on: (_event: unknown, listener: () => void) => {
    editor.flush = listener;
    return () => undefined;
  },
}));

afterEach(cleanup);

function DraftHarness({ apply }: { apply: (text: string) => void }) {
  useHandOverDrafts();
  const input = useDecimalInput("1", "blur", apply);
  return <input aria-label="Seconds" {...input} />;
}

describe("closing the window with a value still being typed", () => {
  it("reports the typed value as held, then hands it over before closing", async () => {
    const applied: string[] = [];
    const view = render(<DraftHarness apply={(text) => applied.push(text)} />);
    expect(editor.reports.at(-1)).toBe(false);
    fireEvent.change(view.getByLabelText("Seconds"), { target: { value: "2.5" } });
    expect(editor.reports.at(-1)).toBe(true);
    expect(applied).toEqual([]);
    act(() => editor.flush?.());
    expect(applied).toEqual(["2.5"]);
    expect(editor.flushed).toBe(1);
    expect(editor.reports.at(-1)).toBe(false);
    view.unmount();
    expect(editor.saves).toBe(1);
  });
});
