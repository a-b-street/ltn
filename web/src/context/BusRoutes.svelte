<script lang="ts">
  import { LineLayer, VectorTileSource } from "svelte-maplibre";
  import { HelpButton, layerId, roadLineWidth } from "../common";
  import { assetUrl } from "../stores";

  let show = false;
</script>

<button class="secondary" on:click={() => (show = !show)}>Bus routes</button>
{#if show}
  <HelpButton>
    <p>
      These are all <a
        href="https://wiki.openstreetmap.org/wiki/Tag:route%3Dbus"
        target="_blank"
      >
        bus routes
      </a>
      according to OpenStreetMap.
    </p>
  </HelpButton>
{/if}

<VectorTileSource
  url={`pmtiles://${assetUrl("cnt_layers/bus_routes.pmtiles")}`}
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
