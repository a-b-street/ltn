import { LTN } from "backend";
import type { Feature, Polygon } from "geojson";
import type { Map } from "maplibre-gl";
import { writable, type Writable } from "svelte/store";
import { RouteTool } from "./common/snapper/route_tool";

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
    }
  | {
      mode: "debug";
    };

export let app: Writable<LTN | null> = writable(null);
// A way for different components to know when internal app state has changed
// and they might need to rerender
export let mutationCounter: Writable<number> = writable(1);
export let mode: Writable<Mode> = writable({ mode: "title" });
export let showBasemap: Writable<boolean> = writable(true);
export let map: Writable<Map | null> = writable(null);
export let route_tool: Writable<RouteTool | null> = writable(null);

export let example: Writable<string> = writable("");
export let showAbout: Writable<boolean> = writable(true);

export let sidebarContents: Writable<HTMLDivElement | null> = writable(null);
export let mapContents: Writable<HTMLDivElement | null> = writable(null);
