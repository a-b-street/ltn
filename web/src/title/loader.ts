import type { Feature, Polygon } from "geojson";
import { LngLat } from "maplibre-gl";
import { RouteTool } from "route-snapper-ts";
import { emptyGeojson } from "svelte-utils/map";
import { overpassQueryForPolygon } from "svelte-utils/overpass";
import { get, writable } from "svelte/store";
import { routeTool } from "../common/draw_area/stores";
import {
  backend,
  map,
  mode,
  one_destination,
  projectName,
  route_pt_a,
  route_pt_b,
  useLocalVite,
} from "../stores";
import { Backend } from "../wasm";

export async function loadFromLocalStorage(key: string) {
  projectName.set(key);
  try {
    let gj = JSON.parse(window.localStorage.getItem(key)!);

    console.time("get OSM input");
    let [buffer, boundary] = await getOsmInput(gj);
    console.timeEnd("get OSM input");
    console.time("load");
    backend.set(
      new Backend(
        new Uint8Array(buffer),
        boundary,
        gj.study_area_name || undefined,
      ),
    );
    // TODO Rename savefile -> project? Or combine this call with the constructor?
    get(backend)!.loadSavefile(gj);
    console.timeEnd("load");

    afterProjectLoaded();
  } catch (err) {
    window.alert(`Couldn't open project: ${err}`);
    projectName.set("");
  }
}

// Returns OSM input and the boundary polygon, either from a pre-hosted pbf
// file or from Overpass.
async function getOsmInput(gj: any): Promise<[ArrayBuffer, Feature<Polygon>]> {
  if (gj.study_area_name) {
    let url1 = get(useLocalVite)
      ? `/osm/${gj.study_area_name}.pbf`
      : `https://assets.od2net.org/severance_pbfs/${gj.study_area_name}.pbf`;
    console.log(`Grabbing ${url1}`);
    let resp1 = await fetch(url1);
    let bytes = await resp1.arrayBuffer();

    let url2 = get(useLocalVite)
      ? `/boundaries/${gj.study_area_name}.geojson`
      : `https://assets.od2net.org/boundaries/${gj.study_area_name}.geojson`;
    let resp2 = await fetch(url2);
    let boundary = await resp2.json();

    return [bytes, boundary];
  } else {
    console.log(`Grabbing from Overpass`);
    let study_area_boundary = gj.features.find(
      (f: Feature) => f.properties!.kind == "study_area_boundary",
    )!;
    let resp = await fetch(overpassQueryForPolygon(study_area_boundary));
    let bytes = await resp.arrayBuffer();
    return [bytes, study_area_boundary];
  }
}

export function afterProjectLoaded() {
  mode.set({
    mode: "network",
  });
  // The stores are unused; the WASM API is used directly. This TS wrapper is unused.
  routeTool.set(
    new RouteTool(
      get(map)!,
      get(backend)!.toRouteSnapper(),
      writable(emptyGeojson()),
      writable(true),
      writable(0),
    ),
  );
  get(map)!.fitBounds(get(backend)!.getBounds(), { animate: false });
  route_pt_a.set(randomPoint());
  route_pt_b.set(randomPoint());
  one_destination.set(randomPoint());

  // Update the URL
  let url = new URL(window.location.href);
  url.searchParams.set("project", get(projectName));
  window.history.replaceState(null, "", url.toString());
}

function randomPoint(): LngLat {
  let bounds = get(backend)!.getBounds();
  let lng = bounds[0] + Math.random() * (bounds[2] - bounds[0]);
  let lat = bounds[1] + Math.random() * (bounds[3] - bounds[1]);
  return new LngLat(lng, lat);
}
