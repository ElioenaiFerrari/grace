CREATE TABLE IF NOT EXISTS accounts (
  id TEXT PRIMARY KEY,
  chat_id BIGINT NOT NULL,
  first_name VARCHAR NOT NULL,
  last_name VARCHAR NOT NULL,
  verified BOOLEAN NOT NULL DEFAULT FALSE,
  did_onboarding BOOLEAN NOT NULL DEFAULT FALSE
);