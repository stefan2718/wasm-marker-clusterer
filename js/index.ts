import * as clusterer from "../pkg/webassembly_marker_clusterer.js";
import { IConfig, IMarker, IBounds, ICluster } from "./interfaces";

const camelToSnake = (str: string): string => str.replace(
  /([A-Z]?[a-z]+)/g,
  (group) => `${group.toLowerCase()}_`
).slice(0, -1);

const configProps: (keyof IConfig)[] = ["gridSize", "averageCenter", "logTime", "onlyReturnModifiedClusters"];

const mapConfigNames = (config: IConfig) => {
  let wasmConfig: any = {};
  configProps.forEach((property) => {
    if (config[property] !== null || config[property] !== undefined) {
      wasmConfig[camelToSnake(property)] = config[property];
    }
  });
  return wasmConfig;
}

const mergeModifiedClusters = (prevClusters: ICluster[], modifiedClusters: ICluster[]): ICluster[] => {
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

export class WasmMarkerClusterer {
  private config?: IConfig;
  private previousZoom = -1;
  private previousClusters: ICluster[] = [];

  constructor(config?: IConfig) {
    this.config = Object.assign({
      onlyReturnModifiedClusters: true
    }, config);
    clusterer.configure(mapConfigNames(this.config));
  }

  addMarkers = (markers: IMarker[]) => {
    clusterer.addMarkers(markers);
  }

  clusterMarkersInBounds = (bounds: IBounds, zoom: number): ICluster[] => {
    let zoomChanged = zoom !== this.previousZoom;
    this.previousZoom = zoom;

    if (this.config.logTime) console.time("into-wasm");
    let wasmClusters = clusterer.clusterMarkersInBounds(bounds, zoom);
    if (this.config.logTime) console.timeEnd("out-of-wasm");

    this.previousClusters = !this.config.onlyReturnModifiedClusters || zoomChanged 
        ? wasmClusters 
        : mergeModifiedClusters(this.previousClusters, wasmClusters);
    return this.previousClusters;
  }

  clear = () => clusterer.clear();
}
export * from "./interfaces";