import { expose } from "comlink";

import(/* webpackChunkName: "wasm" */ "../pkg/webassembly_marker_clusterer.js").then(clusterer => {
  self.postMessage({ ready: true }, undefined);
  expose(clusterer);
})