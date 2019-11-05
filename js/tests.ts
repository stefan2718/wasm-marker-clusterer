import { WasmMarkerClusterer } from "./index";

function createDiv(className: string, text: string | number) {
  let div = document.createElement("div");
  div.innerText = String(text);
  div.className = className;
  document.body.appendChild(div);
}

new WasmMarkerClusterer().init({}).then(clusterer => {
  createDiv("loaded", "Libary loaded");

  clusterer.addMarkers([{lat: 1, lng: 2}]);
  let clusters = clusterer.clusterMarkersInBounds({north: 3, south: 0, east:3, west: 0}, 8);

  createDiv("clusters-length", clusters.length);
  createDiv("cluster-size", clusters[0].size);

  clusterer.clear();
  clusters = clusterer.clusterMarkersInBounds({north: 3, south: 0, east:3, west: 0}, 8);

  createDiv("clusters-length-2", clusters.length);
})