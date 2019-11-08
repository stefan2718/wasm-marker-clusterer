import * as clusterer from "../pkg/webassembly_marker_clusterer.js";
import { IConfig, IMarker, IBounds, ICluster } from "./interfaces";
export * from "./interfaces";

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
  private config?: IConfig = { onlyReturnModifiedClusters: true };
  private previousZoom = -1;
  private previousClusters: ICluster[] = [];

  /**
   * @param config {IConfig} Uses default config if none passed.
   */
  constructor(config?: IConfig) {
    if (config) {
      this.configure(config);
    }
  }
  
  /**
   * Merges any passed config parameters into existing config. 
   * 
   * Clears cached clusters if `averageCenter` or `gridSize` is modified.
   */
  configure = (config: IConfig) => {
    if (this.config.averageCenter != config.averageCenter || this.config.gridSize !== config.gridSize) {
      this.clearClusters();
    }
    this.config = Object.assign(this.config, config);
    clusterer.configure(mapConfigNames(this.config));
  }

  /**
   * Calculates clusters for the markers within the given bounds.
   * 
   * @returns Newly calculated clusters merged with any previously calculated clusters
   */
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

  /**
   * Add an array of lat/lng markers so that they can be clustered.
   */
  addMarkers = (markers: IMarker[]) => clusterer.addMarkers(markers);
  /**
   * Clears all added markers and calculated clusters.
   */
  clear = () => {
    this.previousClusters = [];
    clusterer.clear();
  }
  /**
   * Clears only calculated clusters.
   */
  clearClusters = () => {
    this.previousClusters = [];
    clusterer.clearClusters();
  }
}