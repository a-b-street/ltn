import { LTN } from "backend";
import type { Feature, Polygon } from "geojson";
import { LngLat, type Map } from "maplibre-gl";
import { get, writable, type Writable } from "svelte/store";
import { type AreaProps } from "route-snapper-ts";

export const maptilerApiKey = "MZEJTanw3WpxRvt7qDfo";

export type Mode =
  | {
      mode: "title";
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
      mode: "route";
      prevMode: "network" | "neighbourhood";
    }
  | {
      mode: "debug";
    };

export let map: Writable<Map | null> = writable(null);
export let maptilerBasemap: Writable<string> = writable("streets-v2");

export let useLocalVite: Writable<boolean> = writable(false);
// The exact key in local storage
export let projectName: Writable<string> = writable("");
export let showAbout: Writable<boolean> = writable(true);

export let app: Writable<LTN | null> = writable(null);
export let route_pt_a: Writable<LngLat> = writable(new LngLat(0, 0));
export let route_pt_b: Writable<LngLat> = writable(new LngLat(0, 0));
export let mainRoadPenalty: Writable<number> = writable(1.0);
// A way for different components to know when internal app state has changed
// and they might need to rerender
export let mutationCounter: Writable<number> = writable(1);
export let mode: Writable<Mode> = writable({ mode: "title" });
export let filterType: Writable<string> = writable("walk_cycle_only");
export let animateShortcuts = writable(false);

export function autosave() {
  let key = get(projectName);
  if (!key) {
    window.alert("Autosave failed; no projectName set?!");
  }
  window.localStorage.setItem(key, get(app)!.toSavefile());
}
