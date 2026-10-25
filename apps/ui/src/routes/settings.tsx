import { SETTINGS_PAGES } from "@opsis/core/ipc";
import { createFileRoute } from "@tanstack/react-router";
import { z } from "zod";
import { SettingsWindow } from "../app/settings/settings-window";

export const Route = createFileRoute("/settings")({
  validateSearch: z.object({ page: z.enum(SETTINGS_PAGES).catch("general") }),
  component: SettingsWindow,
});
