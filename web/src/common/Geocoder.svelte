<script lang="ts">
  import { createMapLibreGlMapController } from "@maptiler/geocoding-control/maplibregl";
  import GeocodingControl from "@maptiler/geocoding-control/svelte/GeocodingControl.svelte";
  import maplibregl from "maplibre-gl";
  import { map, maptilerApiKey } from "../stores";

  $: mapController = $map ? createMapLibreGlMapController($map, maplibregl) : null;

  // TODO Show markers
  // TODO Set the flyTo duration
</script>

{#if mapController}
  <div>
    <GeocodingControl {mapController} apiKey={maptilerApiKey} />
  </div>
{/if}

<style>
  div {
    position: absolute;
    top: 20px;
    left: 50px;
  }
</style>
