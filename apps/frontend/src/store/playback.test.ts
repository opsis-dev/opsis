import { afterEach, describe, expect, it } from "vitest";
import { registerPlayer, usePlayback } from "./playback";

class CountingPlayer {
  calls: string[] = [];

  seek(frame: number): void {
    this.calls.push(`seek ${frame}`);
  }

  play(): void {
    this.calls.push("play");
  }

  pause(): void {
    this.calls.push("pause");
  }
}

describe("registerPlayer", () => {
  afterEach(() => registerPlayer(null));

  it("drives a player whose methods live on its class", () => {
    const player = new CountingPlayer();
    registerPlayer(player);
    usePlayback.setState({ durationInFrames: 100 });
    usePlayback.getState().seek(12);
    usePlayback.getState().play();
    usePlayback.getState().pause();
    expect(player.calls).toEqual(["seek 12", "play", "pause"]);
    expect(usePlayback.getState().hasPlayer).toBe(true);
  });
});
