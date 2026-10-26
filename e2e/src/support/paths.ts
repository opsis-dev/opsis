import os from "node:os";
import path from "node:path";

export const ROOT = path.resolve(import.meta.dirname, "..", "..", "..");

export const APP_BINARY = path.join(ROOT, "target", "e2e", "debug", "opsis-desktop");

export const RUN_DIR = path.join(os.homedir(), ".cache", "opsis-e2e");

export const USER_DATA = path.join(RUN_DIR, "user-data");

export const PROJECTS = path.join(RUN_DIR, "projects");

export const API_SOCKET = path.join(USER_DATA, "api.sock");

export const DEV_ENGINE = path.join(os.homedir(), ".config", "Opsis", "engine");

export const SHOTS = path.join(RUN_DIR, "failures");
