ALTER TABLE readings RENAME TO readings_tmp;

CREATE TABLE readings (
  id INTEGER PRIMARY KEY NOT NULL,
  sensor_id INTEGER NOT NULL,
  timestamp DATETIME NOT NULL,
  peak_power_mW FLOAT NOT NULL,
  peak_current_mA FLOAT NOT NULL,
  peak_voltage_V FLOAT NOT NULL,
  temp_celsius FLOAT NOT NULL,
  batt_V FLOAT NOT NULL,
  created DATETIME NOT NULL DEFAULT (datetime('now')),
  FOREIGN KEY(sensor_id) REFERENCES tokens(id)
);

INSERT INTO readings SELECT
  id,
  sensor_id,
  datetime(timestamp, 'unixepoch'),
  peak_power_mW,
  peak_current_mA,
  peak_voltage_V,
  temp_celsius,
  batt_V,
  created
FROM readings_tmp;

DROP TABLE readings_tmp;
