<script lang="ts">
  import type { RenderNeighbourhoodOutput } from "./wasm";
  import OneWayLayer from "./OneWayLayer.svelte";
  import type { Feature } from "geojson";
  import { FillLayer, GeoJSON, LineLayer } from "svelte-maplibre";
  import { setCellColors } from "./cells";
  import { layerId } from "./common";

  export let gjInput: RenderNeighbourhoodOutput;
  // When disabled, can't click lines or filters, no slots, no hoverCursor
  export let interactive = true;
  export let onClickLine = (f: Feature) => {};

  $: gj = setCellColors(gjInput);
  $: maxShortcuts = Math.max(
    ...gjInput.features.map((f) =>
      f.properties.kind == "interior_road" ? f.properties.shortcuts : 0,
    ),
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
      "line-width": 5,
      "line-color": [
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
    }}
    on:click={(e) => interactive && onClickLine(e.detail.features[0])}
    hoverCursor={interactive ? "pointer" : undefined}
  >
    {#if interactive}
      <slot name="line-popup" />
    {/if}
  </LineLayer>

  <OneWayLayer />

  <slot name="more-layers" />
</GeoJSON>
