<script lang="ts">
  // TODO: This should be called "EditableRoadLayer" or something, because it optionally includes the Perimeter (requires changes to backend as well)
  import type { Feature } from "geojson";
  import type {
    DataDrivenPropertyValueSpecification,
    ExpressionSpecification,
    LngLat,
  } from "maplibre-gl";
  import { type Snippet } from "svelte";
  import { hoverStateFilter, LineLayer } from "svelte-maplibre";
  import { makeRamp } from "svelte-utils/map";
  import { colorByCellColor } from ".";
  import { layerId, roadLineWidth } from "../common";
  import {
    signGreen,
    speedColorScale,
    speedLimitsKMPH,
    speedLimitsMPH,
    Style,
  } from "../common/colors";
  import { roadStyle, thickRoadsForShortcuts, useMetricUnits } from "../stores";

  interface Props {
    maxShortcuts: number;
    // When disabled, can't click lines or filters, no linePopup, no hoverCursor
    interactive?: boolean;
    linePopup?: Snippet | undefined;
    onClickLine?: (f: Feature, pt: LngLat) => void;
    show?: boolean;
    prefix?: string;
  }

  let {
    maxShortcuts,
    interactive = true,
    linePopup = undefined,
    onClickLine = (f: Feature, pt: LngLat) => {},
    show = true,
    prefix = "",
  }: Props = $props();

  function interiorRoadLineColor(
    style: "shortcuts" | "cells" | "edits" | "speeds",
    maxShortcuts: number,
  ): DataDrivenPropertyValueSpecification<string> {
    if (style == "cells") {
      return colorByCellColor();
    }
    if (style == "edits") {
      return hoverStateFilter(
        // @ts-expect-error hoverStateFilter is not properly typed - it should accept an expression
        ["case", ["get", "edited"], signGreen, "white"],
        Style.mapFeature.hover.backgroundColor,
      );
    }
    if (style == "speeds") {
      if ($useMetricUnits) {
        return makeRamp(
          ["*", 1.60934, ["get", "speed_mph"]],
          speedLimitsKMPH,
          speedColorScale,
        ) as ExpressionSpecification;
      }
      return makeRamp(
        ["get", "speed_mph"],
        speedLimitsMPH,
        speedColorScale,
      ) as ExpressionSpecification;
    }

    if (maxShortcuts <= 2) {
      return hoverStateFilter("white", "blue");
    }

    console.assert(style == "shortcuts");
    return hoverStateFilter(
      // @ts-expect-error hoverStateFilter is not properly typed - it should accept an expression
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

  function mainRoadLineColor(
    style: "shortcuts" | "cells" | "edits" | "speeds",
  ): DataDrivenPropertyValueSpecification<string> {
    if (style == "edits") {
      return hoverStateFilter(
        // @ts-expect-error hoverStateFilter is not properly typed - it should accept an expression
        ["case", ["get", "edited"], signGreen, "gray"],
        Style.mapFeature.hover.backgroundColor,
      );
    }

    return hoverStateFilter("gray", Style.mapFeature.hover.backgroundColor);
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
  {...layerId(prefix + "interior-roads-outlines")}
  filter={["==", ["get", "kind"], "interior_road"]}
  paint={{
    "line-width": lineWidth($thickRoadsForShortcuts, maxShortcuts, 1),
    "line-color": "black",
  }}
  layout={{
    visibility: show ? "visible" : "none",
  }}
  minzoom={13}
/>

<LineLayer
  {...layerId(prefix + "interior-roads")}
  filter={["==", ["get", "kind"], "interior_road"]}
  paint={{
    "line-width": lineWidth($thickRoadsForShortcuts, maxShortcuts, 0),
    "line-color": interiorRoadLineColor($roadStyle, maxShortcuts),
    "line-opacity": hoverStateFilter(1.0, 0.5),
  }}
  layout={{
    "line-sort-key": ["get", "shortcuts"],
    visibility: show ? "visible" : "none",
  }}
  minzoom={13}
  onclick={(e) => interactive && onClickLine(e.features[0], e.event.lngLat)}
  manageHoverState={interactive}
  hoverCursor={interactive ? "pointer" : undefined}
>
  {#if interactive}
    {@render linePopup?.()}
  {/if}
</LineLayer>

<LineLayer
  {...layerId(prefix + "main-roads-outlines")}
  filter={["==", ["get", "kind"], "main_road"]}
  paint={{
    "line-width": lineWidth($thickRoadsForShortcuts, maxShortcuts, 6),
    "line-color": "black",
  }}
  layout={{
    visibility: show ? "visible" : "none",
  }}
  minzoom={13}
/>

<LineLayer
  {...layerId(prefix + "main-roads")}
  filter={["==", ["get", "kind"], "main_road"]}
  paint={{
    "line-width": lineWidth($thickRoadsForShortcuts, maxShortcuts, 4),
    "line-color": mainRoadLineColor($roadStyle),
    "line-opacity": hoverStateFilter(1.0, 0.5),
  }}
  layout={{
    "line-sort-key": ["get", "shortcuts"],
    visibility: show ? "visible" : "none",
  }}
  minzoom={13}
  onclick={(e) => interactive && onClickLine(e.features[0], e.event.lngLat)}
  manageHoverState={interactive}
  hoverCursor={interactive ? "pointer" : undefined}
>
  {#if interactive}
    {@render linePopup?.()}
  {/if}
</LineLayer>
