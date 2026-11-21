CREATE TABLE settings (
  id INTEGER PRIMARY KEY,
  section TEXT NOT NULL UNIQUE,
  value TEXT NOT NULL
);

CREATE TABLE projects (
  id INTEGER PRIMARY KEY,
  dir TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  opened_at INTEGER NOT NULL
);

CREATE TABLE view_state (
  id INTEGER PRIMARY KEY,
  project_id INTEGER NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
  key TEXT NOT NULL,
  value TEXT NOT NULL,
  UNIQUE (project_id, key)
);

CREATE TABLE fonts (
  id INTEGER PRIMARY KEY,
  font_id TEXT NOT NULL UNIQUE,
  def TEXT NOT NULL
);

CREATE TABLE assistant_events (
  seq INTEGER PRIMARY KEY,
  session_id TEXT NOT NULL,
  project TEXT NOT NULL,
  agent TEXT NOT NULL,
  kind TEXT NOT NULL,
  at INTEGER NOT NULL,
  payload TEXT NOT NULL
);

CREATE TABLE assistant_sessions (
  id TEXT PRIMARY KEY,
  project TEXT NOT NULL,
  agent TEXT NOT NULL,
  agent_session TEXT,
  started_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  open_message INTEGER
);

CREATE TABLE assistant_messages (
  session_id TEXT NOT NULL,
  seq INTEGER NOT NULL,
  role TEXT NOT NULL,
  text TEXT NOT NULL,
  at INTEGER NOT NULL,
  PRIMARY KEY (session_id, seq)
);

CREATE TABLE assistant_turns (
  session_id TEXT NOT NULL,
  seq INTEGER NOT NULL,
  started_at INTEGER NOT NULL,
  ended_at INTEGER,
  stop TEXT,
  PRIMARY KEY (session_id, seq)
);

CREATE TABLE assistant_requests (
  session_id TEXT NOT NULL,
  request TEXT NOT NULL,
  kind TEXT NOT NULL,
  title TEXT NOT NULL,
  answer TEXT,
  asked_at INTEGER NOT NULL,
  answered_at INTEGER,
  PRIMARY KEY (session_id, request)
);
