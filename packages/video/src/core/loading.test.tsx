import { act, render } from "@testing-library/react";
import { useLayoutEffect } from "react";
import { describe, expect, it, vi } from "vitest";
import { holdRender, useHeldLoad } from "#video/core/loading.ts";

const events = vi.hoisted(() => ({ log: [] as string[], labels: new Map<number, string>() }));

vi.mock("remotion", () => {
  let next = 0;
  return {
    delayRender: (label: string) => {
      next += 1;
      events.labels.set(next, label);
      events.log.push(`hold ${label}`);
      return next;
    },
    continueRender: (handle: number) => events.log.push(`release ${events.labels.get(handle)}`),
    cancelRender: (error: unknown) => events.log.push(`cancel ${String(error)}`),
  };
});

function Canvas() {
  useLayoutEffect(() => holdRender("canvas"), []);
  return <p>canvas</p>;
}

function deferred() {
  let resolve: (value: string) => void = () => undefined;
  const promise = new Promise<string>((r) => {
    resolve = r;
  });
  return { promise, resolve };
}

function Loader({ id, load }: { id: string; load: (key: string) => Promise<string> }) {
  const loaded = useHeldLoad(`file ${id}`, id, load);
  return loaded?.key === id ? <Canvas /> : null;
}

describe("holding a render while something loads", () => {
  it("keeps the hold until what the value mounts has taken its own hold", async () => {
    events.log.length = 0;
    const file = deferred();
    render(<Loader id="a" load={() => file.promise} />);
    expect(events.log).toEqual(["hold file a"]);
    await act(async () => file.resolve("data"));
    expect(events.log).toEqual(["hold file a", "hold canvas", "release file a"]);
  });

  it("releases the old hold when the key changes and when the component goes away", async () => {
    events.log.length = 0;
    const first = deferred();
    const second = deferred();
    const load = (key: string) => (key === "a" ? first.promise : second.promise);
    const view = render(<Loader id="a" load={load} />);
    view.rerender(<Loader id="b" load={load} />);
    expect(events.log).toEqual(["hold file a", "release file a", "hold file b"]);
    await act(async () => first.resolve("late"));
    expect(events.log).toEqual(["hold file a", "release file a", "hold file b"]);
    view.unmount();
    expect(events.log.at(-1)).toBe("release file b");
  });

  it("cancels the render when loading fails", async () => {
    events.log.length = 0;
    render(<Loader id="x" load={() => Promise.reject(new Error("broken file"))} />);
    await act(async () => undefined);
    expect(events.log).toContain("cancel Error: broken file");
  });
});
