import { wrap } from "comlink";
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
  private worker = new Worker("./worker.ts", { type: "module" } );
  private clusterer = wrap<typeof import("../pkg/webassembly_marker_clusterer.js")>(this.worker);
  private wasmResolve: (value?: unknown) => void;
  private wasmReady = new Promise((resolve) => {
    this.wasmResolve = resolve;
  })

  /**
   * @param config {IConfig} Uses default config if none passed.
   */
  constructor() {
    const onReady = (event: MessageEvent) => {
      if (event.data && event.data.ready === true) {
        this.wasmResolve();
        this.worker.removeEventListener("message", onReady);
      }
    }
    this.worker.addEventListener("message", onReady);
  }

  /**
   * Merges any passed config parameters into existing config. 
   * 
   * Clears cached clusters if `averageCenter` or `gridSize` is modified.
   */
  configure = async (config: IConfig): Promise<void> => {
    await this.wasmReady;
    if (this.config.averageCenter != config.averageCenter || this.config.gridSize !== config.gridSize) {
      this.clearClusters();
    }
    this.config = Object.assign(this.config, config);
    return this.clusterer.configure(mapConfigNames(this.config));
  }

  /**
   * Calculates clusters for the markers within the given bounds.
   * 
   * @returns Newly calculated clusters merged with any previously calculated clusters
   */
  clusterMarkersInBounds = async (bounds: IBounds, zoom: number): Promise<ICluster[]> => {
    await this.wasmReady;
    let zoomChanged = zoom !== this.previousZoom;
    this.previousZoom = zoom;

    if (this.config.logTime) console.time("into-wasm");
    let wasmClusters = await this.clusterer.clusterMarkersInBounds(bounds, zoom);
    if (this.config.logTime) console.timeEnd("out-of-wasm");

    this.previousClusters = !this.config.onlyReturnModifiedClusters || zoomChanged 
        ? wasmClusters 
        : mergeModifiedClusters(this.previousClusters, wasmClusters);
    return this.previousClusters;
  }

  /**
   * Add an array of lat/lng markers so that they can be clustered.
   */
  addMarkers = async (markers: IMarker[]): Promise<void> => {
    await this.wasmReady;
    return this.clusterer.addMarkers(markers);
  }
  /**
   * Clears all added markers and calculated clusters.
   */
  clear = async () => {
    await this.wasmReady;
    this.previousClusters = [];
    return this.clusterer.clear();
  }
  /**
   * Clears only calculated clusters.
   */
  clearClusters = async () => {
    await this.wasmReady;
    this.previousClusters = [];
    return this.clusterer.clearClusters();
  }
}