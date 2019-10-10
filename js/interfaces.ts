export interface IConfig {
  gridSize?: number;
  averageCenter?: boolean;
  logTime?: boolean;
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