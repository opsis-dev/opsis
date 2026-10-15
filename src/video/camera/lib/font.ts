import { ensureFontNow } from "../../fonts/registry";

export const INTER_FAMILY = "Inter";

export function loadFonts(): void {
  ensureFontNow(INTER_FAMILY);
}
