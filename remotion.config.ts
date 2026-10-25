import path from "node:path";
import { makeBundlerOverride } from "@opsis/sidecar/bundler-override";
import { VIDEO_SOURCE } from "@opsis/sidecar/source-layout";
import { Config } from "@remotion/cli/config";

Config.setVideoImageFormat("jpeg");
Config.setOverwriteOutput(true);
Config.setChromiumOpenGlRenderer("angle");
Config.setRspack(true);
Config.overrideRspackConfig(makeBundlerOverride(path.join(process.cwd(), ...VIDEO_SOURCE)));
