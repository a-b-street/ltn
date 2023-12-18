<script lang="ts">
  import turfBbox from "@turf/bbox";
  import { MapModel } from "backend";
  import type { Map } from "maplibre-gl";
  import { MapLibre } from "svelte-maplibre";
  import { Layout } from "./common";
  import MapLoader from "./MapLoader.svelte";
  import NetworkLayer from "./NetworkLayer.svelte";

  let model: MapModel | undefined = undefined;
  let map: Map;

  function zoomToFit() {
    if (map && model) {
      // TODO wasteful
      let bbox = turfBbox(JSON.parse(model.render()));
      map.fitBounds(bbox, { animate: false });
    }
  }

  function gotModel(_m: MapModel) {
    if (!model) {
      return;
    }
    console.log("New map model loaded");
    zoomToFit();
  }
  $: gotModel(model);
</script>

<Layout>
  <div slot="left">
    {#if map}
      <MapLoader {map} bind:model />
    {/if}
    <div><button on:click={zoomToFit}>Zoom to fit</button></div>
  </div>
  <div slot="main" style="position:relative; width: 100%; height: 100vh;">
    <MapLibre
      style="https://api.maptiler.com/maps/dataviz/style.json?key=MZEJTanw3WpxRvt7qDfo"
      standardControls
      hash
      bind:map
    >
      {#if model}
        <NetworkLayer {model} />
      {/if}
    </MapLibre>
  </div>
</Layout>

<style>
  :global(body, button, input) {
    font-size: 26px;
  }
</style>
