CREATE TYPE role AS ENUM ('user', 'assistant');

CREATE TABLE IF NOT EXISTS messages (
  id TEXT PRIMARY KEY,
  chat_id BIGINT NOT NULL,
  content TEXT NOT NULL,
  role role NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);