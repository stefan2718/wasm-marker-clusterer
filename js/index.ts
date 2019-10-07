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
export default class WasmMarkerClusterer {
  private config?: IConfig;
  private clusterer: typeof import("../pkg/index");
  private previousZoom = -1;
  private previousClusters: ICluster[] = [];

  constructor(config?: IConfig) {
    this.config = config;
  }

  init = async () => {
    const clusterer = await import("../pkg/index.js");
    this.clusterer = clusterer;
    this.clusterer.configure(mapConfigNames(this.config));
  }

  addMarkers = (markers: IMarker[]) => {
    this.clusterer.addMarkers(markers);
  }

  clusterMarkerInBounds = (bounds: IBounds, zoom: number): ICluster[] => {
    let zoomChanged = zoom !== this.previousZoom;
    if (this.config.logTime) console.time("into-wasm");
    let wasmClusters = this.clusterer.clusterMarkersInBounds(bounds, zoom);
    if (this.config.logTime) console.timeEnd("out-of-wasm");
    this.previousClusters = !this.config.onlyReturnModifiedClusters || zoomChanged 
        ? wasmClusters 
        : this.mergeModifiedClusters(this.previousClusters, wasmClusters);
    return this.previousClusters;
  }

  clear = () => this.clusterer.clear();

  private mergeModifiedClusters = (prevClusters: ICluster[], modifiedClusters: ICluster[]): ICluster[] => {
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
}
