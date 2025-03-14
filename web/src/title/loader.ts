import type { Feature, FeatureCollection, Polygon } from "geojson";
import { RouteTool } from "route-snapper-ts";
import { emptyGeojson } from "svelte-utils/map";
import { overpassQueryForPolygon } from "svelte-utils/overpass";
import { get, writable } from "svelte/store";
import { safeFetch, type ProjectID, type StudyAreaName } from "../common";
import { routeTool } from "../common/draw_area/stores";
import {
  appFocus,
  assetUrl,
  backend,
  currentProjectID,
  map,
  mode,
  projectStorage,
} from "../stores";
import { Backend } from "../wasm";

/**
 * Loads an existing project by its ID
 */
export async function loadProject(projectID: ProjectID) {
  let { projectSummary, features } = get(projectStorage).getProject(projectID);
  let { studyAreaName } = projectSummary;
  try {
    console.time("get input files");
    let { osmBuffer, demandBuffer, contextDataBuffer, boundary } =
      await getInputFiles(features, studyAreaName, get(appFocus) == "cnt");
    console.timeEnd("get input files");
    console.time("load");
    backend.set(
      new Backend(
        new Uint8Array(osmBuffer),
        demandBuffer ? new Uint8Array(demandBuffer) : undefined,
        contextDataBuffer ? new Uint8Array(contextDataBuffer) : undefined,
        boundary,
        studyAreaName,
      ),
    );
    // TODO Rename savefile -> project? Or combine this call with the constructor?
    get(backend)!.loadSavefile(features);
    currentProjectID.set(projectID);
    console.timeEnd("load");
    afterProjectLoaded(projectID);
  } catch (err) {
    console.error("Couldn't load project", err);
    window.alert(`Couldn't open project: ${err}`);
  }
}

// Returns OSM input, optional demand model input, and the boundary polygon,
// either from pre-hosted files or from Overpass.
async function getInputFiles(
  projectFeatureCollection: FeatureCollection,
  studyAreaName: StudyAreaName,
  isCnt: boolean,
): Promise<{
  osmBuffer: ArrayBuffer;
  boundary: Feature<Polygon>;
  demandBuffer?: ArrayBuffer;
  contextDataBuffer?: ArrayBuffer;
}> {
  if (isCnt) {
    // CNT projects are stored in a different place
    console.assert!(studyAreaName, "CNT projects must have a study area name");
    let url1 = assetUrl(`cnt_osm/${studyAreaName}.osm.pbf`);
    console.log(`Grabbing ${url1}`);
    let resp1 = await safeFetch(url1);
    let osmBuffer = await resp1.arrayBuffer();

    let url2 = assetUrl(`cnt_boundaries/${studyAreaName}.geojson`);
    let resp2 = await safeFetch(url2);
    let boundary = await resp2.json();

    let url3 = assetUrl(`cnt_demand/demand_${studyAreaName}.bin`);
    console.log(`Grabbing ${url3}`);
    let demandBuffer = undefined;
    try {
      let resp3 = await safeFetch(url3);
      demandBuffer = await resp3.arrayBuffer();
    } catch (err) {
      console.log(`No demand model: ${err}`);
    }

    let url4 = assetUrl(`cnt_prioritization/context_${studyAreaName}.bin`);
    console.log(`Grabbing ${url4}`);
    let contextDataBuffer = undefined;
    try {
      let resp = await safeFetch(url4);
      contextDataBuffer = await resp.arrayBuffer();
    } catch (err) {
      console.log(`No context data for prioritization: ${err}`);
    }

    return { osmBuffer, boundary, demandBuffer, contextDataBuffer };
  } else if (studyAreaName) {
    let url1 = assetUrl(`severance_pbfs/${studyAreaName}.pbf`);
    console.log(`Grabbing ${url1}`);
    let resp1 = await safeFetch(url1);
    let osmBuffer = await resp1.arrayBuffer();

    let url2 = assetUrl(`boundaries/${studyAreaName}.geojson`);
    let resp2 = await safeFetch(url2);
    let boundary = await resp2.json();

    return { osmBuffer, boundary };
  } else {
    console.log(`Grabbing from Overpass`);
    let boundary = projectFeatureCollection.features.find(
      (f: Feature) => f.properties!.kind == "study_area_boundary",
    ) as Feature<Polygon>;
    let resp = await safeFetch(overpassQueryForPolygon(boundary));
    let osmBuffer = await resp.arrayBuffer();
    return { osmBuffer, boundary };
  }
}

function afterProjectLoaded(projectID: ProjectID) {
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
  get(map)!.fitBounds(get(backend)!.getBounds(), { duration: 500 });

  // Update the URL
  let url = new URL(window.location.href);
  url.searchParams.set("project", projectID);
  window.history.replaceState(null, "", url.toString());
}
