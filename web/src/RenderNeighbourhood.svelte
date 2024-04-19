<script lang="ts">
  import type { RenderNeighbourhoodOutput } from "./wasm";
  import OneWayLayer from "./OneWayLayer.svelte";
  import type { Feature, Polygon } from "geojson";
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

  function invertBoundary(gj: RenderNeighbourhoodOutput): Feature<Polygon> {
    // @ts-expect-error TS can't figure out that we're narrowing the case here
    let boundary: Feature<Polygon> = gj.features.find(
      (f) => f.properties.kind == "boundary",
    )!;

    return {
      type: "Feature",
      properties: {},
      geometry: {
        type: "Polygon",
        coordinates: [
          [
            [180.0, 90.0],
            [-180.0, 90.0],
            [-180.0, -90.0],
            [180.0, -90.0],
            [180.0, 90.0],
          ],
          // One hole
          boundary.geometry.coordinates[0],
        ],
      },
    };
  }
</script>

<GeoJSON data={gj} generateId>
  <GeoJSON data={invertBoundary(gj)}>
    <FillLayer
      {...layerId("neighbourhood-boundary")}
      paint={{ "fill-color": "black", "fill-opacity": 0.3 }}
    />
  </GeoJSON>

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
