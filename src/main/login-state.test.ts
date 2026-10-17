import type { Cookie } from "electron";
import { describe, expect, it } from "vitest";
import { storageState } from "./login-state";

const cookie = (over: Partial<Cookie>): Cookie => ({
  name: "sid",
  value: "abc",
  sameSite: "lax",
  ...over,
});

describe("storageState", () => {
  it("writes host-only cookies without a leading dot and shared ones with it", () => {
    const state = storageState(
      [
        cookie({ domain: ".app.test", hostOnly: true }),
        cookie({ domain: "app.test", hostOnly: false }),
      ],
      new Map(),
    );
    expect(state.cookies.map((c) => c.domain)).toEqual(["app.test", ".app.test"]);
  });

  it("keeps session cookies open-ended and maps same-site names", () => {
    const [session, dated] = storageState(
      [
        cookie({ session: true, expirationDate: 99, sameSite: "strict" }),
        cookie({ expirationDate: 1234, sameSite: "no_restriction", path: "/app" }),
      ],
      new Map(),
    ).cookies;
    expect(session).toMatchObject({ expires: -1, sameSite: "Strict", path: "/" });
    expect(dated).toMatchObject({ expires: 1234, sameSite: "None", path: "/app" });
  });

  it("lists local storage per origin", () => {
    const state = storageState([], new Map([["https://app.test", [{ name: "k", value: "v" }]]]));
    expect(state.origins).toEqual([
      { origin: "https://app.test", localStorage: [{ name: "k", value: "v" }] },
    ]);
  });
});
