import { formatConfig } from "./scripts/lib/oxfmt-base.ts";
import { IGNORED_PATHS } from "./scripts/lib/tool-config.ts";

export default formatConfig([...IGNORED_PATHS, "bun.lock"]);
