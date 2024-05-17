<script lang="ts">
  import { LTN } from "backend";
  import { onMount } from "svelte";
  import { Loading } from "svelte-utils";
  import { Link } from "../common";
  import { OverpassSelector } from "svelte-utils/overpass";
  import { PolygonToolLayer } from "maplibre-draw-polygon";
  import SplitComponent from "../SplitComponent.svelte";
  import {
    autosave,
    projectName,
    app,
    map,
    useLocalVite,
    mode,
  } from "../stores";
  import { afterProjectLoaded, loadFromLocalStorage } from "./loader";

  let newProjectName = "";
  let example = "";
  let exampleAreas: [string, [string, string][]][] = [];
  let loading = "";

  onMount(async () => {
    let resp = await fetch(
      $useLocalVite
        ? "/osm/areas.json"
        : "https://assets.od2net.org/severance_pbfs/areas.json",
    );
    exampleAreas = await resp.json();
  });

  function gotXml(e: CustomEvent<string>) {
    loading = "Loading OSM";
    try {
      $app = new LTN(new TextEncoder().encode(e.detail), undefined);
      $projectName = `ltn_${newProjectName}`;
      afterProjectLoaded();
      // No savefile to load. Create it immediately with just the boundary
      autosave();
    } catch (err) {
      window.alert(`Couldn't import from Overpass: ${err}`);
    }
    loading = "";
  }

  export async function loadExample() {
    if (example == "") {
      return;
    }

    let key = `ltn_${newProjectName}`;
    window.localStorage.setItem(
      key,
      JSON.stringify({
        type: "FeatureCollection",
        features: [],
        study_area_name: example,
      }),
    );
    loading = `Loading pre-clipped OSM area ${example}`;
    await loadFromLocalStorage(key);
    loading = "";
  }
</script>

<SplitComponent>
  <div slot="top">
    <nav aria-label="breadcrumb">
      <ul>
        <li>
          <Link on:click={() => ($mode = { mode: "title" })}>
            Choose project
          </Link>
        </li>
        <li>New project</li>
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
