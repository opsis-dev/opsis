import { createFileRoute } from "@tanstack/react-router";
import { Projects } from "../app/Projects";

export const Route = createFileRoute("/")({ component: Projects });
