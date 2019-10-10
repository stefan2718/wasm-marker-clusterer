import * as clusterer from "../pkg/webassembly_marker_clusterer.js";

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

export class WasmMarkerClusterer {
  constructor(config) {
    this.config = config;
    this.previousZoom = -1;
    this.previousClusters = [];
    clusterer.configure(mapConfigNames(this.config));
  }

  addMarkers(markers) {
    clusterer.addMarkers(markers);
  }

  clusterMarkersInBounds(bounds, zoom) {
    let zoomChanged = zoom !== this.previousZoom;
    this.previousZoom = zoom;

    if (this.config.logTime)console.time("into-wasm");
    let wasmClusters = clusterer.clusterMarkersInBounds(bounds, zoom);
    if (this.config.logTime) console.timeEnd("out-of-wasm");

    this.previousClusters = !this.config.onlyReturnModifiedClusters || zoomChanged
      ? wasmClusters
      : mergeModifiedClusters(this.previousClusters, wasmClusters);
    return this.previousClusters;
  }

  clear() {
    clusterer.clear();
  }
}

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