import type { Map } from "maplibre-gl";

// Each MapTiler basemap style uses different layer IDs for roads and paths
export function getRoadLayerNames(map: Map, maptilerStyle: string): string[] {
  if (maptilerStyle == "dataviz") {
    return ["Road network", "Path"];
  }
  if (maptilerStyle == "streets") {
    let layers = [];
    for (let outer of ["road", "bridge", "tunnel"]) {
      for (let inner of [
        "link",
        "minor",
        "minor_construction",
        "motorway",
        "motorway_construction",
        "motorway_link",
        "path_pedestrian",
        "secondary_tertiary",
        "secondary_tertiary_construction",
        "service_track",
        "service_track_construction",
        "street",
        "trunk_primary",
        "trunk_primary_construction",
        "trunk_primary_link",
      ]) {
        layers.push(`${outer}_${inner}`);
      }
    }
    return layers;
  }
  if (maptilerStyle == "hybrid") {
    return ["Path", "Road", "Tunnel"];
  }
  if (maptilerStyle == "uk-openzoomstack-light") {
    return (
      map
        .getStyle()
        // @ts-expect-error It does exist
        .layers.filter((layer) => layer["source-layer"] == "roads")
        .map((layer) => layer.id)
    );
  }
  throw new Error(`Unknown style ${maptilerStyle}`);
}
