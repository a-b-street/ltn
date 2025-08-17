<script lang="ts">
  import type { LngLatLike } from "maplibre-gl";
  import type { Snippet } from "svelte";
  import { Marker } from "svelte-maplibre";

  export let lngLat: LngLatLike;
  export let draggable = false;
  export let children: Snippet | undefined = undefined;
</script>

{#if draggable}
  <Marker bind:lngLat draggable>
    <span class="dot">{@render children?.()}</span>
  </Marker>
{:else}
  <Marker {lngLat}>
    <span class="dot">{@render children?.()}</span>
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

    background-color: orange;
    font-weight: bold;
  }
</style>
