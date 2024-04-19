<script lang="ts">
  import type { RenderNeighbourhoodOutput } from "./wasm";
  import OneWayLayer from "./OneWayLayer.svelte";
  import type { Feature } from "geojson";
  import {
    hoverStateFilter,
    FillLayer,
    GeoJSON,
    LineLayer,
  } from "svelte-maplibre";
  import { setCellColors } from "./cells";
  import type { LngLat } from "maplibre-gl";
  import { layerId } from "./common";

  export let gjInput: RenderNeighbourhoodOutput;
  // When disabled, can't click lines or filters, no slots, no hoverCursor
  export let interactive = true;
  export let onClickLine = (f: Feature, pt: LngLat) => {};

  $: gj = setCellColors(gjInput);
  $: maxShortcuts = Math.max(
    ...gjInput.features.map((f) =>
      f.properties.kind == "interior_road" ? f.properties.shortcuts : 0,
    ),
  );

  $: lineColor = hoverStateFilter(
    // @ts-expect-error TODO Fix upstream types
    [
      "interpolate-hcl",
      ["linear"],
      ["get", "shortcuts"],
      0,
      "white",
      1,
      "#F19A93",
      maxShortcuts,
      "#A32015",
    ],
    "blue",
  );
</script>

<GeoJSON data={gj} generateId>
  <FillLayer
    {...layerId("cells")}
    filter={["==", ["get", "kind"], "cell"]}
    paint={{
      "fill-color": ["get", "color"],
      "fill-opacity": 0.3,
    }}
  />

  <LineLayer
    {...layerId("interior-roads")}
    filter={["==", ["get", "kind"], "interior_road"]}
    paint={{
      "line-width": 10,
      "line-color": lineColor,
      "line-opacity": hoverStateFilter(1.0, 0.5),
    }}
    on:click={(e) =>
      interactive && onClickLine(e.detail.features[0], e.detail.event.lngLat)}
    manageHoverState={interactive}
    hoverCursor={interactive ? "pointer" : undefined}
  >
    {#if interactive}
      <slot name="line-popup" />
    {/if}
  </LineLayer>

  <OneWayLayer />

  <slot name="more-layers" />
</GeoJSON>
