import { createFileRoute } from "@tanstack/react-router";
import { Setup } from "@/app/setup";

export const Route = createFileRoute("/_main/setup")({ component: Setup });
