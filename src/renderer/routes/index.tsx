import { createFileRoute } from "@tanstack/react-router";
import { z } from "zod";
import { Projects } from "../app/projects";

export const Route = createFileRoute("/")({
  validateSearch: z.object({ action: z.enum(["new", "open"]).optional().catch(undefined) }),
  component: Projects,
});
