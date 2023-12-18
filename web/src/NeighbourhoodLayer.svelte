<script lang="ts">
  import type { Feature, Polygon } from "geojson";
  import { FillLayer, GeoJSON } from "svelte-maplibre";

  export let boundary: Feature<Polygon>;
</script>

<GeoJSON data={boundary}>
  <FillLayer
    paint={{
      "fill-color": "black",
      "fill-opacity": 0.5,
    }}
  />
</GeoJSON>
