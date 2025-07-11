import type {
  DataDrivenPropertyValueSpecification,
  ExpressionSpecification,
} from "maplibre-gl";
import { downloadGeneratedFile } from "svelte-utils";
import { get } from "svelte/store";
import tippy from "tippy.js";
import { appFocus, projectStorage } from "../stores";
import { type ProjectID, type StudyAreaName } from "./ProjectStorage";

export { default as ContextLayerButton } from "./ContextLayerButton.svelte";
export { default as DisableInteractiveLayers } from "./DisableInteractiveLayers.svelte";
export { default as DotMarker } from "./DotMarker.svelte";
export { default as HelpButton } from "./HelpButton.svelte";
export { default as Link } from "./Link.svelte";
export { default as Loading } from "./Loading.svelte";
export { default as ModeLink } from "./ModeLink.svelte";
export { default as PrevNext } from "./PrevNext.svelte";
export { default as StreetView } from "./StreetView.svelte";
export { layerId } from "./zorder";
export { Style } from "./colors";
export { pageTitle } from "./navbar";

// TS fix for the imprecise geojson types
export function gjPosition(pt: number[]): [number, number] {
  return pt as [number, number];
}

/// Any component using tooltips must call this in onMount
///
/// Add tooltips like this:
///   <button data-tippy-content="My Tooltip">...</button>;
/// Then call this method in onMount:
export function initTooltips() {
  tippy("[data-tippy-content]");
}

// Zoom-dependant line width, adapted from from the Minor road layer from
// https://api.maptiler.com/maps/streets-v2/style.json. If there's a road_kind
// property, this is used to thin out some lines.
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
    byClass(1.5 + extraWidth, 1 + extraWidth),
    14,
    byClass(4 + extraWidth, 2 + extraWidth),
    16,
    byClass(7 + extraWidth, 4 + extraWidth),
    20,
    byClass(24 + extraWidth, 16 + extraWidth),
  ];
}

function byClass(thick: number, thin: number): ExpressionSpecification {
  return [
    "match",
    ["get", "road_kind"],
    ["private", "pedestrian", "service"],
    thin,
    thick,
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

export function prettyPrintPercent(part: number, total: number): string {
  if (total === 0) {
    return "0%";
  }
  let percent = Math.round((part / total) * 100);
  return `${percent}%`;
}

export function prettyPrintStudyAreaName(studyAreaName: StudyAreaName): string {
  if (!studyAreaName) {
    return "custom area";
  }
  let focus = get(appFocus);
  if (focus == "cnt" || focus == "england") {
    return stripPrefix(studyAreaName, "LAD_");
  } else {
    return studyAreaName;
  }
}

export function downloadProject(projectID: ProjectID) {
  let project = get(projectStorage).project(projectID);
  let dateFormatted = new Date().toISOString().split("T")[0];
  let filename = `${project.project_name}-${dateFormatted}.geojson`;
  downloadGeneratedFile(filename, JSON.stringify(project));
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

// Fetch a URL, throwing if the HTTP response isn't OK. If you want progress
// updates, use fetchWithProgress.
export async function safeFetch(url: string): Promise<Response> {
  let response = await fetch(url);
  if (!response.ok) {
    throw new Error(`${url} not OK: ${response.status}`);
  }
  return response;
}

export function sum(list: number[]): number {
  return list.reduce((total, x) => total + x, 0);
}

export function stripPrefix(value: string, prefix: string): string {
  return value.startsWith(prefix) ? value.slice(prefix.length) : value;
}

export function stripSuffix(value: string, suffix: string): string {
  return value.endsWith(suffix) ? value.slice(0, -suffix.length) : value;
}

// This is a replacement for `svelte.tick`, which doesn't seem to work for some
// reason. Wait for two frames, to give the Loading component a chance to
// update, before doing someting blocking on the UI thread.
export async function refreshLoadingScreen(): Promise<void> {
  await new Promise((resolve) => {
    requestAnimationFrame(() => {
      requestAnimationFrame(resolve);
    });
  });
}
