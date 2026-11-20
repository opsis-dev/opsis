CREATE TABLE model_service (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  service TEXT NOT NULL,
  model TEXT NOT NULL,
  address TEXT
);

CREATE TABLE sessions (
  id TEXT PRIMARY KEY,
  folder TEXT NOT NULL,
  mode TEXT NOT NULL,
  title TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE TABLE messages (
  session_id TEXT NOT NULL REFERENCES sessions (id) ON DELETE CASCADE,
  seq INTEGER NOT NULL,
  role TEXT NOT NULL,
  content TEXT NOT NULL,
  PRIMARY KEY (session_id, seq)
);

CREATE TABLE allowed (
  session_id TEXT NOT NULL REFERENCES sessions (id) ON DELETE CASCADE,
  kind TEXT NOT NULL,
  PRIMARY KEY (session_id, kind)
);
