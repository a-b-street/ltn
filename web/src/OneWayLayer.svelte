<script lang="ts">
  import { layerId } from "./common";
  import { SymbolLayer } from "svelte-maplibre";
</script>

<SymbolLayer
  {...layerId("one-ways")}
  filter={[
    "all",
    ["==", ["get", "kind"], "interior_road"],
    ["!=", ["get", "direction"], "both"],
  ]}
  layout={{
    "icon-image": "oneway-arrow",
    "icon-size": 1.0,
    "symbol-placement": "line",
    "symbol-spacing": 50,
    "icon-allow-overlap": true,
    "icon-rotate": ["case", ["==", ["get", "direction"], "forwards"], 0, 180],
  }}
  paint={{
    "icon-opacity": ["case", ["get", "direction_edited"], 1.0, 0.5],
  }}
/>
