const map = new maplibregl.Map({
  container: 'map',
  style: 'https://api.os.uk/maps/vector/v1/vts/resources/styles?srs=3857&key=' + osKey,
  center: [-1.5, 51.5],
  zoom: 8.7,
  hash: true,
  maxZoom: 15,
  minZoom: 7,
});
map.on('load', () => {
  map.addSource('lines', { 'type': 'geojson', data });
  map.addLayer({
    'id': 'lines',
    'type': 'line',
    'source': 'lines',
    'layout': { 'line-join': 'round', 'line-cap': 'round' },
    'paint': { 
      'line-width': 8,
      'line-color': [
        'case',
        ['==', ['get', 'type'], 'Ride'], '#984ea3', // lilac for Ride
        ['==', ['get', 'type'], 'Run'], '#ff7f00', // green for Run
        '#888', // Default color for any other type
      ],
      'line-opacity': [
        'interpolate', 
        ['linear'], 
        ['zoom'],
        7, 1,
        15, 0.4,
      ],
      'line-width': [
        'interpolate', 
        ['linear'], 
        ['zoom'],
        7, 2,
        15, 10,
      ],
    },
  });
});
