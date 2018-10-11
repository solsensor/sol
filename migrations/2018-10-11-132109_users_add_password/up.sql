ALTER TABLE users RENAME TO users_tmp;

CREATE TABLE users (
  id INTEGER PRIMARY KEY NOT NULL,
  email TEXT NOT NULL,
  password TEXT NOT NULL
);

INSERT INTO users
SELECT
 id, email, "Password123"
FROM
  users_tmp;

DROP TABLE users_tmp;
