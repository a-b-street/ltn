import type { Feature, Polygon } from "geojson";
import {
  LngLat,
  LngLatBounds,
  type LngLatBoundsLike,
  type Map,
} from "maplibre-gl";
import { type AreaProps } from "route-snapper-ts";
import { get, writable, type Writable } from "svelte/store";
import { projectStorage } from "./title/loader";
import type { Backend } from "./wasm";

// NOTE: our maptiler apiKey is baked into the customized assets/map-styles/, so if we rotate keys, we'll need to update that file too.
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
      mode: "pick-neighbourhood";
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
      prevMode:
        | "pick-neighbourhood"
        | "neighbourhood"
        | "impact-one-destination";
    }
  | {
      mode: "predict-impact";
    }
  | {
      mode: "impact-detail";
      road: Feature;
    }
  | {
      mode: "debug-intersections";
    }
  | {
      mode: "debug-neighbourhood";
    }
  | {
      mode: "debug-demand";
    };

export let map: Writable<Map | null> = writable(null);

// The exact key in local storage
export let currentProjectKey: Writable<string> = writable("");
// False until user activates
export let showAbout: Writable<boolean> = writable(false);

export let appFocus: Writable<"global" | "cnt"> = writable("global");

export let backend: Writable<Backend | null> = writable(null);
export let routePtA: Writable<LngLat> = writable(new LngLat(0, 0));
export let routePtB: Writable<LngLat> = writable(new LngLat(0, 0));
export let oneDestination: Writable<LngLat> = writable(new LngLat(0, 0));
export let mainRoadPenalty: Writable<number> = writable(1.0);
// A way for different components to know when internal app state has changed
// and they might need to rerender
export let mutationCounter: Writable<number> = writable(1);
export let mode: Writable<Mode> = writable({ mode: "title", firstLoad: true });

// Settings
export let devMode: Writable<boolean> = writable(false);
export let maptilerBasemap: Writable<string> = writable("streets-v2");
export let filterType: Writable<string> = writable("walk_cycle_only");
export let animateShortcuts = writable(false);
export let roadStyle: Writable<"shortcuts" | "cells" | "edits" | "speeds"> =
  writable("shortcuts");
export let thickRoadsForShortcuts = writable(false);

export function saveCurrentProject() {
  const key = get(currentProjectKey);
  try {
    projectStorage.saveProject(key, JSON.stringify(get(backend)!.toSavefile()));
  } catch (err) {
    window.alert(`${err}`);
  }
}

export let useLocalVite: Writable<boolean> = writable(false);

export function assetUrl(path: string): string {
  return get(useLocalVite) ? `/${path}` : `https://assets.od2net.org/${path}`;
}

export function returnToChooseProject() {
  mode.set({ mode: "title", firstLoad: false });

  let bounds = [-180, -90, 180, 90] as LngLatBoundsLike;
  if (get(appFocus) == "cnt") {
    bounds = [-8.943, 54.631, -0.901, 59.489];
  }
  get(map)?.fitBounds(bounds, { duration: 500 });
}

export function ensurePointInVisibleBounds(point: Writable<LngLat>) {
  function randomPoint(bounds: LngLatBounds): LngLat {
    const width = bounds.getEast() - bounds.getWest();
    let lng = bounds.getWest() + Math.random() * width;

    const height = bounds.getNorth() - bounds.getSouth();
    let lat = bounds.getSouth() + Math.random() * height;

    return new LngLat(lng, lat);
  }

  const bounds: LngLatBounds | undefined = get(map)?.getBounds();
  if (!bounds) {
    console.assert(false, "missing map bounds");
    return;
  }

  if (!bounds.contains(get(point))) {
    point.set(randomPoint(bounds));
  }
}
