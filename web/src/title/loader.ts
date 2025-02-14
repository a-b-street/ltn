import type { Feature, Polygon } from "geojson";
import { LngLat } from "maplibre-gl";
import { RouteTool } from "route-snapper-ts";
import { emptyGeojson } from "svelte-utils/map";
import { overpassQueryForPolygon } from "svelte-utils/overpass";
import { get, writable } from "svelte/store";
import { safeFetch } from "../common";
import { routeTool } from "../common/draw_area/stores";
import {
  assetUrl,
  backend,
  map,
  mode,
  one_destination,
  projectName,
  route_pt_a,
  route_pt_b,
} from "../stores";
import { Backend } from "../wasm";

// Returns whether the project name is already taken, otherwise the project is created.
export async function createNewProject(
  kind: "ltn_cnt" | "ltn",
  studyAreaName: string,
  projectName: string,
): Promise<boolean> {
  console.assert(studyAreaName != "");
  console.assert(projectName != "");

  let key = `${kind}/${studyAreaName}/${projectName}`;

  if (window.localStorage.getItem(key)) {
    return false;
  }

  window.localStorage.setItem(
    key,
    JSON.stringify({
      type: "FeatureCollection",
      features: [],
      study_area_name: studyAreaName,
    }),
  );
  await loadFromLocalStorage(key);
  return true;
}

export async function loadFromLocalStorage(key: string) {
  // REVIEW: rename this to "projectKey" or something - it's more than just the projectName.
  projectName.set(key);
  try {
    let gj = JSON.parse(window.localStorage.getItem(key)!);

    console.time("get input files");
    let { osmBuffer, demandBuffer, contextDataBuffer, boundary } =
      await getInputFiles(gj, key.startsWith("ltn_cnt/"));
    console.timeEnd("get input files");
    console.time("load");
    backend.set(
      new Backend(
        new Uint8Array(osmBuffer),
        demandBuffer ? new Uint8Array(demandBuffer) : undefined,
        contextDataBuffer ? new Uint8Array(contextDataBuffer) : undefined,
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

// Returns OSM input, optional demand model input, and the boundary polygon,
// either from pre-hosted files or from Overpass.
async function getInputFiles(
  gj: any,
  isCnt: boolean,
): Promise<{
  osmBuffer: ArrayBuffer;
  boundary: Feature<Polygon>;
  demandBuffer?: ArrayBuffer;
  contextDataBuffer?: ArrayBuffer;
}> {
  if (isCnt) {
    // CNT projects are stored in a different place
    let url1 = assetUrl(`cnt_osm/${gj.study_area_name}.osm.pbf`);
    console.log(`Grabbing ${url1}`);
    let resp1 = await safeFetch(url1);
    let osmBuffer = await resp1.arrayBuffer();

    let url2 = assetUrl(`cnt_boundaries/${gj.study_area_name}.geojson`);
    let resp2 = await safeFetch(url2);
    let boundary = await resp2.json();

    let url3 = assetUrl(`cnt_demand/demand_${gj.study_area_name}.bin`);
    console.log(`Grabbing ${url3}`);
    let demandBuffer = undefined;
    try {
      let resp3 = await safeFetch(url3);
      demandBuffer = await resp3.arrayBuffer();
    } catch (err) {
      console.log(`No demand model: ${err}`);
    }

    let url4 = assetUrl(`cnt_prioritization/context_${gj.study_area_name}.bin`);
    console.log(`Grabbing ${url4}`);
    let contextDataBuffer = undefined;
    try {
      let resp = await safeFetch(url4);
      contextDataBuffer = await resp.arrayBuffer();
    } catch (err) {
      console.log(`No context data for prioritization: ${err}`);
    }

    return { osmBuffer, boundary, demandBuffer, contextDataBuffer };
  } else if (gj.study_area_name) {
    let url1 = assetUrl(`severance_pbfs/${gj.study_area_name}.pbf`);
    console.log(`Grabbing ${url1}`);
    let resp1 = await safeFetch(url1);
    let osmBuffer = await resp1.arrayBuffer();

    let url2 = assetUrl(`boundaries/${gj.study_area_name}.geojson`);
    let resp2 = await safeFetch(url2);
    let boundary = await resp2.json();

    return { osmBuffer, boundary };
  } else {
    console.log(`Grabbing from Overpass`);
    let boundary = gj.features.find(
      (f: Feature) => f.properties!.kind == "study_area_boundary",
    )!;
    let resp = await safeFetch(overpassQueryForPolygon(boundary));
    let osmBuffer = await resp.arrayBuffer();
    return { osmBuffer, boundary };
  }
}

export function afterProjectLoaded() {
  mode.set({
    mode: "pick-neighbourhood",
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
