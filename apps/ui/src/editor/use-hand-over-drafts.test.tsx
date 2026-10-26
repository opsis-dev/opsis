import { act, cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useDecimalInput } from "./form/decimal";
import { useHandOverDrafts } from "./use-hand-over-drafts";

const editor = vi.hoisted(() => {
  const state: { reports: boolean[]; saves: number; flush: (() => Promise<void>) | null } = {
    reports: [],
    saves: 0,
    flush: null,
  };
  Object.assign(window, {
    opsis: {
      platform: "linux",
      reportDrafts: (held: boolean) => state.reports.push(held),
      saveProject: () => {
        state.saves += 1;
        return Promise.resolve(true);
      },
      onFlushSave: (listener: () => Promise<void>) => {
        state.flush = listener;
        return () => undefined;
      },
    },
  });
  return state;
});

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
    await act(async () => {
      await editor.flush?.();
    });
    expect(applied).toEqual(["2.5"]);
    expect(editor.reports.at(-1)).toBe(false);
    view.unmount();
    expect(editor.saves).toBe(1);
  });
});
