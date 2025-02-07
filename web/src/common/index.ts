import type {
  DataDrivenPropertyValueSpecification,
  ExpressionSpecification,
} from "maplibre-gl";

export { default as BasemapPicker } from "./BasemapPicker.svelte";
export { default as DisableInteractiveLayers } from "./DisableInteractiveLayers.svelte";
export { default as DotMarker } from "./DotMarker.svelte";
export { default as HelpButton } from "./HelpButton.svelte";
export { default as Link } from "./Link.svelte";
export { default as PrevNext } from "./PrevNext.svelte";
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

/**
 * Create a zoom-independent distance.
 *
 * This is useful (e.g.) for using symbol layers whose output reflects the size of a physical feature on the map.
 *
 * @param mapMeters The length on the map in meters
 * @param minimumPixels A minimum size (in pixels) at all zoom levels. You can set it to 0 if you want the feature to disappear at low zoom levels.
 */
export function mapMetersToPixels(
  mapMeters: number,
  minimumPixels: number = 4,
): DataDrivenPropertyValueSpecification<number> {
  // Random googling gives me these values - I'm not entirely sure they're correct but they seem about right.
  const minZoomPixelsPerMeter = 15654.303392;
  const maxZoomPixelsPerMeter = 0.0597164283;

  return [
    "interpolate",
    ["exponential", 2],
    ["zoom"],
    0,
    ["max", ["/", mapMeters, minZoomPixelsPerMeter], minimumPixels],
    22,
    ["max", ["/", mapMeters, maxZoomPixelsPerMeter], minimumPixels],
  ];
}

// Fetch a URL, throwing if the HTTP response isn't OK.
export async function safeFetch(url: string): Promise<Response> {
  let response = await fetch(url);
  if (!response.ok) {
    throw new Error(`${url} not OK: ${response.status}`);
  }
  return response;
}
