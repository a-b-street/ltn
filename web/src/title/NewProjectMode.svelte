<script lang="ts">
  import { LTN } from "backend";
  import { onMount } from "svelte";
  import { Link, Loading, OverpassSelector } from "../common";
  import PolygonToolLayer from "../common/draw_polygon/PolygonToolLayer.svelte";
  import SplitComponent from "../SplitComponent.svelte";
  import { app, map, useLocalVite, projectName, mode } from "../stores";

  let example = "";
  let exampleAreas: [string, [string, string][]][] = [];
  let msg: string | null = null;

  onMount(async () => {
    let resp = await fetch(
      $useLocalVite
        ? "/osm/areas.json"
        : "https://assets.od2net.org/severance_pbfs/areas.json",
    );
    exampleAreas = await resp.json();
  });

  function gotXml(e: CustomEvent<string>) {
    try {
      // TODO Can we avoid turning into bytes?
      $app = new LTN(new TextEncoder().encode(e.detail), undefined);
      // No savefile to load
      // TODO call afterProjectLoaded
    } catch (err) {
      window.alert(`Couldn't import from Overpass: ${err}`);
    }
    msg = null;
  }

  export async function loadExample() {
    if (example == "") {
      return;
    }

    window.localStorage.setItem(
      $projectName,
      JSON.stringify({
        study_area_boundary: example,
      }),
    );
    // TODO Trigger actual loading
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
        <input type="text" bind:value={$projectName} />
      </label>
    </div>

    {#if $projectName}
      <Loading {msg} />

      <label>
        Load an example:
        <select bind:value={example} on:change={() => loadExample()}>
          <option value="">Custom file loaded</option>
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

      <OverpassSelector
        map={$map}
        on:gotXml={gotXml}
        on:loading={(e) => (msg = e.detail)}
        on:error={(e) => window.alert(e.detail)}
      />
    {/if}
  </div>

  <div slot="map">
    <PolygonToolLayer />
  </div>
</SplitComponent>
