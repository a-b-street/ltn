<script lang="ts">
  import { CircleLayer, GeoJSON } from "svelte-maplibre";
  import { app, mutationCounter } from "./stores";

  // TODO Runes would make this so nicer. The > 0 part is a hack...
  $: gj = $mutationCounter > 0 ? JSON.parse($app!.renderModalFilters()) : null;
</script>

<GeoJSON data={gj}>
  <CircleLayer
    paint={{
      "circle-radius": 15,
      "circle-color": "black",
    }}
  />
</GeoJSON>
