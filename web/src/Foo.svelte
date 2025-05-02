<script lang="ts">
        import { layerId } from "./common";
  import { SymbolLayer } from "svelte-maplibre";
</script>

        <SymbolLayer
          {...layerId("neighbourhood-boundaries-ids", false)}
        paint={{
          'text-color': '#333',
          'text-halo-color': '#eee',
          'text-halo-width': 0.5,
          'text-halo-blur': 0.5,
        }}
        layout={{
            "symbol-placement": "line-center",
          'text-allow-overlap': true,
          'text-field': ["literal", "lol"],
          'text-size': 16,
        }}
        />
