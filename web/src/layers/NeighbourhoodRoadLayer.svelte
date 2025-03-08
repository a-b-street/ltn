<script lang="ts">
  // TODO: This should be called "EditableRoadLayer" or something, because it optionally includes the Perimeter (requires changes to backend as well)
  import type { Feature } from "geojson";
  import type { ExpressionSpecification, LngLat } from "maplibre-gl";
  import { getContext } from "svelte";
  import { hoverStateFilter, LineLayer } from "svelte-maplibre";
  import { makeRamp } from "svelte-utils/map";
  import { layerId, roadLineWidth } from "../common";
  import { speedColorScale, speedLimits, Style } from "../common/colors";
  import { roadStyle, thickRoadsForShortcuts } from "../stores";
  import type { RenderNeighbourhoodOutput } from "../wasm";

  let gj: RenderNeighbourhoodOutput = getContext("neighbourhoodGj");

  // When disabled, can't click lines or filters, no slots, no hoverCursor
  export let interactive = true;
  export let onClickLine = (f: Feature, pt: LngLat) => {};

  function roadLineColor(
    style: "shortcuts" | "cells" | "edits" | "speeds",
    maxShortcuts: number,
  ): ExpressionSpecification {
    if (style == "cells") {
      return ["get", "color"];
    }
    if (style == "edits") {
      return ["case", ["get", "edited"], "grey", "white"];
    }
    if (style == "speeds") {
      return makeRamp(
        ["get", "speed_mph"],
        speedLimits,
        speedColorScale,
      ) as ExpressionSpecification;
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
      Style.mapFeature.hover.backgroundColor,
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

<LineLayer
  {...layerId("interior-roads-outlines")}
  filter={["==", ["get", "kind"], "interior_road"]}
  paint={{
    // REVIEW: the interior-roads-outline was previously not visible - was that intentional?
    "line-width": lineWidth($thickRoadsForShortcuts, gj.maxShortcuts, 1),
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
  layout={{
    "line-sort-key": ["get", "shortcuts"],
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

<LineLayer
  {...layerId("primary-roads-outlines")}
  filter={["==", ["get", "kind"], "primary_road"]}
  paint={{
    "line-width": lineWidth($thickRoadsForShortcuts, gj.maxShortcuts, 6),
    "line-color": "black",
  }}
/>

<LineLayer
  {...layerId("primary-roads")}
  filter={["==", ["get", "kind"], "primary_road"]}
  paint={{
    "line-width": lineWidth($thickRoadsForShortcuts, gj.maxShortcuts, 4),
    "line-color": hoverStateFilter("white", "blue"),
    "line-opacity": hoverStateFilter(1.0, 0.5),
  }}
  layout={{
    "line-sort-key": ["get", "shortcuts"],
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
