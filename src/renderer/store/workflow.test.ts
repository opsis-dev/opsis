import { beforeEach, describe, expect, it, vi } from "vitest";
import { getTemplate } from "@/shared/templates";
import { useUiStore } from "./ui";
import { mirrorEditsTo, useWorkflowStore, type RemoteEdit } from "./workflow";

vi.hoisted(() => vi.stubGlobal("window", { opsis: { platform: "linux" } }));

const store = () => useWorkflowStore.getState();
const pastSteps = () => useWorkflowStore.temporal.getState().pastStates.length;
const rename = (name: string) => store().update((wf) => ({ ...wf, name }));
const dirty = () => store().revision !== store().savedRevision;

const project = () => getTemplate("todomvc").build("Start");

describe("workflow store history", () => {
  beforeEach(() => {
    mirrorEditsTo(null);
    store().load("/project", project());
  });

  it("opens a project clean, with no history, even over another open project", () => {
    expect(pastSteps()).toBe(0);
    store().load("/other", project());
    expect(dirty()).toBe(false);
    expect(pastSteps()).toBe(0);
  });

  it("records a gesture as one undo step and counts undo and redo as edits", () => {
    store().beginGesture();
    rename("A");
    rename("AB");
    store().endGesture();
    expect(pastSteps()).toBe(1);
    store().markSaved(store().revision, "v1");

    store().undo();
    expect(store().workflow?.name).toBe("Start");
    expect(dirty()).toBe(true);

    store().markSaved(store().revision, "v2");
    store().redo();
    expect(store().workflow?.name).toBe("AB");
    expect(dirty()).toBe(true);
  });

  it("treats nested gestures as one and records nothing for a gesture without changes", () => {
    store().beginGesture();
    store().beginGesture();
    rename("A");
    store().endGesture();
    rename("B");
    store().endGesture();
    expect(pastSteps()).toBe(1);

    store().beginGesture();
    store().endGesture();
    rename("C");
    expect(pastSteps()).toBe(2);
  });

  it("drops a gesture left open when another project opens", () => {
    store().beginGesture();
    rename("A");
    store().load("/other", project());
    rename("B");
    expect(pastSteps()).toBe(1);
    store().endGesture();
    store().undo();
    expect(store().workflow?.name).toBe("Start");
    expect(pastSteps()).toBe(0);
  });

  it("does not change the revision when undo has nothing to undo", () => {
    const before = store().revision;
    store().undo();
    expect(store().revision).toBe(before);
  });

  it("sends edits and gestures to the editor window while mirroring", () => {
    const sent: RemoteEdit[] = [];
    mirrorEditsTo((edit) => sent.push(edit));
    store().beginGesture();
    rename("A");
    store().endGesture();
    expect(sent.map((e) => (e.type === "gesture" ? e.phase : e.workflow.name))).toEqual([
      "begin",
      "A",
      "end",
    ]);
  });

  it("drops deleted nodes from the selection", () => {
    const [first, second] = store().workflow?.nodes ?? [];
    useUiStore.getState().selectPinned([first.id, second.id], first.id);
    store().removeNodes([first.id]);
    expect(useUiStore.getState().selectedIds).toEqual([second.id]);
    expect(useUiStore.getState().pinnedSource).toBeNull();
  });
});
