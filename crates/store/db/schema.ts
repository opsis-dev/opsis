import { integer, sqliteTable, text, uniqueIndex } from "drizzle-orm/sqlite-core";

export const meta = sqliteTable("meta", {
  id: integer().primaryKey(),
  schemaHash: text().notNull(),
});

export const settings = sqliteTable("settings", {
  id: integer().primaryKey(),
  section: text().notNull().unique(),
  value: text({ mode: "json" }).notNull(),
});

export const projects = sqliteTable("projects", {
  id: integer().primaryKey(),
  dir: text().notNull().unique(),
  name: text().notNull(),
  openedAt: integer().notNull(),
});

export const viewState = sqliteTable(
  "view_state",
  {
    id: integer().primaryKey(),
    projectId: integer()
      .notNull()
      .references(() => projects.id, { onDelete: "cascade" }),
    key: text().notNull(),
    value: text({ mode: "json" }).notNull(),
  },
  (t) => [uniqueIndex("view_state_project_key").on(t.projectId, t.key)],
);

export const windowBounds = sqliteTable("window_bounds", {
  id: integer().primaryKey(),
  window: text().notNull().unique(),
  x: integer().notNull(),
  y: integer().notNull(),
  width: integer().notNull(),
  height: integer().notNull(),
});

export const fonts = sqliteTable("fonts", {
  id: integer().primaryKey(),
  fontId: text().notNull().unique(),
  def: text().notNull(),
});
