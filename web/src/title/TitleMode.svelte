<script lang="ts">
  import type { Feature } from "geojson";
  import { overpassQueryForPolygon } from "../common";
  import PolygonToolLayer from "../common/draw_polygon/PolygonToolLayer.svelte";
  import SplitComponent from "../SplitComponent.svelte";
  import { example, map } from "../stores";
  import About from "./About.svelte";
  import MapLoader from "./MapLoader.svelte";

  export let wasmReady: boolean;

  // TODO Once per session
  let showModal = true;

  let mapLoader: MapLoader | undefined;

  let fileInput: HTMLInputElement;
  async function loadFile(e: Event) {
    try {
      let gj = JSON.parse(await fileInput.files![0].text());

      if (gj.study_area_name) {
        $example = gj.study_area_name;
        // TODO HACK! MapLoader will restore from local storage, so just set that
        window.localStorage.setItem(
          `ltn_${gj.study_area_name}.geojson`,
          JSON.stringify(gj)
        );
        await mapLoader!.loadExample(gj.study_area_name);
      } else {
        $example = "";
        let study_area_boundary = gj.features.find(
          (f: Feature) => f.properties!.kind == "study_area_boundary"
        )!;
        let resp = await fetch(overpassQueryForPolygon(study_area_boundary));
        let bytes = await resp.arrayBuffer();
        // TODO HACK! MapLoader will restore from local storage, so just set that
        window.localStorage.setItem("ltn_custom.geojson", JSON.stringify(gj));
        mapLoader!.loadMap(bytes);
      }
    } catch (err) {
      window.alert(`Couldn't open this file: ${err}`);
    }
  }
</script>

<SplitComponent>
  <div slot="sidebar">
    <About bind:showModal />

    <h1>Choose your study area</h1>
    <button on:click={() => (showModal = true)}>About the LTN tool</button>

    {#if mapLoader}
      <div>
        <label>
          Load a project from a file
          <input bind:this={fileInput} on:change={loadFile} type="file" />
        </label>
      </div>
    {/if}

    <hr />

    {#if $map && wasmReady}
      <MapLoader bind:this={mapLoader} />
    {:else}
      <p>Waiting for MapLibre and WASM to load...</p>
    {/if}
  </div>

  <div slot="map">
    <PolygonToolLayer />
  </div>
</SplitComponent>
