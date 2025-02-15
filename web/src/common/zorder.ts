import { get } from "svelte/store";
import { map as mapStore } from "../stores";

interface LayerProps {
  id: string;
  beforeId: string | undefined;
  eventsIfTopMost: boolean;
}

// Use this helper for every svelte-maplibre layer component. It sets the layer
// ID, beforeId (for z-ordering between layers), and defaults to only using the
// top-most layer for hovering/clicking.
export function layerId(
  layerId: string,
  eventsIfTopMost: boolean = true,
): LayerProps {
  return {
    id: layerId,
    beforeId: getBeforeId(layerId),
    eventsIfTopMost,
  };
}

// Calculates the beforeId for adding a layer. Due to hot-module reloading and
// Svelte component initialization order being unpredictable, callers might add
// layers in any order. Use beforeId to guarantee the layers wind up in an
// explicitly defined order.
function getBeforeId(layerId: string): string | undefined {
  let map = get(mapStore);
  if (!map) {
    console.warn(
      `getBeforeId(${layerId}) called before map is ready. Z-ordering may be incorrect.`,
    );
    return undefined;
  }

  // Find the last layer currently in the map that should be on top of this new
  // layer.
  let beforeId = undefined;
  let found = false;
  for (let i = layerZorder.length - 1; i >= 0; i--) {
    let id = layerZorder[i];
    if (id == layerId) {
      found = true;
      break;
    }
    if (map.getLayer(id)) {
      beforeId = id;
    }
  }
  // When adding a new layer somewhere, force the programmer to decide where it
  // should be z-ordered.
  if (!found) {
    throw new Error(`Layer ID ${layerId} not defined in layerZorder`);
  }
  // If beforeId isn't set, we'll add the layer on top of everything else.
  return beforeId;
}

// Dummy functions just used for documentation below.
let streets = (x: string) => x;
//let satellite = (x: string) => x;
let dataviz = (x: string) => x;
//let zoomstack = (x: string) => x;

// All layer IDs used with layerId must be defined here, with later entries
// drawn on top.

// TODO The order itself needs to change when the basemap does, because dataviz
// and zoomstack are mutually incompatible (buildings and 0 restricted road
// ordering).

// Helpful docs:
// https://docs.maptiler.com/schema/planet/
// https://cloud.maptiler.com/maps/streets-v2/
const layerZorder = [
  streets(dataviz("Background")),

  // Ferry line starts at zoom 6, so these won't be visible before that -- but
  // that's fine; a boundary is too small to see before zoom 6 anyway
  "neighbourhood-boundaries",

  "neighbourhood-prioritization-simd",
  "neighbourhood-prioritization-area",

  streets("Ferry line"),

  "neighbourhood-boundaries-outline",

  "debug-borders",
  "debug-filters",

  "cells",
  "interior-roads-outlines",
  "interior-roads",
  "debug-intersections",
  "debug-movements-outline",
  "debug-movements-fill",
  "debug-editable-intersections",
  "editable-intersections",
  "intersection-filters",

  "compare-route",

  "shortcuts",
  "shortcuts-focus",

  "predict-impact",

  dataviz("Building"),

  "draw-area-lines",
  "draw-area-preview",

  "debug-demand-fill",
  "debug-demand-outline",

  "one-ways",

  // Contextual layers cover up most things
  "context-simd",
  "context-population-density",
  "context-population-outline",
  "context-traffic",
  "context-los",
  "context-existing-infra",
  "context-route-network",
  "context-bus-routes",
  "context-gp-practices",
  "context-hospitals",
  "context-schools",
  "context-railway-stations",
  "context-stats19",

  dataviz("Road labels"),
  streets("road_label"),

  "animate-shortcuts",

  "modal-filters",
  "turn-restrictions",

  "boundary",
  "neighbourhood-boundary",

  // These're outside the neighbourhood-boundary, but don't fade them
  "border-arrows",
  "border-arrow-outlines",

  "freehand-line",

  "edit-polygon-fill",
  "edit-polygon-lines",
  "edit-polygon-vertices",
];
