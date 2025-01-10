<script lang="ts">
  import type { Feature } from "geojson";
  import type { ExpressionSpecification, LngLat } from "maplibre-gl";
  import { GeoJSON, hoverStateFilter, LineLayer } from "svelte-maplibre";
  import { layerId, roadLineWidth } from "./common";
  import { roadStyle, thickRoadsForShortcuts } from "./stores";
  import type { RenderNeighbourhoodOutput } from "./wasm";

  export let gj: RenderNeighbourhoodOutput;
  // When disabled, can't click lines or filters, no slots, no hoverCursor
  export let interactive = true;
  export let onClickLine = (f: Feature, pt: LngLat) => {};

  function roadLineColor(
    style: "shortcuts" | "cells",
    maxShortcuts: number,
  ): ExpressionSpecification {
    if (style == "cells") {
      return ["get", "color"];
    }
    if (maxShortcuts <= 2) {
      return hoverStateFilter("white", "blue");
    }

    return hoverStateFilter(
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
  }

  function lineWidth(
    thickRoadsForShortcuts: boolean,
    maxShortcuts: number,
    extraWidth: number,
  ): ExpressionSpecification {
    if (!thickRoadsForShortcuts || maxShortcuts <= 2) {
      return roadLineWidth(extraWidth);
    }
    // TODO It'd still be nice to depend on zoom here
    return [
      "interpolate",
      ["linear"],
      ["get", "shortcuts"],
      0,
      5 + extraWidth,
      maxShortcuts,
      25 + extraWidth,
    ];
  }
</script>

<GeoJSON data={gj} generateId>
  <LineLayer
    {...layerId("interior-roads-outlines")}
    filter={["==", ["get", "kind"], "interior_road"]}
    paint={{
      "line-width": lineWidth($thickRoadsForShortcuts, gj.maxShortcuts, 0),
      "line-color": "black",
    }}
  />

  <LineLayer
    {...layerId("interior-roads")}
    filter={["==", ["get", "kind"], "interior_road"]}
    paint={{
      "line-width": lineWidth($thickRoadsForShortcuts, gj.maxShortcuts, 0),
      "line-color": roadLineColor($roadStyle, gj.maxShortcuts),
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
</GeoJSON>
