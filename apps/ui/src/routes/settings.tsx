import { createFileRoute } from "@tanstack/react-router";
import { z } from "zod";
import { SETTINGS_PAGES } from "@/app/shell";
import { SettingsWindow } from "../app/settings/settings-window";

export const Route = createFileRoute("/settings")({
  validateSearch: z.object({ page: z.enum(SETTINGS_PAGES).catch("general") }),
  component: SettingsWindow,
});
