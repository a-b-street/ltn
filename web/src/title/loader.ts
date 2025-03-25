import type { Feature, MultiPolygon, Polygon } from "geojson";
import { RouteTool } from "route-snapper-ts";
import { emptyGeojson } from "svelte-utils/map";
import { overpassQueryForPolygon } from "svelte-utils/overpass";
import { get, writable } from "svelte/store";
import { safeFetch } from "../common";
import { routeTool } from "../common/draw_area/stores";
import {
  type ProjectFeatureCollection,
  type ProjectID,
} from "../common/ProjectStorage";
import {
  assetUrl,
  backend,
  currentProjectID,
  map,
  projectStorage,
} from "../stores";
import { Backend } from "../wasm";

/**
 * Loads an existing project by its ID
 */
export async function loadProject(projectID: ProjectID) {
  let project = get(projectStorage).project(projectID);
  try {
    console.time("get input files");
    let { osmBuffer, demandBuffer, contextDataBuffer, boundary } =
      await getInputFiles(project);
    console.timeEnd("get input files");
    console.time("load");
    backend.set(
      new Backend(
        new Uint8Array(osmBuffer),
        demandBuffer ? new Uint8Array(demandBuffer) : undefined,
        contextDataBuffer ? new Uint8Array(contextDataBuffer) : undefined,
        boundary,
        project.app_focus,
        project.study_area_name,
        project.project_name,
        project.db_schema_version,
      ),
    );
    // TODO Rename savefile -> project? Or combine this call with the constructor?
    get(backend)!.loadSavefile(project);
    currentProjectID.set(projectID);
    console.timeEnd("load");
    afterProjectLoaded(projectID);
  } catch (err) {
    window.alert(`Couldn't open project: ${err}`);
    currentProjectID.set(undefined);
  }
}

// Returns OSM input, optional demand model input, and the boundary polygon,
// either from pre-hosted files or from Overpass.
async function getInputFiles(project: ProjectFeatureCollection): Promise<{
  osmBuffer: ArrayBuffer;
  boundary: Feature<Polygon | MultiPolygon>;
  demandBuffer?: ArrayBuffer;
  contextDataBuffer?: ArrayBuffer;
}> {
  if (project.app_focus == "cnt") {
    // CNT projects are stored in a different place
    console.assert!(
      project.study_area_name,
      "CNT projects must have a study area name",
    );
    let url1 = assetUrl(`cnt_osm/${project.study_area_name}.osm.pbf`);
    console.log(`Grabbing ${url1}`);
    let resp1 = await safeFetch(url1);
    let osmBuffer = await resp1.arrayBuffer();

    let url2 = assetUrl(`cnt_boundaries/${project.study_area_name}.geojson`);
    let resp2 = await safeFetch(url2);
    let boundary = await resp2.json();

    let url3 = assetUrl(`cnt_demand/demand_${project.study_area_name}.bin`);
    console.log(`Grabbing ${url3}`);
    let demandBuffer = undefined;
    try {
      let resp3 = await safeFetch(url3);
      demandBuffer = await resp3.arrayBuffer();
    } catch (err) {
      console.log(`No demand model: ${err}`);
    }

    let url4 = assetUrl(
      `cnt_prioritization/context_${project.study_area_name}.bin`,
    );
    console.log(`Grabbing ${url4}`);
    let contextDataBuffer = undefined;
    try {
      let resp = await safeFetch(url4);
      contextDataBuffer = await resp.arrayBuffer();
    } catch (err) {
      console.log(`No context data for prioritization: ${err}`);
    }

    return { osmBuffer, boundary, demandBuffer, contextDataBuffer };
  } else if (project.study_area_name) {
    let url1 = assetUrl(`severance_pbfs/${project.study_area_name}.pbf`);
    console.log(`Grabbing ${url1}`);
    let resp1 = await safeFetch(url1);
    let osmBuffer = await resp1.arrayBuffer();

    let url2 = assetUrl(`boundaries/${project.study_area_name}.geojson`);
    let resp2 = await safeFetch(url2);
    let boundary = await resp2.json();

    return { osmBuffer, boundary };
  } else {
    console.log(`Grabbing from Overpass`);
    let boundary = project.features.find(
      (f: Feature) => f.properties!.kind == "study_area_boundary",
    ) as Feature<Polygon | MultiPolygon>;
    let resp = await safeFetch(overpassQueryForPolygon(boundary));
    let osmBuffer = await resp.arrayBuffer();
    return { osmBuffer, boundary };
  }
}

function afterProjectLoaded(projectID: ProjectID) {
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
