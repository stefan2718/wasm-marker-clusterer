test('two plus two is four', () => {
  expect(2 + 2).toBe(4);
});

test('library functions', () => {
  return import('./index').then(lib => {
    const clusterer = new lib.WasmMarkerClusterer({});
    expect(clusterer).toBeDefined();
  
    clusterer.addMarkers([{lat: 1, lng: 2}]);
    let clusters = clusterer.clusterMarkersInBounds({north: 3, south: 0, east:3, west: 0}, 8);
  
    expect(clusters.length).toBe(1);
    expect(clusters[0].size).toBe(1);
  
    clusterer.clear();
    clusters = clusterer.clusterMarkersInBounds({north: 3, south: 0, east:3, west: 0}, 8);
  
    expect(clusters.length).toBe(0);
  });
});