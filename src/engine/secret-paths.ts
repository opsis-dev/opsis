import path from "node:path";
import { secretFileName } from "@opsis/core/secrets";

export const SECRETS_DIR = "secrets";

export const SECRET_EXT = ".json.enc";

export const secretsDir = (projectDir: string): string => path.join(projectDir, SECRETS_DIR);

export const secretFile = (projectDir: string, name: string): string =>
  path.join(secretsDir(projectDir), `${secretFileName(name)}${SECRET_EXT}`);
