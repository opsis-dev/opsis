import { createFileRoute } from "@tanstack/react-router";
import { z } from "zod";
import { useMenuShortcuts } from "@/app/menu-shortcuts";
import { SETTINGS_PAGES } from "@/app/shell";
import { SettingsWindow } from "../app/settings/settings-window";

function SettingsRoute() {
  useMenuShortcuts();
  return <SettingsWindow />;
}

export const Route = createFileRoute("/settings")({
  validateSearch: z.object({ page: z.enum(SETTINGS_PAGES).catch("general") }),
  component: SettingsRoute,
});
