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
  if (mapStyle == "streets-v2") {
    return map
      .getStyle()
      .layers.filter(
        (layer) =>
          // @ts-expect-error source-layer is present
          layer["source-layer"] == "transportation" && layer.type == "line",
      )
      .map((layer) => layer.id);
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
