<script lang="ts">
  import {
    CircleLayer,
    FillLayer,
    GeoJSON,
    hoverStateFilter,
    LineLayer,
    Popup,
  } from "svelte-maplibre";
  import {
    constructMatchExpression,
    isLine,
    isPoint,
    isPolygon,
    PropertiesTable,
  } from "./common";

  export let gjInput;
  export let showBasemap: boolean;
  export let onClickLine = (f) => {};
  export let onClickCircle = (f) => {};
  // TODO Or take some named slots
  export let popups = false;

  let gj;
  let maxShortcuts;
  $: render(gjInput, showBasemap);

  function render(x, y) {
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
      ...gjInput.features.map((f) => f.properties.shortcuts ?? 0)
    );

    for (let f of gjInput.features) {
      if (f.properties.color == "disconnected") {
        f.properties.color = "red";
      } else if (Object.hasOwn(f.properties, "color")) {
        f.properties.color =
          cell_colors[f.properties.color % cell_colors.length];
      }
    }

    gj = gjInput;
  }
</script>

<GeoJSON data={gj} generateId>
  <FillLayer
    beforeId={showBasemap ? "Building" : undefined}
    filter={isPolygon}
    manageHoverState
    paint={{
      "fill-color": ["get", "color"],
      "fill-opacity": hoverStateFilter(0.3, 0.5),
    }}
  />

  <LineLayer
    filter={isLine}
    paint={{
      "line-width": 5,
      "line-color": constructMatchExpression(
        ["get", "kind"],
        {
          interior_road: [
            "interpolate-hcl",
            ["linear"],
            ["get", "shortcuts"],
            0,
            "#F19A93",
            maxShortcuts,
            "#A32015",
          ],
          crosses: "blue",
        },
        "red"
      ),
    }}
    on:click={(e) => onClickLine(e.detail.features[0])}
    hoverCursor="pointer"
  >
    {#if popups}
      <Popup openOn="hover" let:data>
        <PropertiesTable properties={data.properties} />
      </Popup>
    {/if}
  </LineLayer>

  <CircleLayer
    filter={isPoint}
    paint={{
      "circle-radius": 15,
      "circle-color": constructMatchExpression(
        ["get", "kind"],
        {
          border_intersection: "green",
          modal_filter: "black",
        },
        "red"
      ),
    }}
    on:click={(e) => onClickCircle(e.detail.features[0])}
  >
    {#if popups}
      <Popup openOn="hover" let:data>
        <PropertiesTable properties={data.properties} />
      </Popup>
    {/if}
  </CircleLayer>
</GeoJSON>
