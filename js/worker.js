import { expose } from "comlink";
// import * as clusterer from "../pkg/webassembly_marker_clusterer.js";

// const wrapper = {
//   main_js: () => clusterer.main_js(),
//   configure: (config) => clusterer.configure(config),
//   addMarkers: (markers_val) => clusterer.addMarkers(markers_val),
//   clusterMarkersInBounds: (bounds_val, zoom) => clusterer.clusterMarkersInBounds(bounds_val, zoom),
//   clear: () => clusterer.clear(),
//   clearClusters: () => clusterer.clearClusters()
// };

import(/* webpackChunkName: "wasm" */ "../pkg/webassembly_marker_clusterer.js").then(clusterer => {
  expose(clusterer);
})