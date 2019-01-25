var vlSpec = {
	$schema: 'https://vega.github.io/schema/vega-lite/v3.json',
	description: 'Battery voltage over time',
	data: {
		values: [
			{ timestamp: 1542513093, batt_V: 1.2300000190734863 },
			{ timestamp: 1542513094, batt_V: 1.3300000190734863 },
			{ timestamp: 1542513095, batt_V: 1.2300000190734863 },
			{ timestamp: 1542513098, batt_V: 1.4300000190734863 },
			{ timestamp: 1542513099, batt_V: 1.1300000190734863 },
		]
	},
	mark: {
		type: 'line',
		point: true,
	},
	encoding: {
		x: {
			field: 'timestamp',
			type: 'quantitative',
			axis: { title: 'timestamp (unix)' },
		},
		y: {
			field: 'batt_V',
			type: 'quantitative',
			axis: { title: 'battery (V)' },
		},
	}
};

vegaEmbed('#chart', vlSpec);
