import { stat } from "node:fs/promises";
import path from "node:path";
import type { Artifact } from "../../shared/schema/values";
import type { FfmpegTools } from "../media/ffmpeg";

const POSTER_WIDTH = 480;

export async function decorateArtifacts(
  projectDir: string,
  nodeDir: string,
  ffmpeg: FfmpegTools,
  artifacts: Artifact[],
): Promise<Artifact[]> {
  return Promise.all(
    artifacts.map(async (artifact, i) => {
      const file = path.join(projectDir, artifact.path);
      const size = await stat(file).then(
        (s) => s.size,
        () => undefined,
      );
      const withSize = size === undefined ? artifact : { ...artifact, bytes: size };
      if (artifact.kind !== "video" || artifact.poster) return withSize;
      const poster = path.join(nodeDir, `poster-${i}.jpg`);
      try {
        const probe = await ffmpeg.probe(file);
        const at = Math.max(0, probe.durationMs / 2000);
        await ffmpeg.run([
          "-ss",
          at.toFixed(2),
          "-i",
          file,
          "-frames:v",
          "1",
          "-vf",
          `scale=${POSTER_WIDTH}:-2`,
          "-q:v",
          "4",
          poster,
        ]);
        return { ...withSize, poster: path.relative(projectDir, poster).split(path.sep).join("/") };
      } catch {
        return withSize;
      }
    }),
  );
}
