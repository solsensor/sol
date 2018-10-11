ALTER TABLE users RENAME TO users_tmp;

CREATE TABLE users (
  id INTEGER PRIMARY KEY NOT NULL,
  email TEXT NOT NULL
);

INSERT INTO users
SELECT
 id, email
FROM
  users_tmp;

DROP TABLE users_tmp;
