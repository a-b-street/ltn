import type { Feature, MultiPolygon, Polygon } from "geojson";
import { RouteTool } from "route-snapper-ts";
import { fetchWithProgress } from "svelte-utils";
import { emptyGeojson } from "svelte-utils/map";
import { overpassQueryForPolygon } from "svelte-utils/overpass";
import { get, writable, type Writable } from "svelte/store";
import { refreshLoadingScreen, safeFetch } from "../common";
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
  metricBuckets,
  projectStorage,
} from "../stores";
import { Backend } from "../wasm";

export let loadingMessage = writable("");
export let loadingProgress: Writable<number | null> = writable(null);

/**
 * Loads an existing project by its ID. Changes loadingMessage and
 * loadingProgress stores along the way.
 */
export async function loadProject(projectID: ProjectID) {
  let project = get(projectStorage).project(projectID);
  try {
    console.time("get input files");
    let {
      osmBuffer,
      demandBuffer,
      contextDataBuffer,
      boundary,
      mapModelBuffer,
    } = await getInputFiles(project);
    console.timeEnd("get input files");
    loadingMessage.set("Download finished, setting up project");
    // TODO The animation won't work, because we block the UI thread below
    loadingProgress.set(100);
    await refreshLoadingScreen();

    console.time("load");
    backend.set(
      new Backend(
        mapModelBuffer ? new Uint8Array(mapModelBuffer) : undefined,
        osmBuffer ? new Uint8Array(osmBuffer) : undefined,
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

  loadingMessage.set("");
  loadingProgress.set(null);
}

// Returns input needed to set up the LTN backend, either from pre-hosted files
// or from Overpass.
async function getInputFiles(project: ProjectFeatureCollection): Promise<{
  osmBuffer?: Uint8Array<ArrayBufferLike>;
  boundary?: Feature<Polygon | MultiPolygon>;
  demandBuffer?: Uint8Array<ArrayBufferLike>;
  contextDataBuffer?: Uint8Array<ArrayBufferLike>;

  // This one bundled version replaces all the others
  mapModelBuffer?: Uint8Array<ArrayBufferLike>;
}> {
  if (project.app_focus == "cnt") {
    // CNT projects are stored in a different place
    console.assert!(
      project.study_area_name,
      "CNT projects must have a study area name",
    );

    let osmBuffer = await download(
      assetUrl(`cnt/osm/${project.study_area_name}.osm.pbf.gz`),
    );

    let url2 = assetUrl(`cnt/boundaries/${project.study_area_name}.geojson`);
    let resp2 = await safeFetch(url2);
    let boundary = await resp2.json();

    let demandBuffer = undefined;
    try {
      demandBuffer = await download(
        assetUrl(`cnt/demand/${project.study_area_name}.bin.gz`),
      );
    } catch (err) {
      console.log(`No demand model: ${err}`);
    }

    let contextDataBuffer = undefined;
    try {
      contextDataBuffer = await download(
        assetUrl(`cnt/prioritization/${project.study_area_name}.bin.gz`),
      );
    } catch (err) {
      console.log(`No context data for prioritization: ${err}`);
    }

    return { osmBuffer, boundary, demandBuffer, contextDataBuffer };
  } else if (project.app_focus == "england") {
    let mapModelBuffer = await download(
      assetUrl(`england/maps_v1/${project.study_area_name}.bin.gz`),
    );
    return { mapModelBuffer };
  } else if (project.study_area_name) {
    let osmBuffer = await download(
      assetUrl(`severance_pbfs/${project.study_area_name}.pbf`),
    );

    let url2 = assetUrl(`boundaries/${project.study_area_name}.geojson`);
    let resp2 = await safeFetch(url2);
    let boundary = await resp2.json();

    return { osmBuffer, boundary };
  } else {
    loadingMessage.set("Grabbing OSM data from Overpass");
    loadingProgress.set(100);
    let boundary = project.features.find(
      (f: Feature) => f.properties!.kind == "study_area_boundary",
    ) as Feature<Polygon | MultiPolygon>;
    let resp = await safeFetch(overpassQueryForPolygon(boundary));
    // @ts-expect-error TODO The return types are probably wrong
    let osmBuffer = (await resp.arrayBuffer()) as Uint8Array<ArrayBufferLike>;
    return { osmBuffer, boundary };
  }
}

async function download(url: string): Promise<Uint8Array> {
  console.log(`Grabbing ${url}`);
  loadingMessage.set(`Downloading ${url}`);
  return await fetchWithProgress(url, (p) => loadingProgress.set(p));
}

export function afterProjectLoaded(projectID: ProjectID) {
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

  // If there are no buckets defined for this project, then leave at the default value
  let buckets = get(backend)!.getMetricBuckets();
  if (buckets) {
    metricBuckets.set(buckets);
  }

  // Update the URL
  let url = new URL(window.location.href);
  url.searchParams.set("project", projectID);
  window.history.replaceState(null, "", url.toString());
}
