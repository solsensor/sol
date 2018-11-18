DROP TABLE readings;

CREATE TABLE readings (
  id INTEGER PRIMARY KEY NOT NULL,
  sensor_id INTEGER NOT NULL,
  timestamp INTEGER NOT NULL,
  peak_power_mW FLOAT NOT NULL,
  peak_current_mA FLOAT NOT NULL,
  peak_voltage_V FLOAT NOT NULL,
  temp_celsius FLOAT NOT NULL,
  batt_V FLOAT NOT NULL,
  FOREIGN KEY(sensor_id) REFERENCES tokens(id)
);
