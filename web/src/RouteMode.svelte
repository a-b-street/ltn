<script lang="ts">
  import type { FeatureCollection } from "geojson";
  import { onDestroy, onMount } from "svelte";
  import { GeoJSON, LineLayer } from "svelte-maplibre";
  import RenderNeighbourhood from "./RenderNeighbourhood.svelte";
  import SplitComponent from "./SplitComponent.svelte";
  import { app, map, mode } from "./stores";

  onMount(() => {
    $map?.keyboard.disable();
  });
  onDestroy(() => {
    $map?.keyboard.enable();
  });

  function onKeyDown(e: KeyboardEvent) {
    if (e.key == "Escape") {
      e.stopPropagation();
      back();
    }
  }

  function back() {
    $mode = { mode: "neighbourhood" };
  }
</script>

<svelte:window on:keydown={onKeyDown} />

<SplitComponent>
  <div slot="sidebar">
    <div><button on:click={back}>Back to editing</button></div>

    <p>Drag markers for a route</p>
  </div>

  <div slot="map">
    <RenderNeighbourhood
      gjInput={JSON.parse($app.renderNeighbourhood())}
      interactive={false}
    />
  </div>
</SplitComponent>
