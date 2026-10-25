import { describe, expect, it, vi } from "vitest";
import { onAbort, withoutRemotePrefix } from "#core/errors";

describe("onAbort", () => {
  it("runs once on abort and not after it is stopped", () => {
    const first = new AbortController();
    const fn = vi.fn();
    onAbort(first.signal, fn);
    first.abort();
    first.abort();
    expect(fn).toHaveBeenCalledTimes(1);

    const second = new AbortController();
    const stopped = vi.fn();
    onAbort(second.signal, stopped)();
    second.abort();
    expect(stopped).not.toHaveBeenCalled();
  });

  it("does nothing without a signal", () => {
    expect(() => onAbort(undefined, () => undefined)()).not.toThrow();
  });
});

describe("withoutRemotePrefix", () => {
  it("leaves the main process's own message", () => {
    expect(
      withoutRemotePrefix(
        "Error invoking remote method 'opsis:fonts:catalog': Error: Couldn't load the font list.",
      ),
    ).toBe("Couldn't load the font list.");
    expect(withoutRemotePrefix("Error invoking remote method 'x': TypeError: bad")).toBe("bad");
    expect(withoutRemotePrefix("Plain message.")).toBe("Plain message.");
  });
});
