<script lang="ts">
  import type { Feature, Polygon } from "geojson";
  import { PolygonToolLayer } from "maplibre-draw-polygon";
  import { onMount } from "svelte";
  import { OverpassSelector } from "svelte-utils/overpass";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { Loading, ModeLink, pageTitle, safeFetch } from "../common";
  import {
    appFocus,
    assetUrl,
    backend,
    map,
    mode,
    projectStorage,
  } from "../stores";
  import { Backend } from "../wasm";
  import { loadProject } from "./loader";

  let newProjectName = "";
  let example: string | null = null;
  let exampleAreas: [string, [string, string][]][] = [];
  let loading = "";

  onMount(async () => {
    let resp = await safeFetch(assetUrl("severance_pbfs/areas.json"));
    exampleAreas = await resp.json();
  });

  async function gotXml(
    e: CustomEvent<{ xml: string; boundary: Feature<Polygon> }>,
  ) {
    loading = "Loading OSM";
    try {
      let studyAreaName = undefined;
      $backend = new Backend(
        new TextEncoder().encode(e.detail.xml),
        undefined,
        undefined,
        e.detail.boundary,
        $appFocus,
        studyAreaName,
        newProjectName,
        $projectStorage.dbSchemaVersion,
      );

      const projectID = $projectStorage.createProject($backend.toSavefile());
      await loadProject(projectID);
      $mode = { mode: "add-neighbourhood" };
    } catch (err) {
      window.alert(`Couldn't import from Overpass: ${err}`);
    }
    loading = "";
  }

  export async function loadExample() {
    if (!example) {
      return;
    }

    let projectID;
    try {
      projectID = $projectStorage.createEmptyProject(newProjectName, example);
    } catch (err) {
      example = null;
      window.alert(`Couldn't create project: ${err}`);
      return;
    }
    loading = `Loading pre-clipped OSM area ${example}`;
    await loadProject(projectID);
    $mode = { mode: "add-neighbourhood" };
    loading = "";
  }
</script>

<SplitComponent>
  <div slot="top">
    <nav aria-label="breadcrumb">
      <ul>
        <li>
          <ModeLink mode={{ mode: "title" }} />
        </li>
        <li>{pageTitle($mode.mode)}</li>
      </ul>
    </nav>
  </div>

  <div slot="sidebar">
    <div>
      <label>
        Project name:
        <input type="text" bind:value={newProjectName} />
      </label>
    </div>

    {#if newProjectName}
      <Loading {loading} />

      <label>
        Load a built-in area:
        <select bind:value={example} on:change={() => loadExample()}>
          <option value=""></option>
          {#each exampleAreas as [country, areas]}
            <optgroup label={country}>
              {#each areas as [value, label]}
                <option {value}>{label}</option>
              {/each}
            </optgroup>
          {/each}
        </select>
      </label>

      <i>or...</i>

      <div>
        <OverpassSelector
          map={$map}
          on:gotXml={gotXml}
          on:loading={(e) => (loading = e.detail)}
          on:error={(e) => window.alert(e.detail)}
        />
      </div>
    {/if}
  </div>

  <div slot="map">
    <PolygonToolLayer />
  </div>
</SplitComponent>
