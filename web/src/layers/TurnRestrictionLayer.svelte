<script lang="ts">
  import { GeoJSON, SymbolLayer } from "svelte-maplibre";
  import { emptyGeojson, Popup } from "svelte-utils/map";
  import { layerId } from "../common";
  import { backend, mutationCounter } from "../stores";

  // TODO Runes would make this so nicer. The > 0 part is a hack...
  $: gj =
    $mutationCounter > 0 ? $backend!.renderTurnRestrictions() : emptyGeojson();
</script>

<GeoJSON data={gj} generateId>
  <SymbolLayer
    {...layerId("turn-restrictions")}
    layout={{
      "icon-image": ["concat", "no_", ["get", "kind"], "_turn"],
      "icon-rotate": ["get", "icon_angle"],
      "icon-allow-overlap": true,
      "icon-size": 0.05,
    }}
    paint={{
      "icon-opacity": ["case", ["get", "edited"], 1.0, 0.5],
    }}
    manageHoverState
  >
    <Popup let:props>
      No {props.kind} turn, based on bearing {Math.round(props.bearing)}. Rotate
      this icon {Math.round(props.icon_angle)}
    </Popup>
  </SymbolLayer>
</GeoJSON>
