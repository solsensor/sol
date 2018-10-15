DROP TABLE tokens;
CREATE TABLE tokens (
  token TEXT PRIMARY KEY NOT NULL,
  user_id INTEGER NOT NULL,
  FOREIGN KEY(user_id) REFERENCES users(id)
);
