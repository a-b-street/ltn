import { LTN } from "backend";
import type { Feature, Polygon } from "geojson";
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
      prevMode: Mode;
    };

export let app: Writable<LTN | null> = writable(null);
export let mode: Writable<Mode> = writable({ mode: "network" });

export let sidebarContents = writable(null);
export let mapContents = writable(null);
