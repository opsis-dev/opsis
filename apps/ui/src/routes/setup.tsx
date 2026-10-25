import { createFileRoute } from "@tanstack/react-router";
import { Setup } from "../app/setup";

export const Route = createFileRoute("/setup")({ component: Setup });
