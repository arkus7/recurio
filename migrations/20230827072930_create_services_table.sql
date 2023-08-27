-- Basic `services` structure
CREATE TABLE IF NOT EXISTS services(
  id uuid NOT NULL,
  PRIMARY KEY (id),
  name TEXT NOT NULL,
  created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);
