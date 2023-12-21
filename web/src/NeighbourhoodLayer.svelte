<script lang="ts">
  import { LTN } from "backend";
  import type { Feature, Polygon } from "geojson";
  import type { Map, MapMouseEvent } from "maplibre-gl";
  import { onDestroy } from "svelte";
  import {
    CircleLayer,
    FillLayer,
    GeoJSON,
    hoverStateFilter,
    LineLayer,
    Popup,
    type LayerClickInfo,
  } from "svelte-maplibre";
  import {
    constructMatchExpression,
    isLine,
    isPoint,
    isPolygon,
    PropertiesTable,
  } from "./common";

  export let map: Map;
  export let app: LTN;
  export let boundary: Feature<Polygon>;
  export let offlineMode: boolean;

  export let addingFilter = false;
  export let undoLength = 0;
  export let redoLength = 0;
  export let rerender = 0;

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

  let details;
  let maxShortcuts;
  render(JSON.parse(app.analyzeNeighbourhood(boundary)));

  function render(gj) {
    maxShortcuts = Math.max(
      ...gj.features.map((f) => f.properties.shortcuts ?? 0)
    );

    for (let f of gj.features) {
      if (f.properties.color == "disconnected") {
        f.properties.color = "red";
      } else if (Object.hasOwn(f.properties, "color")) {
        f.properties.color =
          cell_colors[f.properties.color % cell_colors.length];
      }
    }

    undoLength = gj.undo_length;
    redoLength = gj.redo_length;

    details = gj;
  }

  $: if (addingFilter) {
    map.on("click", onClick);
    map.style.cursor = "crosshair";
  }
  onDestroy(() => {
    stopAddingFilter();
    app.unsetNeighbourhood();
  });
  function onClick(e: MapMouseEvent) {
    render(JSON.parse(app.addModalFilter(e.lngLat)));
    stopAddingFilter();
  }
  function stopAddingFilter() {
    addingFilter = false;
    map.off("click", onClick);
    map.style.cursor = "inherit";
  }

  function deleteFilter(e: CustomEvent<LayerClickInfo>) {
    let props = e.detail.features[0].properties;
    if (props.kind == "modal_filter") {
      render(JSON.parse(app.deleteModalFilter(props.road)));
    }
  }

  $: if (rerender > 0) {
    render(JSON.parse(app.rerender()));
  }
</script>

<GeoJSON data={details} generateId>
  <FillLayer
    beforeId={offlineMode ? undefined : "Building"}
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
    on:click={(e) => window.open(e.detail.features[0].properties.way, "_blank")}
    hoverCursor="pointer"
  >
    <Popup openOn="hover" let:data>
      <PropertiesTable properties={data.properties} />
    </Popup>
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
    on:click={deleteFilter}
  >
    <Popup openOn="hover" let:data>
      <PropertiesTable properties={data.properties} />
    </Popup>
  </CircleLayer>
</GeoJSON>
