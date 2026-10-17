import { describe, expect, it } from "vitest";
import { PLAYBACK_LOCAL_KEYS, PLAYBACK_SYNC_KEYS, usePlayback } from "./playback";
import { RUN_SYNC_KEYS, useRunStore } from "./run";
import { SELECTION_KEYS, UI_LOCAL_KEYS, useUiStore } from "./ui";

const stateKeys = (state: object): string[] =>
  Object.entries(state)
    .filter(([, value]) => typeof value !== "function")
    .map(([key]) => key)
    .toSorted();

const listed = (...lists: ReadonlyArray<readonly string[]>): string[] => lists.flat().toSorted();

describe("state mirrored to panel windows", () => {
  it("lists every run store field", () => {
    expect(stateKeys(useRunStore.getState())).toEqual(listed(RUN_SYNC_KEYS));
  });

  it("lists every selection field and keeps the rest of the UI state per window", () => {
    expect(stateKeys(useUiStore.getState())).toEqual(listed(SELECTION_KEYS, UI_LOCAL_KEYS));
  });

  it("lists every playback field and keeps the player's presence per window", () => {
    expect(stateKeys(usePlayback.getState())).toEqual(
      listed(PLAYBACK_SYNC_KEYS, PLAYBACK_LOCAL_KEYS),
    );
  });
});
