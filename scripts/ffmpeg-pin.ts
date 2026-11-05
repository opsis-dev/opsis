import {
  FFMPEG_CACHE,
  FFMPEG_PINS,
  FFMPEG_RELEASE,
  FFMPEG_TARGETS,
  ffmpegLicenceUrl,
  ffmpegProgramUrl,
  ffprobeProgramUrl,
} from "./lib/ffmpeg";
import { pinned, writeGeneratedJson } from "./lib/pins";

const targets = [];
for (const [target, asset] of FFMPEG_TARGETS)
  targets.push({
    target,
    asset,
    program: await pinned(ffmpegProgramUrl(asset), FFMPEG_CACHE),
    probe: await pinned(ffprobeProgramUrl(asset), FFMPEG_CACHE),
    licence: await pinned(ffmpegLicenceUrl(asset), FFMPEG_CACHE),
  });
writeGeneratedJson(FFMPEG_PINS, { release: FFMPEG_RELEASE, targets });
