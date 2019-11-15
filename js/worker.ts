import { expose } from "comlink";

import(/* webpackChunkName: "wasm" */ "../pkg/webassembly_marker_clusterer.js").then(clusterer => {
  console.log("worker")
  self.postMessage({ ready: true }, undefined);
  expose(clusterer);
})