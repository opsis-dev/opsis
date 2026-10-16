import { createFileRoute } from "@tanstack/react-router";
import { SettingsWindow } from "../app/settings/settings-window";

function SettingsRoute() {
  const { page } = Route.useSearch();
  return <SettingsWindow page={page} />;
}

export const Route = createFileRoute("/settings")({
  validateSearch: (search: Record<string, unknown>) => ({ page: String(search.page ?? "general") }),
  component: SettingsRoute,
});
