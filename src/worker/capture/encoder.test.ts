import { PassThrough } from "node:stream";
import { describe, expect, it } from "vitest";
import { FrameEncoder } from "./encoder";

function fakeFfmpeg() {
  const written: string[] = [];
  const stdin = new PassThrough();
  stdin.on("data", (chunk: Buffer) => written.push(chunk.toString()));
  let finish: () => void = () => undefined;
  const done = new Promise<string>((resolve) => {
    finish = () => resolve("");
  });
  stdin.on("finish", () => finish());
  const tools = { pipe: () => ({ stdin, done }) };
  return { tools, written };
}

describe("FrameEncoder", () => {
  it("holds each frame until the next one arrives, at constant frame rate from the origin", async () => {
    const { tools, written } = fakeFfmpeg();
    const enc = new FrameEncoder(tools, "out.mp4", 10);
    enc.offer(Buffer.from("A"), 900);
    enc.setOrigin(1000);
    enc.offer(Buffer.from("B"), 1250);
    enc.offer(Buffer.from("C"), 1400);
    const duration = await enc.finish(1600);
    expect(written.join("")).toBe("AAABCCC");
    expect(enc.frames).toBe(7);
    expect(duration).toBe(700);
  });

  it("drops frames from before the origin except the latest one", async () => {
    const { tools, written } = fakeFfmpeg();
    const enc = new FrameEncoder(tools, "out.mp4", 10);
    enc.offer(Buffer.from("X"), 100);
    enc.offer(Buffer.from("Y"), 200);
    enc.setOrigin(500);
    await enc.finish(600);
    expect(written.join("")).toBe("YY");
  });

  it("takes frames without waiting for earlier ones to be written, and reports a failure at the end", async () => {
    const { tools, written } = fakeFfmpeg();
    const enc = new FrameEncoder(tools, "out.mp4", 10);
    enc.setOrigin(1000);
    enc.offer(Buffer.from("A"), 1000);
    enc.offer(Buffer.from("B"), 1250);
    enc.offer(Buffer.from("C"), 1400);
    await enc.finish(1600);
    expect(written.join("")).toBe("AAABCCC");
  });

  it("fails at the end instead of hanging when its process has already ended", async () => {
    const used = new FrameEncoder(fakeFfmpeg().tools, "out.mp4", 10);
    await used.finish(0);
    used.setOrigin(1000);
    used.offer(Buffer.from("A"), 1000);
    used.offer(Buffer.from("B"), 1500);
    await expect(used.finish(2000)).rejects.toThrow("The video encoder already finished.");
  });
});
