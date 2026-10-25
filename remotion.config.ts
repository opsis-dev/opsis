import path from "node:path";
import { VIDEO_SOURCE } from "@opsis/sidecar/source-layout";
import { makeWebpackOverride } from "@opsis/sidecar/webpack-override";
import { Config } from "@remotion/cli/config";

Config.setVideoImageFormat("jpeg");
Config.setOverwriteOutput(true);
Config.setChromiumOpenGlRenderer("angle");
Config.overrideWebpackConfig(makeWebpackOverride(path.join(process.cwd(), ...VIDEO_SOURCE)));
