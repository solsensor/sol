ALTER TABLE users RENAME TO users_tmp;

CREATE TABLE users (
  id INTEGER PRIMARY KEY NOT NULL,
  email TEXT NOT NULL,
  pwd_hash TEXT NOT NULL,
  superuser BOOLEAN NOT NULL DEFAULT 0
);

INSERT INTO users SELECT id, email, pwd_hash, 0 FROM users_tmp;

DROP TABLE users_tmp;
