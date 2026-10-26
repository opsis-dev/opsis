import { useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";
import { openEditor } from "./open-project";

export function useApiHost(): void {
  const navigate = useNavigate();
  useEffect(() => window.opsis.onOpenRequested(({ dir }) => openEditor(navigate, dir)), [navigate]);
}
