import type { ExpressionSpecification } from "maplibre-gl";

export { default as BasemapPicker } from "./BasemapPicker.svelte";
export { default as DisableInteractiveLayers } from "./DisableInteractiveLayers.svelte";
export { default as DotMarker } from "./DotMarker.svelte";
export { default as HelpButton } from "./HelpButton.svelte";
export { default as Link } from "./Link.svelte";
export { default as StreetView } from "./StreetView.svelte";
export { layerId } from "./zorder";

// TS fix for the imprecise geojson types
export function gjPosition(pt: number[]): [number, number] {
  return pt as [number, number];
}

// Zoom-dependant line width, adapted from from the Minor road layer (secondary
// road class) from https://api.maptiler.com/maps/streets-v2/style.json.
export function roadLineWidth(extraWidth: number): ExpressionSpecification {
  return [
    "interpolate",
    ["linear"],
    ["zoom"],
    5,
    0.5 + extraWidth,
    10,
    1 + extraWidth,
    12,
    1.5 + extraWidth,
    14,
    4 + extraWidth,
    16,
    7 + extraWidth,
    20,
    24 + extraWidth,
  ];
}

export function prettyPrintDistance(meters: number): string {
  if (meters < 1000.0) {
    return Math.round(meters) + "m";
  }
  return (meters / 1000.0).toFixed(1) + "km";
}

export function prettyPrintTime(seconds: number): string {
  if (seconds < 60.0) {
    return Math.round(seconds) + "s";
  }
  let minutes = Math.floor(seconds / 60);
  let leftover = Math.round(seconds - minutes * 60);
  return `${minutes}m${leftover}s`;
}
