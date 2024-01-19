<script lang="ts">
  import { CircleLayer, FillLayer, GeoJSON, LineLayer } from "svelte-maplibre";
  import { isLine, isPoint, isPolygon } from "../";
  import { polygonToolGj } from "./stores";
</script>

<GeoJSON data={$polygonToolGj}>
  <FillLayer
    filter={isPolygon}
    paint={{
      "fill-color": "red",
      "fill-opacity": [
        "case",
        ["boolean", ["get", "hover"], "false"],
        1.0,
        0.5,
      ],
    }}
  />
  <LineLayer
    filter={isLine}
    paint={{
      // TODO Dashed
      "line-color": "black",
      "line-width": 8,
      "line-opacity": 0.5,
    }}
  />
  <CircleLayer
    filter={isPoint}
    paint={{
      "circle-color": "black",
      "circle-opacity": ["case", ["has", "hovered"], 1.0, 0.5],
      "circle-radius": 10,
    }}
  />
</GeoJSON>
