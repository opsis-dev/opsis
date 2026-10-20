import { describe, expect, it, vi } from "vitest";
import { defaultParams } from "../../shared/nodes/registry";
import type { SessionStep } from "../../shared/schema/values";
import { workflowNode } from "../../shared/test-fixtures";
import { restoreLocalStorage } from "./page-scripts";
import { atTakeStart, type TakeRun } from "./take-run";
import { clickKindOf, runStep, videoStart } from "./take-steps";

const sessionStep = (
  type: string,
  params: Record<string, unknown>,
  id = "n",
  label?: string,
): SessionStep => ({ id, type, label, params: { ...defaultParams(type), ...params } });

function fakeTake(pageUrl: string, saved: string | null) {
  const context = { addCookies: vi.fn(), addInitScript: vi.fn() };
  const page = { url: () => pageUrl, goto: vi.fn(), evaluate: vi.fn() };
  const take = {
    opts: { secrets: { read: async () => saved } },
    session: { context },
    page,
    signedIn: null,
    elapsed: () => 0,
    pushAction: () => undefined,
    settleAfterPage: async () => undefined,
  };
  return { take: take as unknown as TakeRun, context, page };
}

const SIGN_IN = JSON.stringify({
  cookies: [
    {
      name: "sid",
      value: "1",
      domain: "x.test",
      path: "/",
      expires: -1,
      httpOnly: true,
      secure: true,
      sameSite: "Lax",
    },
  ],
  origins: [{ origin: "https://x.test", localStorage: [{ name: "token", value: "t" }] }],
});

describe("signing in during a take", () => {
  const login = sessionStep(
    "browser.signIn",
    { url: "https://x.test/login", secretName: "shop" },
    "login",
    "Sign in to the shop",
  );

  it("adds the saved cookies and page storage, now and for later pages", async () => {
    const { take, context, page } = fakeTake("https://x.test/home", SIGN_IN);
    await runStep(take, login);
    expect(context.addCookies).toHaveBeenCalledWith([expect.objectContaining({ name: "sid" })]);
    const storage = {
      key: "opsis-sign-in:login",
      origin: "https://x.test",
      items: [{ name: "token", value: "t" }],
    };
    expect(context.addInitScript).toHaveBeenCalledWith(restoreLocalStorage, storage);
    expect(page.evaluate).toHaveBeenCalledWith(restoreLocalStorage, storage);
    expect(take.signedIn).toEqual({ url: "https://x.test/login", name: "Sign in to the shop" });
  });

  it("leaves another site's page alone until a page of the site loads", async () => {
    const { take, page } = fakeTake("about:blank", SIGN_IN);
    await runStep(take, login);
    expect(page.evaluate).not.toHaveBeenCalled();
  });

  it("asks for the sign-in when it isn't saved", async () => {
    const { take } = fakeTake("about:blank", null);
    await expect(runStep(take, login)).rejects.toThrow(
      "The sign-in “shop” isn't saved yet. Run “Sign in to the shop” first.",
    );
  });

  it("fails the take with the Sign in node's name when a page lands on the sign-in page", async () => {
    const { take } = fakeTake("https://x.test/login?next=app", SIGN_IN);
    take.signedIn = { url: "https://x.test/login", name: "Sign in to the shop" };
    await expect(
      runStep(take, sessionStep("browser.navigate", { url: "https://x.test/app", after: 0 })),
    ).rejects.toThrow("Signed out during the take. Run “Sign in to the shop” again.");
  });
});

describe("where the video starts", () => {
  it("leaves the actions before Start recording out of the video", () => {
    const steps = [
      workflowNode("browser.signIn", {}, "login"),
      workflowNode("browser.navigate", { url: "https://example.com" }, "go"),
      workflowNode("browser.wait", { mode: "element", target: "Board" }, "wait"),
      workflowNode("browser.startRecording", {}, "start"),
      workflowNode("browser.click", { target: "Open" }, "open"),
      workflowNode("browser.startRecording", {}, "again"),
    ];
    const start = videoStart(steps);
    expect(steps.slice(0, start).map((s) => s.id)).toEqual(["login", "go", "wait"]);
  });

  it("films from the first action without Start recording", () => {
    const steps = [
      workflowNode("browser.navigate", { url: "https://example.com" }),
      workflowNode("browser.click", { target: "Open" }),
    ];
    expect(videoStart(steps)).toBe(0);
  });
});

describe("clickKindOf", () => {
  it.each([
    ["browser.click", "click"],
    ["browser.type", "type"],
    ["browser.hover", "hover"],
    ["browser.drag", "drag"],
    ["browser.select", "focus"],
  ])("logs a %s as a %s", (type, kind) => {
    expect(clickKindOf(type)).toBe(kind);
  });
});

describe("actions before the camera rolls", () => {
  it("sit at the start of the take", () => {
    expect(atTakeStart({ id: "c:chapter", kind: "chapter", t: 3008, label: "Intro" })).toEqual({
      id: "c:chapter",
      kind: "chapter",
      t: 0,
      label: "Intro",
    });
    expect(atTakeStart({ id: "n:nav", kind: "navigate", t: 3009, tEnd: 3037 })).toEqual({
      id: "n:nav",
      kind: "navigate",
      t: 0,
      tEnd: 0,
    });
  });
});
