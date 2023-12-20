<script lang="ts">
  import { MapModel } from "backend";
  import type { Feature, Polygon } from "geojson";
  import {
    CircleLayer,
    FillLayer,
    GeoJSON,
    LineLayer,
    Popup,
  } from "svelte-maplibre";
  import {
    constructMatchExpression,
    isLine,
    isPoint,
    PropertiesTable,
  } from "./common";

  export let model: MapModel;
  export let boundary: Feature<Polygon>;

  let details = JSON.parse(model.analyzeNeighbourhood(boundary));
  let maxShortcuts = Math.max(
    ...details.features.map((f) => f.properties.shortcuts ?? 0)
  );
</script>

<GeoJSON data={boundary}>
  <FillLayer
    paint={{
      "fill-color": "black",
      "fill-opacity": 0.2,
    }}
  />
</GeoJSON>

<GeoJSON data={details}>
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
</GeoJSON>
