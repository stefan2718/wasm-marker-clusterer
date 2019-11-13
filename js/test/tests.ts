function createDiv(className: string, text: string | number) {
  let div = document.createElement("div");
  div.innerText = String(text);
  div.className = className;
  document.body.appendChild(div);
}

import(/* webpackChunkName: "index" */ "../index").then(async lib => {
  let clusterer = new lib.WasmMarkerClusterer()
  createDiv("loaded", "Libary loaded");

  console.log("a")
  await clusterer.addMarkers([{lat: 1, lng: 2}]);
  console.log("b")
  let clusters = await clusterer.clusterMarkersInBounds({north: 3, south: 0, east:3, west: 0}, 8);
  console.log("c")

  createDiv("clusters-length", clusters.length);
  createDiv("cluster-size", clusters[0].size);

  await clusterer.clear();
  console.log("d")
  clusters = await clusterer.clusterMarkersInBounds({north: 3, south: 0, east:3, west: 0}, 8);
  console.log("e")

  createDiv("clusters-length-2", clusters.length);
})