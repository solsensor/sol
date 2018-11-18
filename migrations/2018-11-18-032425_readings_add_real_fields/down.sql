DROP TABLE readings;

CREATE TABLE readings (
  id INTEGER PRIMARY KEY NOT NULL,
  voltage FLOAT NOT NULL,
  sensor_id INTEGER NOT NULL,
  FOREIGN KEY(sensor_id) REFERENCES tokens(id)
);
