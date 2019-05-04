var spec = {
	"$schema": "https://vega.github.io/schema/vega-lite/v3.json",
	"description": "Sensor Data",
	"data": {
		"url": "/api/sensor/10/readings?start=1548210420&end=1548220420",
		"format": { "property": "data.readings" }
	},
	"transform": [
		{ "calculate": "1000*datum.timestamp", as: "timestampms" }
	],
	"vconcat": [
		{
			"width": 1000,
			"mark": {
				"type": "line",
				"point": true
			},
			"encoding": {
				"x": {
					"field": "timestampms",
					"type": "temporal",
					"timeUnit": "yearmonthdatehoursminutesseconds",
					"axis": { "title": "" }
				},
				"y": {
					"field": "peak_current_mA",
					"type": "quantitative",
					"axis": { "title": "peak current (mA)" },
				}
			}
		},
		{
			"width": 1000,
			"mark": {
				"type": "line",
				"point": true
			},
			"encoding": {
				"x": {
					"field": "timestampms",
					"type": "temporal",
					"timeUnit": "yearmonthdatehoursminutesseconds",
					"axis": { "title": "" }
				},
				"y": {
					"field": "peak_power_mW",
					"type": "quantitative",
					"axis": { "title": "peak power (mW)" },
				}
			}
		},
		{
			"width": 1000,
			"mark": {
				"type": "line",
				"point": true
			},
			"encoding": {
				"x": {
					"field": "timestampms",
					"type": "temporal",
					"timeUnit": "yearmonthdatehoursminutesseconds",
					"axis": { "title": "" }
				},
				"y": {
					"field": "peak_voltage_V",
					"type": "quantitative",
					"axis": { "title": "peak voltage (V)" },
				}
			}
		},
		{
			"width": 1000,
			"mark": {
				"type": "line",
				"point": true
			},
			"encoding": {
				"x": {
					"field": "timestampms",
					"type": "temporal",
					"timeUnit": "yearmonthdatehoursminutesseconds",
					"axis": { "title": "" }
				},
				"y": {
					"field": "temp_celsius",
					"type": "quantitative",
					"axis": { "title": "temp (C)" },
				}
			}
		}
	]
}

var embedOpts = {
	actions: false,
};

vegaEmbed('#charts', spec, embedOpts);
