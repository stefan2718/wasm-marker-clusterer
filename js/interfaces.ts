export interface IConfig {
  /** 
   * Size of the square (in pixels) that each cluster accumulates markers from.
   * Default: 60
   */
  gridSize?: number;
  /** 
   * Whether the center of a cluster should be recalculated when a marker is added.
   * If false, center is always equal to the first marker added. 
   * Default: false
   */
  averageCenter?: boolean;
  /** 
   * Whether to log to the console the time spent clustering and passing data between Wasm/JS 
   * Default: false
   */
  logTime?: boolean;
  /**
   * Whether the Wasm module should only return clusters that have changed since the last call. 
   * Setting to true will save time spent serializing data between Wasm/JS
   * Default: true
   */
  onlyReturnModifiedClusters?: boolean;
}

export interface IMarker {
  lat: number;
  lng: number;
}

export interface ICluster {
  uuid?: string;
  size: number;
  center: IMarker;
  bounds?: IBounds;
  markers: IMarker[];
}

export interface IBounds {
  north: number;
  east: number;
  south: number;
  west: number;
} 