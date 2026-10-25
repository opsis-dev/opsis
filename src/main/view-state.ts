import type { ViewPrefs } from "@opsis/core/ipc";
import { eq } from "drizzle-orm";
import { db } from "./db";
import { projects, viewState } from "./db/schema";

const projectId = (dir: string): number | undefined =>
  db().select({ id: projects.id }).from(projects).where(eq(projects.dir, dir)).get()?.id;

export function readViewState(dir: string): ViewPrefs {
  const id = projectId(dir);
  if (id === undefined) return {};
  const rows = db().select().from(viewState).where(eq(viewState.projectId, id)).all();
  return Object.fromEntries(rows.map((row) => [row.key, row.value]));
}

export function writeViewState(dir: string, key: string, value: unknown): boolean {
  const id = projectId(dir);
  if (id === undefined) return false;
  db()
    .insert(viewState)
    .values({ projectId: id, key, value })
    .onConflictDoUpdate({ target: [viewState.projectId, viewState.key], set: { value } })
    .run();
  return true;
}

export function forgetViewState(dir: string): void {
  const id = projectId(dir);
  if (id !== undefined) db().delete(viewState).where(eq(viewState.projectId, id)).run();
}
