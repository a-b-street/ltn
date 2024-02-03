import { LTN } from "backend";
import type { Feature, Polygon } from "geojson";
import { LngLat, type Map } from "maplibre-gl";
import { writable, type Writable } from "svelte/store";
import { RouteTool } from "./common/snapper/route_tool";

export const maptilerApiKey = "MZEJTanw3WpxRvt7qDfo";

export type Mode =
  | {
      mode: "title";
    }
  | {
      mode: "network";
    }
  | {
      mode: "set-boundary";
      name: string;
      existing: Feature<Polygon> | null;
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
    }
  | {
      mode: "debug-gj";
    };

export let map: Writable<Map | null> = writable(null);
export let mapStyle: Writable<string> = writable(
  `https://api.maptiler.com/maps/dataviz/style.json?key=${maptilerApiKey}`,
);

export let example: Writable<string> = writable("");
export let showAbout: Writable<boolean> = writable(true);

export let topContents: Writable<HTMLDivElement | null> = writable(null);
export let sidebarContents: Writable<HTMLDivElement | null> = writable(null);
export let mapContents: Writable<HTMLDivElement | null> = writable(null);

export let app: Writable<LTN | null> = writable(null);
export let route_tool: Writable<RouteTool | null> = writable(null);
export let route_pt_a: Writable<LngLat> = writable(new LngLat(0, 0));
export let route_pt_b: Writable<LngLat> = writable(new LngLat(0, 0));
// A way for different components to know when internal app state has changed
// and they might need to rerender
export let mutationCounter: Writable<number> = writable(1);
export let mode: Writable<Mode> = writable({ mode: "title" });
