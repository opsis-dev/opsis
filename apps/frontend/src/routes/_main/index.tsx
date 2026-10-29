import { createFileRoute } from "@tanstack/react-router";
import { Projects } from "@/app/projects";

export const Route = createFileRoute("/_main/")({
  component: Projects,
});
