<script lang="ts">
  import { GeoJSON, SymbolLayer } from "svelte-maplibre";
  import { Popup } from "svelte-utils/map";
  import nationalRailUrl from "../../assets/national_rail.png?url";
  import { HelpButton, layerId } from "../common";
  import { assetUrl } from "../stores";

  let show = false;
</script>

<button class="secondary" on:click={() => (show = !show)}>
  Railway stations
</button>
{#if show}
  <div>
    <img src={nationalRailUrl} alt="National Rail logo" />
    <HelpButton>
      <p>
        These are all <a
          href="https://wiki.openstreetmap.org/wiki/Tag:railway%3Dstation"
          target="_blank"
        >
          railway stations
        </a>
         according to OpenStreetMap.
      </p>
    </HelpButton>
  </div>
{/if}

<GeoJSON data={assetUrl("cnt_layers/railways.geojson")} generateId>
  <SymbolLayer
    {...layerId("context-railway-stations")}
    layout={{
      "icon-image": "national_rail",
      "icon-size": 1,
      "icon-allow-overlap": true,
      visibility: show ? "visible" : "none",
    }}
  >
    <Popup let:props>
      <p>{props.name ?? "Unnamed railway station"}</p>
    </Popup>
  </SymbolLayer>
</GeoJSON>
