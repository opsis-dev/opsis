import { registerFaces } from "@opsis/video/common/font-faces";
import { useEffect, useState } from "react";
import { commands } from "@/generated/bindings";
import { fileUrl } from "../editor/file-url";

let registered: Promise<void> | null = null;

function registerTerminalFonts(): Promise<void> {
  registered ??= commands.terminalFonts().then(async (defs) => {
    await Promise.all(defs.map((def) => registerFaces(def.family, def, fileUrl)));
  });
  return registered;
}

export function useTerminalFonts(): boolean {
  const [ready, setReady] = useState(false);
  useEffect(() => {
    let alive = true;
    void registerTerminalFonts().finally(() => {
      if (alive) setReady(true);
    });
    return () => {
      alive = false;
    };
  }, []);
  return ready;
}
