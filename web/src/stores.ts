import { LTN } from "backend";
import type { Feature, Polygon } from "geojson";
import type { Map } from "maplibre-gl";
import { writable, type Writable } from "svelte/store";

export type Mode =
  | {
      mode: "network";
    }
  | {
      mode: "set-boundary";
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
    };

export let app: Writable<LTN | null> = writable(null);
export let mode: Writable<Mode> = writable({ mode: "network" });
export let showBasemap: Writable<boolean> = writable(false);
export let map: Writable<Map | null> = writable(null);

export let sidebarContents: Writable<HTMLDivElement | null> = writable(null);
export let mapContents: Writable<HTMLDivElement | null> = writable(null);
