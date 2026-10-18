import { stat } from "node:fs/promises";
import path from "node:path";
import type { Artifact } from "../../shared/schema/values";
import { frameGrabArgs } from "../media/codecs";
import type { FfmpegTools } from "../media/ffmpeg";
import { toPosixRel } from "../../engine/project-paths";
import { errorMessage } from "../../shared/errors";

const POSTER_WIDTH = 480;
const POSTER_QUALITY = 4;

export async function decorateArtifacts(
  projectDir: string,
  nodeDir: string,
  ffmpeg: FfmpegTools,
  artifacts: Artifact[],
  warn: (message: string) => void,
): Promise<Artifact[]> {
  return Promise.all(
    artifacts.map(async (artifact, i) => {
      const file = path.join(projectDir, artifact.path);
      const size = await stat(file).then(
        (s) => s.size,
        (error: unknown) => {
          warn(`Couldn't read ${artifact.label ?? artifact.path}: ${errorMessage(error)}`);
          return undefined;
        },
      );
      const withSize = size === undefined ? artifact : { ...artifact, bytes: size };
      if (artifact.kind !== "video" || artifact.poster) return withSize;
      const poster = path.join(nodeDir, `poster-${i}.jpg`);
      try {
        const probe = await ffmpeg.probe(file);
        await ffmpeg.run(
          frameGrabArgs(file, poster, {
            atMs: Math.max(0, probe.durationMs / 2),
            width: POSTER_WIDTH,
            evenHeight: true,
            jpegQuality: POSTER_QUALITY,
          }),
        );
        return { ...withSize, poster: toPosixRel(projectDir, poster) };
      } catch (error) {
        warn(
          `Couldn't make a poster frame for ${artifact.label ?? artifact.path}: ${errorMessage(error)}`,
        );
        return withSize;
      }
    }),
  );
}
