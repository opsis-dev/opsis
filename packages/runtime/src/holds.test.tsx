import { act, render } from "@testing-library/react";
import { useLayoutEffect } from "react";
import { describe, expect, it } from "vitest";
import { holdRender, openHolds, renderFailure, settled, useHeldLoad } from "#runtime/holds.ts";

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
    const file = deferred();
    const view = render(<Loader id="a" load={() => file.promise} />);
    expect(openHolds()).toEqual(["file a"]);
    await act(async () => file.resolve("data"));
    expect(openHolds()).toEqual(["canvas"]);
    view.unmount();
    expect(openHolds()).toEqual([]);
  });

  it("releases the old hold when the key changes and when the component goes away", async () => {
    const first = deferred();
    const second = deferred();
    function load(key: string) {
      return key === "a" ? first.promise : second.promise;
    }
    const view = render(<Loader id="a" load={load} />);
    view.rerender(<Loader id="b" load={load} />);
    expect(openHolds()).toEqual(["file b"]);
    await act(async () => first.resolve("late"));
    expect(openHolds()).toEqual(["file b"]);
    view.unmount();
    expect(openHolds()).toEqual([]);
  });

  it("settles once every hold is released, and a release only counts once", async () => {
    const release = holdRender("frame");
    let done = false;
    const waited = settled().then(() => {
      done = true;
    });
    await Promise.resolve();
    expect(done).toBe(false);
    const other = holdRender("other");
    release();
    release();
    await Promise.resolve();
    expect(done).toBe(false);
    other();
    await waited;
    expect(done).toBe(true);
  });

  it("fails the render when loading fails, and every later wait with it", async () => {
    render(<Loader id="x" load={() => Promise.reject(new Error("broken file"))} />);
    await act(async () => undefined);
    expect(renderFailure()?.message).toBe("broken file");
    await expect(settled()).rejects.toThrow("broken file");
  });
});
