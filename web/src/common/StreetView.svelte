<script lang="ts">
  import type { Map, MapMouseEvent } from "maplibre-gl";
  import { onDestroy } from "svelte";
  import { getRoadLayerNames } from "./highlight_roads";
  import { interactiveMapLayersEnabled } from "./stores";

  // TODO Need to intercept the escape key always
  // TODO Make sure all layers respect DisableInteractiveLayers

  export let map: Map;
  export let maptilerBasemap: string;

  let source: "google" | "bing" = "google";
  let defaultLineColorPerLayer: [string, any][] = [];

  function start() {
    $interactiveMapLayersEnabled = false;
    map.on("click", onClick);
    map.getCanvas().style.cursor = "crosshair";

    for (let layer of getRoadLayerNames(map, maptilerBasemap)) {
      defaultLineColorPerLayer.push([
        layer,
        map.getPaintProperty(layer, "line-color"),
      ]);
      map.setPaintProperty(layer, "line-color", "cyan");
    }
  }

  function stop() {
    $interactiveMapLayersEnabled = true;
    map.off("click", onClick);
    map.getCanvas().style.cursor = "inherit";

    for (let [layer, value] of defaultLineColorPerLayer) {
      map.setPaintProperty(layer, "line-color", value);
    }
    defaultLineColorPerLayer = [];
  }

  onDestroy(stop);

  function onClick(e: MapMouseEvent) {
    let lon = e.lngLat.lng;
    let lat = e.lngLat.lat;
    if (source == "google") {
      window.open(
        `http://maps.google.com/maps?q=&layer=c&cbll=${lat},${lon}&cbp=11,0,0,0,0`,
        "_blank",
      );
    } else if (source == "bing") {
      window.open(
        `https://www.bing.com/maps?cp=${lat}~${lon}&style=x`,
        "_blank",
      );
    }
  }

  function onKeyDown(e: KeyboardEvent) {
    if (!$interactiveMapLayersEnabled && e.key == "Escape") {
      e.stopPropagation();
      stop();
    }
  }
</script>

<svelte:window on:keydown={onKeyDown} />

{#if $interactiveMapLayersEnabled}
  <button class="secondary" on:click={start}>StreetView</button>
{:else}
  <button class="secondary" on:click={stop}>Stop StreetView</button>

  <fieldset>
    <legend>Source:</legend>
    <label>
      <input type="radio" value="google" bind:group={source} />
      Google Street View
    </label>
    <label>
      <input type="radio" value="bing" bind:group={source} />
      Bing Streetside
    </label>
  </fieldset>
{/if}
