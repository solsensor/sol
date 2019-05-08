var startTime = moment().subtract(10, 'days').format('X');
var endTime = moment().format('X');

var a = document.createElement('a');
a.href = window.location.href;

var url = '/api' + a.pathname + '/readings?start=' + startTime + '&end=' + endTime;

var spec = {
	"$schema": "https://vega.github.io/schema/vega-lite/v3.json",
	"description": "Sensor Data",
	"data": {
		"url": url,
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
