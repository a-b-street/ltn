<script lang="ts">
  import { FillLayer, LineLayer } from "svelte-maplibre";
  import { layerId } from "../common";
  import { roadStyle } from "../stores";
</script>

<FillLayer
  {...layerId("cells")}
  filter={["==", ["get", "kind"], "cell"]}
  layout={{
    visibility: $roadStyle == "cells" ? "none" : "visible",
  }}
  paint={{
    "fill-color": ["get", "color"],
    "fill-opacity": 0.5,
    "fill-outline-color": "hsla(0, 0%, 0%, 0.3)",
  }}
/>

<FillLayer
  {...layerId("border-arrows")}
  filter={["==", ["get", "kind"], "border_arrow"]}
  paint={{
    "fill-color": ["get", "color"],
  }}
/>
<LineLayer
  {...layerId("border-arrow-outlines")}
  filter={["==", ["get", "kind"], "border_arrow"]}
  paint={{
    "line-color": "black",
    "line-width": 2,
  }}
/>
