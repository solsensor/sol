var startTime = moment().subtract(10, 'days').format('X');
var endTime = moment().format('X');

var a = document.createElement('a');
a.href = window.location.href;

var url = '/api' + a.pathname + '/energy_stats';

var spec = {
	"$schema": "https://vega.github.io/schema/vega-lite/v3.json",
	"description": "Sensor Data",
	"data": {
		"url": url,
		"format": { "property": "stats" }
	},
	"vconcat": [
		{
			"width": 1000,
			"mark": {
				"type": "line",
				"point": true
			},
			"encoding": {
				"x": {
					"field": "date",
					"type": "temporal",
					"timeUnit": "yearmonthdate",
					"axis": { "title": "" }
				},
				"y": {
					"field": "dollars_saved",
					"type": "quantitative",
					"axis": { "title": "dollars saved" },
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
					"field": "date",
					"type": "temporal",
					"timeUnit": "yearmonthdate",
					"axis": { "title": "" }
				},
				"y": {
					"field": "co2_saved",
					"type": "quantitative",
					"axis": { "title": "lbs CO2 equivalent saved" },
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
					"field": "date",
					"type": "temporal",
					"timeUnit": "yearmonthdate",
					"axis": { "title": "" }
				},
				"y": {
					"field": "equiv_kWh",
					"type": "quantitative",
					"axis": { "title": "kWh" },
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
					"field": "date",
					"type": "temporal",
					"timeUnit": "yearmonthdate",
					"axis": { "title": "" }
				},
				"y": {
					"field": "cap_factor",
					"type": "quantitative",
					"axis": { "title": "capacity factor" },
				}
			}
		}
	]
}

var embedOpts = {
	actions: false,
};

vegaEmbed('#stats-charts', spec, embedOpts);
