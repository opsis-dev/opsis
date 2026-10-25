import { act, cleanup, renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { commitHeldDrafts, useCommitDraft, useHeldDrafts } from "./use-commit-draft";

afterEach(cleanup);

function draftHook(applyOn: "edit" | "blur") {
  const commits: string[] = [];
  const hook = renderHook(() => useCommitDraft("old", (text) => commits.push(text), applyOn));
  return { commits, ...hook };
}

describe("useCommitDraft", () => {
  it("commits a typed value when the field goes away before it loses focus", () => {
    const { commits, result, unmount } = draftHook("blur");
    act(() => result.current.change("new"));
    expect(commits).toEqual([]);
    unmount();
    expect(commits).toEqual(["new"]);
  });

  it("commits a typed value when the page hides, and only once", () => {
    const { commits, result, unmount } = draftHook("blur");
    act(() => result.current.change("new"));
    act(() => void window.dispatchEvent(new Event("pagehide")));
    unmount();
    expect(commits).toEqual(["new"]);
  });

  it("doesn't commit again after the field lost focus", () => {
    const { commits, result, unmount } = draftHook("blur");
    act(() => result.current.change("new"));
    act(() => result.current.commit());
    unmount();
    expect(commits).toEqual(["new"]);
  });

  it("holds nothing once the typed text is back to the saved value", () => {
    const { commits, result, unmount } = draftHook("blur");
    const held = renderHook(() => useHeldDrafts());
    act(() => result.current.change("new"));
    expect(held.result.current).toBe(true);
    act(() => result.current.change("old"));
    expect(held.result.current).toBe(false);
    unmount();
    expect(commits).toEqual([]);
  });

  it("commits every held value on request, as the save before closing does", () => {
    const first = draftHook("blur");
    const second = draftHook("blur");
    act(() => first.result.current.change("one"));
    act(() => second.result.current.change("two"));
    act(() => commitHeldDrafts());
    expect([...first.commits, ...second.commits]).toEqual(["one", "two"]);
  });

  it("applies every edit at once when told to, and nothing more on the way out", () => {
    const { commits, result, unmount } = draftHook("edit");
    act(() => result.current.change("ne"));
    act(() => result.current.change("new"));
    unmount();
    expect(commits).toEqual(["ne", "new"]);
  });
});
