import type { GeoJSON } from "geojson";
import { writable, type Writable } from "svelte/store";

// These are necessary to communicate between components nested under the sidebar and map

export const routeToolGj: Writable<GeoJSON> = writable({
  type: "FeatureCollection",
  features: [],
});
export const snapMode: Writable<boolean> = writable(true);
export const undoLength: Writable<number> = writable(0);
export const showAllNodes: Writable<boolean> = writable(false);
export const showAllNodesGj: Writable<GeoJSON> = writable({
  type: "FeatureCollection",
  features: [],
});
