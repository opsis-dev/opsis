import { cpSync, rmSync } from "node:fs";
import path from "node:path";
import { devCache, ROOT } from "../../../scripts/lib/tool-config";
import { seedUserData } from "../../../scripts/lib/user-data";

const CHECK_DIR = devCache("check");
const USER_DATA = path.join(CHECK_DIR, "user-data");
const PROJECTS = path.join(CHECK_DIR, "projects");

rmSync(CHECK_DIR, { recursive: true, force: true });
seedUserData(USER_DATA);
cpSync(path.join(ROOT, "examples"), PROJECTS, { recursive: true });
console.log(USER_DATA);
