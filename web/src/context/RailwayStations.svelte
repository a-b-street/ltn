<script lang="ts">
  import { GeoJSON, Popup, SymbolLayer } from "svelte-maplibre";
  import nationalRailUrl from "../../assets/national_rail.png?url";
  import { ContextLayerButton, layerId } from "../common";
  import { assetUrl } from "../stores";

  let show = false;
</script>

<ContextLayerButton bind:show label="Railway stations">
  <div slot="legend" style="display: flex; gap: 8px; align-items: center;">
    <img src={nationalRailUrl} alt="National Rail logo" />
    Station
  </div>

  <p slot="help">
    These are all <a
      href="https://wiki.openstreetmap.org/wiki/Tag:railway%3Dstation"
      target="_blank"
    >
      railway stations
    </a>
    according to OpenStreetMap.
  </p>
</ContextLayerButton>

<GeoJSON data={assetUrl("cnt/layers/railways.geojson")} generateId>
  <SymbolLayer
    {...layerId("context-railway-stations")}
    layout={{
      "icon-image": "national_rail",
      "icon-size": 1,
      "icon-allow-overlap": true,
      visibility: show ? "visible" : "none",
    }}
  >
    <Popup openOn="hover">
      {#snippet children({ data })}
        <p>{data!.properties!.name ?? "Unnamed railway station"}</p>
      {/snippet}
    </Popup>
  </SymbolLayer>
</GeoJSON>
