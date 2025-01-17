import type { Feature, Polygon } from "geojson";
import { LngLat, type Map } from "maplibre-gl";
import { type AreaProps } from "route-snapper-ts";
import { get, writable, type Writable } from "svelte/store";
import type { Backend } from "./wasm";

export const maptilerApiKey = "MZEJTanw3WpxRvt7qDfo";

export type Mode =
  | {
      mode: "title";
      firstLoad: boolean;
    }
  | {
      mode: "new-project";
    }
  | {
      mode: "network";
    }
  | {
      mode: "set-boundary";
      name: string;
      existing: Feature<Polygon, AreaProps> | null;
    }
  | {
      mode: "auto-boundaries";
    }
  | {
      mode: "neighbourhood";
    }
  | {
      mode: "view-shortcuts";
    }
  | {
      mode: "impact-one-destination";
    }
  | {
      mode: "route";
      prevMode: "network" | "neighbourhood" | "impact-one-destination";
    }
  | {
      mode: "predict-impact";
    }
  | {
      mode: "debug";
    };

export let map: Writable<Map | null> = writable(null);

export let useLocalVite: Writable<boolean> = writable(false);
// The exact key in local storage
export let projectName: Writable<string> = writable("");
export let showAbout: Writable<boolean> = writable(true);

export let backend: Writable<Backend | null> = writable(null);
export let route_pt_a: Writable<LngLat> = writable(new LngLat(0, 0));
export let route_pt_b: Writable<LngLat> = writable(new LngLat(0, 0));
export let one_destination: Writable<LngLat> = writable(new LngLat(0, 0));
export let mainRoadPenalty: Writable<number> = writable(1.0);
// A way for different components to know when internal app state has changed
// and they might need to rerender
export let mutationCounter: Writable<number> = writable(1);
export let mode: Writable<Mode> = writable({ mode: "title", firstLoad: true });

// Settings
export let maptilerBasemap: Writable<string> = writable("streets-v2");
export let filterType: Writable<string> = writable("walk_cycle_only");
export let animateShortcuts = writable(false);
export let editPerimeterRoads = writable(false);
export let roadStyle: Writable<"shortcuts" | "cells" | "edits"> =
  writable("shortcuts");
export let thickRoadsForShortcuts = writable(false);

export function autosave() {
  let key = get(projectName);
  if (!key) {
    window.alert("Autosave failed; no projectName set?!");
  }
  window.localStorage.setItem(key, JSON.stringify(get(backend)!.toSavefile()));
}
