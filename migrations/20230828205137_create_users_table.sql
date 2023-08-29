CREATE TABLE IF NOT EXISTS users (
  id uuid NOT NULL,
  PRIMARY KEY (id),
  login TEXT NOT NULL,
  role TEXT NOT NULL,
  password_hash TEXT NOT NULL,
  created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);
