DROP TABLE tokens;
CREATE TABLE tokens (
  token TEXT PRIMARY KEY NOT NULL,
  type TEXT NOT NULL,
  user_id INTEGER,
  sensor_id INTEGER,
  FOREIGN KEY(user_id) REFERENCES users(id)
);
