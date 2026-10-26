import { platform } from "@/lib/platform";
import { forwardConsole } from "./page-log";
import { rememberDrops } from "./tauri";

forwardConsole();
if (platform !== "win32") rememberDrops();
