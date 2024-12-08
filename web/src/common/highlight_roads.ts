import type { Map } from "maplibre-gl";

// Each basemap style uses different layer IDs for roads and paths
export function getRoadLayerNames(map: Map, mapStyle: string): string[] {
  // The styles may change over time. Guarantee we only return valid line layers.
  let availableLayers = new Set(
    map
      .getStyle()
      .layers.filter((l) => l.type == "line")
      .map((l) => l.id),
  );

  if (mapStyle == "dataviz") {
    return ["Road network", "Path"].filter((l) => availableLayers.has(l));
  }
  if (mapStyle == "hybrid") {
    return ["Path", "Road", "Tunnel"].filter((l) => availableLayers.has(l));
  }
  // TODO Check if this is true in the new version
  if (mapStyle == "streets-v2") {
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
    return layers.filter((l) => availableLayers.has(l));
  }
  if (mapStyle == "uk-openzoomstack-light") {
    return map
      .getStyle()
      .layers.filter(
        // @ts-expect-error source-layer is present
        (layer) => layer["source-layer"] == "roads" && layer.type == "line",
      )
      .map((layer) => layer.id);
  }

  return [];
}
