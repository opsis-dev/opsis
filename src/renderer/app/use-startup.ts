import { useRouter } from "@tanstack/react-router";
import { useEffect } from "react";
import { reportError } from "@/renderer/lib/toast";
import { loadedAppSettings } from "../store/app-settings";
import { useEnvironment } from "../store/environment";
import { dispatchMenuCommand } from "./menu-commands";
import { openEditor } from "./open-project";
import { markStarted } from "./startup-flags";

export function useStartup(): void {
  const router = useRouter();
  useEffect(() => {
    if (router.state.location.pathname !== "/") return;
    const first = markStarted();
    void Promise.all([useEnvironment.getState().refresh(), loadedAppSettings()])
      .then(async ([env, settings]) => {
        if (env && !env.engine.complete && !settings.app.setupSkipped) {
          void router.navigate({ to: "/setup" });
          return;
        }
        if (!first) return;
        if (settings.app.onStart === "last-project") {
          const [last] = await window.opsis.listRecentProjects();
          if (last) openEditor(router.navigate, last.dir);
        } else if (settings.app.onStart === "new-project") {
          dispatchMenuCommand("project:new");
        }
      })
      .catch((e: unknown) => reportError("Couldn't finish starting up", e));
  }, [router]);
}
