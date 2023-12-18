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
          interior_road: "black",
          crosses: "blue",
        },
        "red"
      ),
    }}
    on:click={(e) => window.open(e.detail.features[0].properties.way, "_blank")}
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
