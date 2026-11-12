import type { ComponentType } from "react";

const CASES = import.meta.glob<{ Case: ComponentType }>("./cases/*/*.tsx", { eager: true });

export function caseNamed(id: string): ComponentType {
  const found = CASES[`./cases/${id}.tsx`];
  if (!found) throw new Error(`There's no oracle case named “${id}”.`);
  return found.Case;
}
