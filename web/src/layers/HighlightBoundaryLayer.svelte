<script lang="ts">
  import { FillLayer } from "svelte-maplibre";
  import { layerId } from "../common";
</script>

<FillLayer
  {...layerId("neighbourhood-boundary")}
  filter={["==", ["get", "kind"], "boundary"]}
  paint={{ "fill-color": "black", "fill-opacity": 0.3 }}
/>
