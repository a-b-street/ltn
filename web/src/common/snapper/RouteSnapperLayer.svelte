<script lang="ts">
  import { CircleLayer, FillLayer, GeoJSON, LineLayer } from "svelte-maplibre";
  import { layerId } from "../";
  import {
    constructMatchExpression,
    isLine,
    isPoint,
    isPolygon,
  } from "svelte-utils/map";
  import { routeToolGj } from "./stores";

  const circleRadiusPixels = 10;
</script>

<GeoJSON data={$routeToolGj}>
  <CircleLayer
    {...layerId("route-points")}
    filter={isPoint}
    paint={{
      "circle-color": constructMatchExpression(
        ["get", "type"],
        {
          "snapped-waypoint": "red",
          "free-waypoint": "blue",
        },
        "black",
      ),
      "circle-opacity": ["case", ["has", "hovered"], 0.5, 1.0],
      "circle-radius": constructMatchExpression(
        ["get", "type"],
        { node: circleRadiusPixels / 2.0 },
        circleRadiusPixels,
      ),
    }}
  />
  <LineLayer
    {...layerId("route-lines")}
    filter={isLine}
    paint={{
      "line-color": ["case", ["get", "snapped"], "red", "blue"],
      "line-width": 2.5,
    }}
  />
  <FillLayer
    {...layerId("route-polygons")}
    filter={isPolygon}
    paint={{
      "fill-color": "black",
      "fill-opacity": 0.5,
    }}
  />
</GeoJSON>
