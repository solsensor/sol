ALTER TABLE sensors RENAME TO sensors_tmp;

CREATE TABLE sensors (
  id INTEGER PRIMARY KEY NOT NULL,
  owner_id INTEGER NOT NULL,
  hardware_id INTEGER NOT NULL,
  active BOOLEAN NOT NULL DEFAULT 1,
  name TEXT,
  description TEXT,
  FOREIGN KEY(owner_id) REFERENCES users(id)
);

INSERT INTO sensors (
  id, owner_id, hardware_id, active
) SELECT
  id, owner_id, hardware_id, active
FROM
  sensors_tmp;

DROP TABLE sensors_tmp;
