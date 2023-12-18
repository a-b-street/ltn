<script lang="ts">
  import type { Feature } from "geojson";
  import { CircleLayer, FillLayer, GeoJSON, LineLayer } from "svelte-maplibre";
  import {
    constructMatchExpression,
    isLine,
    isPoint,
    isPolygon,
  } from "./index";
  import { routeToolGj } from "./stores";

  const circleRadiusPixels = 10;
</script>

<GeoJSON data={$routeToolGj}>
  <CircleLayer
    filter={isPoint}
    paint={{
      "circle-color": constructMatchExpression(
        ["get", "type"],
        {
          "snapped-waypoint": "red",
          "free-waypoint": "blue",
        },
        "black"
      ),
      "circle-opacity": ["case", ["has", "hovered"], 0.5, 1.0],
      "circle-radius": constructMatchExpression(
        ["get", "type"],
        { node: circleRadiusPixels / 2.0 },
        circleRadiusPixels
      ),
    }}
  />
  <LineLayer
    filter={isLine}
    paint={{
      "line-color": ["case", ["get", "snapped"], "red", "blue"],
      "line-width": 2.5,
    }}
  />
  <FillLayer
    filter={isPolygon}
    paint={{
      "fill-color": "black",
      "fill-opacity": 0.5,
    }}
  />
</GeoJSON>
