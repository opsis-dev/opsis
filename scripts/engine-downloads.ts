import {
  ENGINE_DOWNLOADS,
  readEngineDownloads,
  SHELL_PLATFORMS,
  shellUrl,
} from "./lib/engine-downloads";
import { pinned, writeGeneratedJson } from "./lib/pins";
import { devCache } from "./lib/tool-config";

const CACHE = devCache("engine-downloads");

const version = process.argv[2] ?? readEngineDownloads().testedVersion;
const shells = [];
for (const platform of SHELL_PLATFORMS)
  shells.push({ platform, ...(await pinned(shellUrl(version, platform), CACHE)) });
writeGeneratedJson(ENGINE_DOWNLOADS, { testedVersion: version, shells });
