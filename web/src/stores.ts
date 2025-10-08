import type { Feature, Polygon } from "geojson";
import { LngLat, LngLatBounds, type Map } from "maplibre-gl";
import { type AreaProps } from "route-snapper-ts";
import {
  derived,
  get,
  writable,
  type Readable,
  type Writable,
} from "svelte/store";
import { stripPrefix } from "./common";
import {
  Database,
  ProjectStorage,
  type ProjectFeatureCollection,
  type ProjectID,
} from "./common/ProjectStorage";
import type { Backend, MetricBuckets } from "./wasm";

// NOTE: our maptiler apiKey is baked into the customized assets/map-styles/, so if we rotate keys, we'll need to update that file too.
export const maptilerApiKey = "MZEJTanw3WpxRvt7qDfo";

export type Mode =
  | {
      mode: "title";
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
      existing: Feature<Polygon, AreaProps>;
    }
  | {
      mode: "add-neighbourhood";
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
      prevMode: "pick-neighbourhood" | "neighbourhood";
    }
  | {
      mode: "impact-detail";
      road: Feature;
      prevPrevMode: "pick-neighbourhood" | "neighbourhood";
    }
  | {
      mode: "debug-intersections";
    }
  | {
      mode: "debug-neighbourhood";
    }
  | {
      mode: "debug-demand";
    }
  | {
      mode: "debug-traffic";
    };

export type AppFocus = "global" | "cnt" | "england";

export let map: Writable<Map | undefined> = writable(undefined);

export let appFocus: Writable<AppFocus> = writable("global");
// The id of the project currently being worked on
export let currentProjectID: Writable<ProjectID | undefined> =
  writable(undefined);
export let currentProject: Readable<ProjectFeatureCollection | undefined> =
  derived(currentProjectID, ($id) => {
    if ($id) {
      return get(projectStorage).project($id);
    } else {
      return undefined;
    }
  });

export let currentNeighbourhoodName: Writable<string | undefined> =
  writable(undefined);

export let database = new Database();
export let projectStorage: Readable<ProjectStorage> = derived(
  appFocus,
  ($appFocus) => database.projectStorage($appFocus),
);

export let firstTimeLoadProjectFromURL = writable(true);

export let backend: Writable<Backend | null> = writable(null);
// This changes alongside the backend and isn't valid for non-CNT areas. Rather
// than express `| null` in TS and deal with it everywhere, just set default
// values that are clearly visibly broken, since we never expect these to be
// used when invalid.
export let metricBuckets: Writable<MetricBuckets> = writable({
  population_density: [0, 1, 2, 3, 4, 5],
  collision_density: [0, 1, 2, 3, 4, 5],
  poi_density: [0, 1, 2, 3, 4, 5],
});
export let routePtA: Writable<LngLat> = writable(new LngLat(0, 0));
export let routePtB: Writable<LngLat> = writable(new LngLat(0, 0));
export let oneDestination: Writable<LngLat> = writable(new LngLat(0, 0));
export let mainRoadPenalty: Writable<number> = writable(1.0);
// A way for different components to know when internal app state has changed
// and they might need to rerender
export let mutationCounter: Writable<number> = writable(1);
export let mode: Writable<Mode> = writable({ mode: "title", firstLoad: true });

// Settings
export let showBeforeEdits: Writable<boolean> = writable(false);
export let devMode: Writable<boolean> = writable(false);
export let maptilerBasemap: Writable<string> = writable("streets-v2");
export let currentFilterType: Writable<string> = writable("walk_cycle_only");
export let animateShortcuts = writable(false);
export let drawBorderEntries = writable(true);
// This default value must match the backend
export let hideUnimportantCells = writable(false);
export let showExistingFiltersAndTRs = writable(true);
export let roadStyle: Writable<"shortcuts" | "cells" | "edits" | "speeds"> =
  writable("shortcuts");
export let thickRoadsForShortcuts = writable(false);
export let useMetricUnits = writable(false);

// Settings for impact prediction
export let fastSample: Writable<boolean> = writable(true);
export let minImpactCount: Writable<number> = writable(500);

export function saveCurrentProject() {
  const projectID = get(currentProjectID)!;
  try {
    get(projectStorage).saveProject(projectID, get(backend)!.toSavefile());
  } catch (err) {
    window.alert(`Autosave failed: ${err}`);
  }
}

export let useLocalVite: Writable<boolean> = writable(false);

export let locale = writable(languageFromURL());

export function languageFromURL(): string {
  let param = new URL(window.location.href).searchParams.get("lang");
  if (param && ["en", "fr", "hu"].includes(param)) {
    return param;
  }
  return "en";
}

export function assetUrl(path: string): string {
  if (get(useLocalVite)) {
    return `/${path}`;
  }

  if (path.startsWith("cnt/")) {
    return `https://assets.cnt.scot/${stripPrefix(path, "cnt/")}`;
  }
  if (path.startsWith("england/")) {
    return `https://assets.od2net.org/ltn/${path}`;
  }

  return `https://assets.od2net.org/${path}`;
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

export function printSpeed(mph: number): string {
  if (get(useMetricUnits)) {
    let kph = Math.round(1.60934 * mph);
    return `${kph} km/h`;
  } else {
    return `${Math.round(mph)} mph`;
  }
}
