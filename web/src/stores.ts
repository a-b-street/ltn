import { LTN } from "backend";
import type { Feature, Polygon } from "geojson";
import type { Map } from "maplibre-gl";
import { writable, type Writable } from "svelte/store";

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
export let mode: Writable<Mode> = writable({ mode: "title" });
export let showBasemap: Writable<boolean> = writable(true);
export let map: Writable<Map | null> = writable(null);

export let example: Writable<string> = writable("");

export let sidebarContents: Writable<HTMLDivElement | null> = writable(null);
export let mapContents: Writable<HTMLDivElement | null> = writable(null);