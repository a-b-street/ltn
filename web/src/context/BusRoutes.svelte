<script lang="ts">
  import { LineLayer, VectorTileSource } from "svelte-maplibre";
  import { ContextLayerButton, layerId, roadLineWidth } from "../common";
  import { assetUrl } from "../stores";

  let show = $state(false);
</script>

<ContextLayerButton bind:show label="Bus routes">
  {#snippet help()}
    <p>
      These are all <a
        href="https://wiki.openstreetmap.org/wiki/Tag:route%3Dbus"
        target="_blank"
      >
        bus routes
      </a>
      according to OpenStreetMap.
    </p>
  {/snippet}
</ContextLayerButton>

<VectorTileSource
  url={`pmtiles://${assetUrl("cnt/layers/bus_routes.pmtiles")}`}
>
  <LineLayer
    {...layerId("context-bus-routes")}
    sourceLayer="bus_routes"
    paint={{
      "line-color": "blue",
      "line-width": roadLineWidth(0),
    }}
    layout={{
      visibility: show ? "visible" : "none",
    }}
  />
</VectorTileSource>
