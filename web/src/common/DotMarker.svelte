<script lang="ts">
  import type { LngLatLike } from "maplibre-gl";
  import { Marker } from "svelte-maplibre";

  export let lngLat: LngLatLike;
  export let draggable = false;
</script>

{#if draggable}
  <Marker bind:lngLat draggable>
    <span class="dot"><slot /></span>
  </Marker>
{:else}
  <Marker {lngLat}>
    <span class="dot"><slot /></span>
  </Marker>
{/if}

<style>
  .dot {
    width: 30px;
    height: 30px;
    border-radius: 50%;
    display: flex;
    justify-content: center;
    align-items: center;

    background-color: grey;
    font-weight: bold;
  }
</style>
