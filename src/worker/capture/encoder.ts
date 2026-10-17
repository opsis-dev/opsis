import { once } from "node:events";
import type { FfmpegTools } from "../media/ffmpeg";

const ENCODE_CRF = 16;

const encoderArgs = (fps: number, output: string): string[] => [
  "-f",
  "image2pipe",
  "-framerate",
  String(fps),
  "-c:v",
  "mjpeg",
  "-i",
  "-",
  "-an",
  "-vf",
  "pad=ceil(iw/2)*2:ceil(ih/2)*2,format=yuv420p",
  "-c:v",
  "libx264",
  "-preset",
  "medium",
  "-crf",
  String(ENCODE_CRF),
  "-r",
  String(fps),
  "-movflags",
  "+faststart",
  output,
];

export class FrameEncoder {
  private origin: number | null = null;
  private tick = 0;
  private last: Buffer | null = null;
  private queue: Promise<void> = Promise.resolve();
  private readonly stdin;
  private readonly done;
  private ended: Error | null = null;
  readonly frameMs: number;

  constructor(
    ffmpeg: Pick<FfmpegTools, "pipe">,
    readonly output: string,
    fps: number,
    signal?: AbortSignal,
  ) {
    this.frameMs = 1000 / fps;
    const proc = ffmpeg.pipe(encoderArgs(fps, output), { signal });
    this.stdin = proc.stdin;
    this.done = proc.done;
    this.done.then(
      () => {
        this.ended = new Error("The video encoder already finished.");
      },
      (error: unknown) => {
        this.ended = error instanceof Error ? error : new Error(String(error));
      },
    );
    this.stdin.on("error", (error) => {
      this.ended ??= error;
    });
  }

  get frames() {
    return this.tick;
  }

  get originMs() {
    return this.origin;
  }

  setOrigin(t: number) {
    this.origin = t;
  }

  push(jpeg: Buffer, t: number): Promise<void> {
    this.queue = this.queue
      .then(() => this.advance(t))
      .then(() => {
        this.last = jpeg;
      });
    return this.queue;
  }

  offer(jpeg: Buffer, t: number): void {
    this.push(jpeg, t).catch(() => undefined);
  }

  async finish(t: number): Promise<number> {
    await this.queue;
    await this.advance(t, true);
    this.stdin.end();
    await this.done;
    return this.tick * this.frameMs;
  }

  async abort(): Promise<void> {
    this.stdin.destroy();
    await this.done.catch(() => undefined);
  }

  private async advance(t: number, inclusive = false) {
    if (this.origin === null || !this.last) return;
    const due = inclusive ? (at: number) => at <= t : (at: number) => at < t;
    while (due(this.origin + this.tick * this.frameMs)) {
      if (this.ended || this.stdin.writableEnded)
        throw this.ended ?? new Error("The video encoder stopped.");
      if (!this.stdin.write(this.last)) {
        await Promise.race([once(this.stdin, "drain"), this.done]).catch(() => undefined);
        if (this.ended) throw this.ended;
      }
      this.tick++;
    }
  }
}
