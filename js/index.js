import * as clusterer from "../pkg/index";

const camelToSnake = (str) => str.replace(
  /([A-Z]?[a-z]+)/g,
  (group) => `${group.toLowerCase()}_`
).slice(0, -1);

const configProps = ["gridSize", "averageCenter", "logTime", "onlyReturnModifiedClusters"];

const mapConfigNames = (config) => {
  let wasmConfig = {};
  configProps.forEach((property) => {
    if (config[property] !== null || config[property] !== undefined) {
      wasmConfig[camelToSnake(property)] = config[property];
    }
  });
  return wasmConfig;
}
function WasmMarkerClusterer(config) {
  this.config = config;
  this.previousZoom = -1;
  this.previousClusters = [];
  clusterer.configure(mapConfigNames(this.config));
}

WasmMarkerClusterer.prototype.addMarkers = function(markers) {
  clusterer.addMarkers(markers);
}

WasmMarkerClusterer.prototype.clusterMarkersInBounds = function (bounds, zoom) {
  let zoomChanged = zoom !== this.previousZoom;
  if (this.config.logTime) console.time("into-wasm");
  let wasmClusters = clusterer.clusterMarkersInBounds(bounds, zoom);
  if (this.config.logTime) console.timeEnd("out-of-wasm");
  this.previousClusters = !this.config.onlyReturnModifiedClusters || zoomChanged 
      ? wasmClusters 
      : mergeModifiedClusters(this.previousClusters, wasmClusters);
  return this.previousClusters;
}

WasmMarkerClusterer.prototype.clear = function () { 
  clusterer.clear();
};

const mergeModifiedClusters = (prevClusters, modifiedClusters) => {
  modifiedClusters.forEach(modifiedCluster => {
    let index = prevClusters.findIndex(prevCluster => prevCluster.uuid === modifiedCluster.uuid);
    if (index === -1) {
      prevClusters.push(modifiedCluster);
    } else {
      prevClusters[index] = modifiedCluster;
    }
  });
  return prevClusters;
}

// if (typeof module == 'object') {
  module.exports = WasmMarkerClusterer;
// }
// if(typeof exports === 'object' && typeof module === 'object')
// 		module.exports = factory();
// 	else if(typeof define === 'function' && define.amd)
// 		define([], factory);
// 	else if(typeof exports === 'object')
// 		exports["WasmMarkerClusterer"] = factory();
// 	else
// 		root["WasmMarkerClusterer"] = factory();