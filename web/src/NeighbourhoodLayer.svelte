<script lang="ts">
  import { MapModel } from "backend";
  import type { Feature, Polygon } from "geojson";
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

  export let model: MapModel;
  export let boundary: Feature<Polygon>;

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

  let details = JSON.parse(model.analyzeNeighbourhood(boundary));
  let maxShortcuts = Math.max(
    ...details.features.map((f) => f.properties.shortcuts ?? 0)
  );
  for (let f of details.features) {
    if (f.properties.color == "disconnected") {
      f.properties.color = "red";
    } else if (f.properties.color) {
      f.properties.color = cell_colors[f.properties.color % cell_colors.length];
    }
  }
</script>

<!--<GeoJSON data={boundary}>
  <FillLayer
    paint={{
      "fill-color": "black",
      "fill-opacity": 0.2,
    }}
  />
</GeoJSON>-->

<GeoJSON data={details} generateId>
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
        },
        "red"
      ),
    }}
  >
    <Popup openOn="hover" let:data>
      <PropertiesTable properties={data.properties} />
    </Popup>
  </CircleLayer>
  <FillLayer
    filter={isPolygon}
    manageHoverState
    paint={{
      "fill-color": ["get", "color"],
      "fill-opacity": hoverStateFilter(0.6, 1.0),
    }}
  />
</GeoJSON>
