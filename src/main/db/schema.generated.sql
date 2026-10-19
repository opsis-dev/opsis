CREATE TABLE `fonts` (
	`id` integer PRIMARY KEY,
	`fontId` text NOT NULL UNIQUE,
	`def` text NOT NULL
);

CREATE TABLE `meta` (
	`id` integer PRIMARY KEY,
	`schemaHash` text NOT NULL
);

CREATE TABLE `projects` (
	`id` integer PRIMARY KEY,
	`dir` text NOT NULL UNIQUE,
	`name` text NOT NULL,
	`openedAt` integer NOT NULL
);

CREATE TABLE `settings` (
	`id` integer PRIMARY KEY,
	`section` text NOT NULL UNIQUE,
	`value` text NOT NULL
);

CREATE TABLE `view_state` (
	`id` integer PRIMARY KEY,
	`projectId` integer NOT NULL,
	`key` text NOT NULL,
	`value` text NOT NULL,
	CONSTRAINT `fk_view_state_projectId_projects_id_fk` FOREIGN KEY (`projectId`) REFERENCES `projects`(`id`) ON DELETE CASCADE
);

CREATE TABLE `window_bounds` (
	`id` integer PRIMARY KEY,
	`window` text NOT NULL UNIQUE,
	`x` integer NOT NULL,
	`y` integer NOT NULL,
	`width` integer NOT NULL,
	`height` integer NOT NULL
);

CREATE UNIQUE INDEX `view_state_project_key` ON `view_state` (`projectId`,`key`);
