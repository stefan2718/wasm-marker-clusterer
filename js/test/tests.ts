function createDiv(className: string, text: string | number) {
  let div = document.createElement("div");
  div.innerText = String(text);
  div.className = className;
  document.body.appendChild(div);
}

import(/* webpackChunkName: "index" */ "../index").then(async lib => {
  let clusterer = new lib.WasmMarkerClusterer()
  createDiv("loaded", "Libary loaded");
  await clusterer.configure({});

  await clusterer.addMarkers([{lat: 1, lng: 2}]);
  let clusters = await clusterer.clusterMarkersInBounds({north: 3, south: 0, east:3, west: 0}, 8);

  createDiv("clusters-length", clusters.length);
  createDiv("cluster-size", clusters[0].size);

  await clusterer.clear();
  clusters = await clusterer.clusterMarkersInBounds({north: 3, south: 0, east:3, west: 0}, 8);

  createDiv("clusters-length-2", clusters.length);
})