import type { Feature, Polygon } from "geojson";
import { RouteTool } from "route-snapper-ts";
import { get, writable, type Writable } from "svelte/store";

export let routeTool: Writable<RouteTool | null> = writable(null);

export interface Waypoint {
  point: [number, number];
  snapped: boolean;
}

export function calculateArea(waypoints: Waypoint[]): Feature<Polygon> {
  // TODO Or just fail?
  if (waypoints.length < 3) {
    return JSON.parse(get(routeTool)!.inner.calculateRoute(waypoints));
  }

  // Glue the end to the start
  let copy = JSON.parse(JSON.stringify(waypoints));
  copy.push(copy[0]);
  let out = JSON.parse(get(routeTool)!.inner.calculateRoute(copy));
  out.properties.waypoints.pop();
  out.geometry.type = "Polygon";
  out.geometry.coordinates = [out.geometry.coordinates];
  return out;
}
