import { Config } from "@remotion/cli/config";
import { makeWebpackOverride } from "./src/worker/render/webpack-override";

Config.setVideoImageFormat("jpeg");
Config.setOverwriteOutput(true);
Config.setChromiumOpenGlRenderer("angle");
Config.overrideWebpackConfig(makeWebpackOverride(process.cwd()));
