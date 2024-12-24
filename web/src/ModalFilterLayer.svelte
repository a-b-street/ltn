<script lang="ts">
  import { layerId } from "./common";
  import { GeoJSON, SymbolLayer } from "svelte-maplibre";
  import { app, mutationCounter } from "./stores";

  // TODO Runes would make this so nicer. The > 0 part is a hack...
  $: gj = $mutationCounter > 0 ? JSON.parse($app!.renderModalFilters()) : null;
</script>

<GeoJSON data={gj} generateId>
  <SymbolLayer
    {...layerId("modal-filters")}
    layout={{
      "icon-image": ["get", "filter_kind"],
      "icon-rotate": ["get", "angle"],
      "icon-allow-overlap": true,
      "icon-size": 0.1,
    }}
    paint={{
      "icon-opacity": ["case", ["get", "edited"], 1.0, 0.5],
    }}
    on:click
  >
    <slot />
  </SymbolLayer>
</GeoJSON>
