import { useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";
import { on } from "@/app/tauri";
import { events } from "@/generated/bindings";
import { openEditor } from "./open-project";

export function useApiHost(): void {
  const navigate = useNavigate();
  useEffect(() => on(events.openRequested, ({ dir }) => openEditor(navigate, dir)), [navigate]);
}
