import { describe, expect, it } from "vitest";
import { playbackSnapshot, usePlayback } from "./playback";
import { selection, useUiStore } from "./ui";

function stateKeys(state: object): string[] {
  return Object.entries(state)
    .filter(([, value]) => typeof value !== "function")
    .map(([key]) => key)
    .toSorted();
}

const STORES = [
  {
    name: "UI",
    state: useUiStore.getState,
    mirrored: selection,
    perWindow: ["paletteOpen", "replayNodeId", "replayTab", "reveal"],
  },
  {
    name: "playback",
    state: usePlayback.getState,
    mirrored: playbackSnapshot,
    perWindow: ["hasPlayer"],
  },
];

describe("state mirrored to panel windows", () => {
  it.each(STORES)(
    "the $name store mirrors every field it doesn't keep per window",
    ({ state, mirrored, perWindow }) => {
      expect(stateKeys(state())).toEqual([...Object.keys(mirrored()), ...perWindow].toSorted());
    },
  );
});
