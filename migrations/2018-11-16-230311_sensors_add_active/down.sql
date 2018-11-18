ALTER TABLE sensors RENAME TO sensors_tmp;

CREATE TABLE sensors (
  id INTEGER PRIMARY KEY NOT NULL,
  owner_id INTEGER NOT NULL,
  hardware_id INTEGER NOT NULL,
  FOREIGN KEY(owner_id) REFERENCES users(id)
);

INSERT INTO sensors
SELECT
  id, owner_id, hardware_id
FROM
  sensors_tmp;

DROP TABLE sensors_tmp;
