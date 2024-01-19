import type { GeoJSON } from "geojson";
import { writable, type Writable } from "svelte/store";

// These are necessary to communicate between components nested under the sidebar and map

export const polygonToolGj: Writable<GeoJSON> = writable({
  type: "FeatureCollection",
  features: [],
});
export const undoLength: Writable<number> = writable(0);
