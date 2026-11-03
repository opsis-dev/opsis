import { describe, expect, it } from "vitest";
import { frameAddress, terminalTitle } from "#video/camera/frame-address.ts";
import type { ActionLog } from "#video/generated/spec.ts";

const log: ActionLog = {
  viewport: { width: 1280, height: 720 },
  dpr: 1,
  t0: 0,
  durationMs: 9000,
  actions: [{ id: "nav", kind: "navigate", t: 0, label: "http://localhost:3000/inbox" }],
  addresses: [
    { t: 0, url: "http://localhost:3000/inbox" },
    { t: 4000, url: "http://localhost:3000/settings#billing" },
  ],
};

describe("the address in the window frame", () => {
  it("shows the take's first page when no address is typed", () => {
    expect(frameAddress(log, 6000)).toBe("localhost:3000/inbox");
    expect(frameAddress(log, 6000, "  ")).toBe("localhost:3000/inbox");
  });

  it("keeps a typed address for the whole take", () => {
    expect(frameAddress(log, 0, "https://app.example.com", "fixed")).toBe("app.example.com");
    expect(frameAddress(log, 6000, "app.example.com/home", "fixed")).toBe("app.example.com/home");
  });

  it("puts the typed host in front of the path of the page on screen", () => {
    function at(ms: number) {
      return frameAddress(log, ms, "https://app.example.com/ignored", "host");
    }
    expect(at(1000)).toBe("app.example.com/inbox");
    expect(at(3999)).toBe("app.example.com/inbox");
    expect(at(4000)).toBe("app.example.com/settings#billing");
  });

  it("falls back to the first page's path when the take logged no addresses", () => {
    const old = { ...log, addresses: undefined };
    expect(frameAddress(old, 6000, "app.example.com", "host")).toBe("app.example.com/inbox");
  });

  it("shows just the host on a page without a path", () => {
    const bare = { ...log, actions: [], addresses: [{ t: 0, url: "about:srcdoc" }] };
    expect(frameAddress(bare, 0, "app.example.com", "host")).toBe("app.example.com");
  });
});

describe("terminal titles", () => {
  const titled = {
    ...log,
    titles: [
      { t: 0, title: "zsh" },
      { t: 2000, title: "vim notes.md" },
    ],
  };

  it("shows the program's title at the time and the typed text over it", () => {
    expect(terminalTitle(titled, 500)).toBe("zsh");
    expect(terminalTitle(titled, 2500)).toBe("vim notes.md");
    expect(terminalTitle(titled, 2500, " my-app ")).toBe("my-app");
    expect(terminalTitle({ ...log, titles: [] }, 100)).toBeUndefined();
  });
});
