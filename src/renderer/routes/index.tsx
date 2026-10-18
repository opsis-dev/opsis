import { createFileRoute } from "@tanstack/react-router";
import { z } from "zod";
import { PROJECT_ACTIONS } from "../app/menu-commands";
import { Projects } from "../app/projects";

export const Route = createFileRoute("/")({
  validateSearch: z.object({ action: z.enum(PROJECT_ACTIONS).optional().catch(undefined) }),
  component: Projects,
});
