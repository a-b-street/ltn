<script lang="ts">
  import type { Feature, FeatureCollection } from "geojson";
  import { FillLayer, GeoJSON, LineLayer } from "svelte-maplibre";
  import { showBasemap } from "./stores";

  export let gjInput: FeatureCollection;
  // When disabled, can't click lines or circles, no slots, no hoverCursor
  export let interactive = true;
  export let onClickLine = (f: Feature) => {};

  let gj: FeatureCollection;
  let maxShortcuts: number;
  // TODO if we could set both reactively, thatd be ideal
  $: render(gjInput);

  function render(x: FeatureCollection) {
    // A qualitative palette from colorbrewer2.org, skipping the red hue (used
    // for levels of shortcutting) and grey (too close to the basemap)
    let cell_colors = [
      "#8dd3c7",
      "#ffffb3",
      "#bebada",
      "#80b1d3",
      "#fdb462",
      "#b3de69",
      "#fccde5",
      "#bc80bd",
      "#ccebc5",
      "#ffed6f",
    ];

    maxShortcuts = Math.max(
      ...gjInput.features.map((f) => f.properties!.shortcuts ?? 0)
    );

    for (let f of gjInput.features) {
      f.properties ??= {};
      if (f.properties.cell_color == "disconnected") {
        f.properties.color = "red";
      } else if (Object.hasOwn(f.properties, "cell_color")) {
        f.properties.color =
          cell_colors[f.properties.cell_color % cell_colors.length];
      }
    }

    gj = gjInput;
  }
</script>

<GeoJSON data={gj} generateId>
  <FillLayer
    beforeId={$showBasemap ? "Building" : undefined}
    filter={["==", ["get", "kind"], "cell"]}
    paint={{
      "fill-color": ["get", "color"],
      "fill-opacity": 0.3,
    }}
  />

  <LineLayer
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
  <slot name="more-layers" />
</GeoJSON>
