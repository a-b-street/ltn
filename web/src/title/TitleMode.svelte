<script lang="ts">
  import { LngLat } from "maplibre-gl";
  import { LTN } from "backend";
  import type { Feature } from "geojson";
  import { Link, overpassQueryForPolygon } from "../common";
  import { RouteTool } from "../common/snapper/route_tool";
  import SplitComponent from "../SplitComponent.svelte";
  import {
    app,
    projectName,
    map,
    showAbout,
    mode,
    useLocalVite,
    route_tool,
    route_pt_a,
    route_pt_b,
  } from "../stores";
  import About from "./About.svelte";

  export let wasmReady: boolean;

  // When other modes reset here, they can't clear state without a race condition
  $app = null;
  $route_tool = null;
  $projectName = "";

  let projectList = getProjectList();

  function getProjectList(): string[] {
    let list = [];
    for (let i = 0; i < window.localStorage.length; i++) {
      let key = window.localStorage.key(i)!;
      if (key.startsWith("ltn_")) {
        list.push(key);
      }
    }
    list.sort();
    return list;
  }

  let fileInput: HTMLInputElement;
  async function loadFile(e: Event) {
    // TODO Be careful with overwriting stuff, leading ltn_, etc
    let key = "ltn_" + fileInput.files![0].name;
    window.localStorage.setItem(key, await fileInput.files![0].text());
    projectList = getProjectList();
    await loadFromLocalStorage(key);
  }

  async function loadFromLocalStorage(key: string) {
    $projectName = key;
    try {
      let gj = JSON.parse(window.localStorage.getItem($projectName)!);

      let buffer = await getOsmInput(gj);
      //msg = "Building map model from OSM input";
      console.time("load");
      $app = new LTN(
        new Uint8Array(buffer),
        gj.study_area_boundary || undefined,
      );
      // TODO rename as project?
      // TODO or actually, combo this with the constructor.
      $app.loadSavefile(gj);
      console.timeEnd("load");

      afterProjectLoaded();
    } catch (err) {
      window.alert(`Couldn't open project: ${err}`);
      $projectName = "";
    }
  }

  // Either from a pre-hosted pbf file or from Overpass
  async function getOsmInput(gj: any): Promise<ArrayBuffer> {
    if (gj.study_area_name) {
      let url = $useLocalVite
        ? `/osm/${gj.study_area_name}.pbf`
        : `https://assets.od2net.org/severance_pbfs/${gj.study_area_name}.pbf`;
      //msg = `Downloading ${url}`;
      let resp = await fetch(url);
      let bytes = await resp.arrayBuffer();
      return bytes;
    } else {
      let study_area_boundary = gj.features.find(
        (f: Feature) => f.properties!.kind == "study_area_boundary",
      )!;
      let resp = await fetch(overpassQueryForPolygon(study_area_boundary));
      let bytes = await resp.arrayBuffer();
      return bytes;
    }
  }

  function afterProjectLoaded() {
    $mode = {
      mode: "network",
    };
    $route_tool = new RouteTool($map!, $app!.toRouteSnapper());
    $map!.fitBounds(
      Array.from($app!.getBounds()) as [number, number, number, number],
      { animate: false },
    );
    $route_pt_a = randomPoint();
    $route_pt_b = randomPoint();
  }

  function randomPoint(): LngLat {
    let bounds = $app!.getBounds();
    let lng = bounds[0] + Math.random() * (bounds[2] - bounds[0]);
    let lat = bounds[1] + Math.random() * (bounds[3] - bounds[1]);
    return new LngLat(lng, lat);
  }
</script>

<SplitComponent>
  <div slot="top">
    <nav aria-label="breadcrumb">
      <ul>
        <li>Choose project</li>
      </ul>
    </nav>
  </div>
  <div slot="sidebar">
    <About />
    <button on:click={() => ($showAbout = true)}>About the LTN tool</button>

    {#if $map && wasmReady}
      <div>
        <Link on:click={() => ($mode = { mode: "new-project" })}>
          New project
        </Link>
      </div>

      <p>Load a saved project:</p>
      <ul>
        {#each projectList as project}
          <li>
            <Link on:click={() => loadFromLocalStorage(project)}>
              {project}
            </Link>
          </li>
        {/each}
      </ul>

      <label>
        Load a project from a file
        <input bind:this={fileInput} on:change={loadFile} type="file" />
      </label>
    {:else}
      <p>Waiting for MapLibre and WASM to load...</p>
    {/if}
  </div>
</SplitComponent>
