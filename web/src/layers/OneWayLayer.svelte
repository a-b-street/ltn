<script lang="ts">
  import { SymbolLayer } from "svelte-maplibre";
  import { layerId } from "../common";

  // TODO Figure out if hoverCursor is necessary here, or if svelte-maplibre
  // ignores it when interactive is false
</script>

<SymbolLayer
  {...layerId("one-ways")}
  filter={[
    "all",
    ["in", ["get", "kind"], ["literal", ["interior_road", "main_road"]]],
    ["!=", ["get", "travel_flow"], "both"],
  ]}
  layout={{
    "icon-image": "oneway_arrow",
    "icon-size": 1.0,
    "symbol-placement": "line",
    "symbol-spacing": 50,
    "icon-allow-overlap": true,
    "icon-rotate": ["case", ["==", ["get", "travel_flow"], "forwards"], 0, 180],
  }}
  paint={{
    "icon-opacity": ["case", ["get", "travel_flow_edited"], 1.0, 0.5],
  }}
  minzoom={13}
  interactive={false}
  hoverCursor="pointer"
/>
