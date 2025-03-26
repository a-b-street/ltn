<script lang="ts">
  import { type DataDrivenPropertyValueSpecification } from "maplibre-gl";
  import { FillLayer, SymbolLayer } from "svelte-maplibre";
  import { layerId } from "../common";
  import { roadStyle } from "../stores";

  function colorIconByCell(): DataDrivenPropertyValueSpecification<string> {
    return [
      "match",
      ["get", "cell_color"],
      "disconnected",
      "red",
      // For numeric values, need to use a step function with % operation
      [
        "let",
        "index",
        ["%", ["to-number", ["get", "cell_color"]], 10],
        [
          "match",
          ["var", "index"],
          0,
          "#8dd3c7",
          1,
          "#ffffb3",
          2,
          "#bebada",
          3,
          "#80b1d3",
          4,
          "#fdb462",
          5,
          "#b3de69",
          6,
          "#fccde5",
          7,
          "#bc80bd",
          8,
          "#ccebc5",
          9,
          "#ffed6f",
          "blue", // default color if something goes wrong
        ],
      ],
    ];
  }

  function borderEntryIconSize(
    scale: number,
  ): DataDrivenPropertyValueSpecification<number> {
    return [
      "interpolate",
      ["linear"],
      ["zoom"],
      14,
      0.08 * scale,
      22,
      0.7 * scale,
    ];
  }

  function borderEntryIconOffset(
    scale: number,
  ): DataDrivenPropertyValueSpecification<[number, number]> {
    return [0, -50 * scale];
  }
  function borderEntryIconOpacity(): DataDrivenPropertyValueSpecification<number> {
    return ["interpolate", ["linear"], ["zoom"], 14, 1.0, 22, 0.7];
  }
</script>

<FillLayer
  {...layerId("cells")}
  filter={["==", ["get", "kind"], "cell"]}
  layout={{
    visibility: $roadStyle == "cells" ? "none" : "visible",
  }}
  paint={{
    "fill-color": ["get", "color"],
    "fill-opacity": 0.5,
    "fill-outline-color": "hsla(0, 0%, 0%, 0.3)",
  }}
/>

<SymbolLayer
  {...layerId("border-entries")}
  filter={["==", ["get", "kind"], "border_entry"]}
  layout={{
    "icon-image": "border_entry_arrow",
    "icon-rotate": ["get", "bearing_upon_entry"],
    "icon-allow-overlap": true,
    "icon-size": borderEntryIconSize(1.0),
    "icon-offset": borderEntryIconOffset(1.0),
    // Without some explicit sort order, the icon-halo (icon stroke) of overlapping icons is
    // rendered as if the icons had been unioned, which isn't desirable.
    // Using bearing is arbitrary, but it's already available and seems to work in practice.
    // Any number which is unique between overlapping icons should suffice.
    "symbol-sort-key": ["get", "bearing_upon_entry"],
  }}
  paint={{
    "icon-color": colorIconByCell(),
    "icon-halo-color": "black",
    "icon-opacity": borderEntryIconOpacity(),
    // We can add a "stroke" to our icon with halo-width.
    //
    // Ideally, we'd have a little thicker stroke, but I haven't figured it out.
    //
    // If you increase this value much, soon you'll jump from a 1px stroke to
    // stroking the entire background. It's more obvious when you're zoomed out.
    //
    // I am not familiar with SDF, but my theory is that this is related to how
    // our icon was transformed into SDF - maybe there are some different
    // filters/bluring we need to apply in order for this to behave how I'm
    // expecting.
    "icon-halo-width": borderEntryIconSize(1.5),
    "icon-halo-blur": 0.0,
  }}
  minzoom={13}
  interactive={false}
/>
