<script lang="ts">
  import { Eye, EyeClosed } from "lucide-svelte";
  import type { Map, MapMouseEvent } from "maplibre-gl";
  import { onDestroy } from "svelte";
  import { ControlButton } from "svelte-maplibre";
  import { getRoadLayerNames } from "./highlight_roads";
  import { interactiveMapLayersEnabled } from "./stores";

  // TODO Need to intercept the escape key always

  interface Props {
    // TODO Make sure all layers respect DisableInteractiveLayers (they currently do not)
    map: Map;
    maptilerBasemap: string;
  }

  let { map, maptilerBasemap }: Props = $props();
  let expanded = $state(false);

  let source: "google" | "bing" = $state("google");
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

<svelte:window onkeydown={onKeyDown} />

<ControlButton>
  <button
    onclick={() => {
      expanded = !expanded;
      expanded ? start() : stop();
    }}
    title="Street view"
  >
    <div class="ltn-map-btn">
      {#if expanded}
        <EyeClosed />
      {:else}
        <Eye />
      {/if}
    </div>
  </button>
  {#if expanded}
    <div class="street-view-control-source" class:expanded>
      Click the map to see street view
      <br />
      <br />
      <b>Street view source</b>
      <label>
        <input type="radio" value="google" bind:group={source} />
        Google Street View
      </label>
      <label>
        <input type="radio" value="bing" bind:group={source} />
        Bing Streetside
      </label>
    </div>
  {/if}
</ControlButton>

<style>
  .street-view-control-source.expanded {
    position: absolute;
    top: 34px;
    width: 200px;
    background: white;
    border-radius: 4px;
    padding: 10px;
    text-align: left;
    box-shadow: 0 0 10px rgba(0, 0, 0, 0.2);
    cursor: default;
  }
  .street-view-control-source label {
    display: block;
    text-align: left;
    margin-top: 4px;
    cursor: pointer;
  }
  .street-view-control-source input {
    cursor: pointer;
  }
</style>
