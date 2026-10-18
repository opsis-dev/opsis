import type { ReactNode } from "react";

export function FontRow({ children }: { children: ReactNode }) {
  return <div className="flex items-center gap-3 rounded-md px-2 py-1.5 text-sm">{children}</div>;
}
