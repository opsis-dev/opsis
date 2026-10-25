import { afterEach, describe, expect, it, vi } from "vitest";
import { pendingReplies } from "#core/pending-replies";

afterEach(() => {
  vi.useRealTimers();
});

describe("pendingReplies", () => {
  it("answers the request with the matching id", async () => {
    const replies = pendingReplies<string>();
    const a = replies.wait("a");
    const b = replies.wait("b");
    replies.settle("b", "for b");
    replies.settle("a", "for a");
    expect(await Promise.all([a, b])).toEqual(["for a", "for b"]);
  });

  it("answers every waiting request when the other side goes away", async () => {
    const replies = pendingReplies<string>();
    const waiting = [replies.wait("a"), replies.wait("b")];
    replies.settleAll("gone");
    expect(await Promise.all(waiting)).toEqual(["gone", "gone"]);
  });

  it("gives the timeout answer when no reply comes, and ignores a late reply", async () => {
    vi.useFakeTimers();
    const replies = pendingReplies<string>();
    const reply = replies.wait("a", { ms: 1000, result: "late" });
    vi.advanceTimersByTime(1000);
    replies.settle("a", "too late");
    expect(await reply).toBe("late");
  });
});
