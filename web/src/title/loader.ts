import type { Feature, Polygon } from "geojson";
import { RouteTool } from "route-snapper-ts";
import { emptyGeojson } from "svelte-utils/map";
import { overpassQueryForPolygon } from "svelte-utils/overpass";
import { get, writable } from "svelte/store";
import { safeFetch } from "../common";
import { routeTool } from "../common/draw_area/stores";
import {
  appFocus,
  assetUrl,
  backend,
  currentProjectKey,
  map,
  mode,
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

  createEmptyProject(key, studyAreaName);
  await loadProject(key);
  return true;
}

export async function loadProject(key: string) {
  currentProjectKey.set(key);
  let isCnt = key.startsWith("ltn_cnt/");
  // TODO Should we also change the URL?
  appFocus.set(isCnt ? "cnt" : "global");

  try {
    let gj = JSON.parse(window.localStorage.getItem(key)!);

    console.time("get input files");
    let { osmBuffer, demandBuffer, contextDataBuffer, boundary } =
      await getInputFiles(gj, isCnt);
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
    currentProjectKey.set("");
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

// Returns a list, grouped and sorted by the optional study_area_name, with
// custom cases at the end
export function listProjects(
  appFocus: "cnt" | "global",
): Array<[string | undefined, { projectId: string; projectName: string }[]]> {
  let studyAreas = new Map();
  let custom = [];
  for (let i = 0; i < window.localStorage.length; i++) {
    let key = window.localStorage.key(i)!;
    if (key.startsWith("ltn_cnt/")) {
      if (appFocus != "cnt") {
        continue;
      }
      try {
        let [_, studyAreaId, projectName] = key.split("/");
        let studyAreaName = studyAreaId.split("LAD_")[1];
        if (!studyAreas.has(studyAreaName)) {
          studyAreas.set(studyAreaName, []);
        }
        studyAreas.get(studyAreaName)!.push({ projectId: key, projectName });
      } catch (err) {
        console.log(`error loading cnt project: ${key}`, err);
      }
    } else if (key.startsWith("ltn_")) {
      if (appFocus != "global") {
        continue;
      }
      let projectName = key.split("ltn_")[1];
      let studyAreaName = "";
      try {
        let gj = JSON.parse(window.localStorage.getItem(key)!);
        studyAreaName = gj.study_area_name;
      } catch (err) {
        console.log(`error loading cnt project: ${key}`, err);
      }
      if (studyAreaName && studyAreaName.length > 0) {
        if (!studyAreas.has(studyAreaName)) {
          studyAreas.set(studyAreaName, []);
        }
        studyAreas.get(studyAreaName)!.push({ projectId: key, projectName });
      } else {
        custom.push({ projectId: key, projectName });
      }
    }
  }

  let out = [...studyAreas.entries()];
  out.sort((a, b) => a[0].localeCompare(b[0]));
  if (custom.length > 0) {
    out.push([undefined, custom]);
  }
  return out;
}

export function createEmptyProject(key: string, studyAreaName: string) {
  if (!key) {
    throw new Error("Cannot create project: no key specified");
  }

  const projectData = {
    type: "FeatureCollection",
    features: [],
    study_area_name: studyAreaName,
  };

  window.localStorage.setItem(key, JSON.stringify(projectData));
}

export function saveProject(key: string, projectData: string) {
  if (!key) {
    throw new Error("Cannot save project: no key specified");
  }

  window.localStorage.setItem(key, projectData);
}

export function removeProject(key: string) {
  if (!key) {
    throw new Error("Cannot remove project: no key specified");
  }

  if (window.localStorage.getItem(key) == null) {
    console.warn(`Trying to remove a project that doesn't exist: ${key}`);
  }

  window.localStorage.removeItem(key);
}

export function renameProject(oldKey: string, newKey: string) {
  if (!oldKey || !newKey) {
    throw new Error("Cannot rename project: keys must be specified");
  }

  const projectData = window.localStorage.getItem(oldKey);
  if (!projectData) {
    throw new Error(`Project ${oldKey} not found`);
  }

  saveProject(newKey, projectData);
  removeProject(oldKey);
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
  get(map)!.fitBounds(get(backend)!.getBounds(), { duration: 500 });

  // Update the URL
  let url = new URL(window.location.href);
  url.searchParams.set("project", get(currentProjectKey));
  window.history.replaceState(null, "", url.toString());
}
