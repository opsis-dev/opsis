import { createFileRoute } from "@tanstack/react-router";
import { Setup } from "../app/Setup";

export const Route = createFileRoute("/setup")({ component: Setup });
