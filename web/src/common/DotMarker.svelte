<script lang="ts">
  import type { LngLatLike } from "maplibre-gl";
  import type { Snippet } from "svelte";
  import { Marker } from "svelte-maplibre";

  interface Props {
    lngLat: LngLatLike;
    draggable?: boolean;
    children: Snippet;
  }

  let { lngLat = $bindable(), draggable = false, children }: Props = $props();
</script>

{#if draggable}
  <Marker bind:lngLat draggable>
    <span class="dot">{@render children()}</span>
  </Marker>
{:else}
  <Marker {lngLat}>
    <span class="dot">{@render children()}</span>
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
