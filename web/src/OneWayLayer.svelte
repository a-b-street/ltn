<script lang="ts">
  import { layerId } from "./common";
  import { SymbolLayer } from "svelte-maplibre";

  // TODO Using the icon from maptiler basemap, so it won't work on some basemaps
</script>

<SymbolLayer
  {...layerId("one-ways")}
  filter={[
    "all",
    ["==", ["get", "kind"], "interior_road"],
    ["!=", ["get", "direction"], "both"],
  ]}
  layout={{
    "icon-image": "oneway",
    "icon-size": 1.0,
    "symbol-placement": "line",
    "symbol-spacing": 50,
    "icon-allow-overlap": true,
    "icon-rotate": ["case", ["==", ["get", "direction"], "forwards"], 0, 180],
  }}
  paint={{
    "icon-opacity": 1.0,
  }}
/>
