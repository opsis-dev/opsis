import { useRouter } from "@tanstack/react-router";
import { useEffect } from "react";
import { commands } from "@/generated/bindings";
import { onMenuCommand, projectActionOf, usePendingProjectAction } from "./menu-commands";

export function useMenuNavigation(): void {
  const router = useRouter();
  useEffect(
    () =>
      onMenuCommand((command) => {
        const path = router.state.location.pathname;
        const action = projectActionOf(command);
        if (command === "view:settings") void commands.openSettings(null);
        else if (command === "view:projects") void router.navigate({ to: "/" });
        else if (action) {
          usePendingProjectAction.setState({ pending: action });
          if (path !== "/") void router.navigate({ to: "/" });
        }
      }),
    [router],
  );
}
