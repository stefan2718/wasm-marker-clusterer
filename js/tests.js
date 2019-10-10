// TODO Figure out how to run this
const lib = require("./index.js");

const clusterer = new lib.WasmMarkerClusterer({});

clusterer.addMarkers([{lat: 1, lng: 2}]);
let clusters = clusterer.clusterMarkersInBounds({north: 3, south: 0, east:3, west: 0}, 8);

console.assert(clusters.length === 1);
console.assert(clusters[0].size === 1);

clusterer.clear();
clusters = clusterer.clusterMarkersInBounds({north: 3, south: 0, east:3, west: 0}, 8);

console.assert(clusters.length === 0);