DROP TABLE sensors;
CREATE TABLE sensors (
  id INTEGER PRIMARY KEY NOT NULL,
  owner_id INTEGER NOT NULL,
  hardware_id INTEGER NOT NULL,
  FOREIGN KEY(owner_id) REFERENCES users(id)
);
