import { RouteTool } from "route-snapper-ts";
import { writable, type Writable } from "svelte/store";

export let routeTool: Writable<RouteTool | null> = writable(null);

export interface Waypoint {
  point: [number, number];
  snapped: boolean;
}
