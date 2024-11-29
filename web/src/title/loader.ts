import { get } from "svelte/store";
import { LngLat } from "maplibre-gl";
import { LTN } from "backend";
import type { Feature, Polygon } from "geojson";
import { overpassQueryForPolygon } from "svelte-utils/overpass";
import { RouteTool } from "route-snapper-ts";
import {
  routeToolGj,
  snapMode,
  undoLength,
  showAllNodes,
  showAllNodesGj,
} from "../common/snapper/stores";
import {
  app,
  projectName,
  map,
  mode,
  useLocalVite,
  route_tool,
  route_pt_a,
  route_pt_b,
} from "../stores";

export async function loadFromLocalStorage(key: string) {
  projectName.set(key);
  try {
    let gj = JSON.parse(window.localStorage.getItem(key)!);

    console.time("get OSM input");
    let [buffer, boundary] = await getOsmInput(gj);
    console.timeEnd("get OSM input");
    console.time("load");
    app.set(
      new LTN(
        new Uint8Array(buffer),
        boundary,
        gj.study_area_name || undefined,
      ),
    );
    // TODO Rename savefile -> project? Or combine this call with the constructor?
    get(app)!.loadSavefile(gj);
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
  route_tool.set(
    new RouteTool(
      get(map)!,
      get(app)!.toRouteSnapper(),
      routeToolGj,
      snapMode,
      undoLength,
    ),
  );
  showAllNodes.set(false);
  showAllNodesGj.set({ type: "FeatureCollection", features: [] });
  get(map)!.fitBounds(
    Array.from(get(app)!.getBounds()) as [number, number, number, number],
    { animate: false },
  );
  route_pt_a.set(randomPoint());
  route_pt_b.set(randomPoint());
}

function randomPoint(): LngLat {
  let bounds = get(app)!.getBounds();
  let lng = bounds[0] + Math.random() * (bounds[2] - bounds[0]);
  let lat = bounds[1] + Math.random() * (bounds[3] - bounds[1]);
  return new LngLat(lng, lat);
}
