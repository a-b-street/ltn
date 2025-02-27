<script lang="ts">
  import { GeoJSON, SymbolLayer, type LayerClickInfo } from "svelte-maplibre";
  import { emptyGeojson } from "svelte-utils/map";
  import { layerId } from "../common";
  import { backend, mutationCounter } from "../stores";
  // TODO Maybe make another component wrapping both modal filters and turn
  // restrictions, since all callers want both
  import TurnRestrictionLayer from "./TurnRestrictionLayer.svelte";

  export let onClickModalFilter: (
    e: CustomEvent<LayerClickInfo>,
  ) => void = () => {};
  export let onClickTurnRestriction: (
    e: CustomEvent<LayerClickInfo>,
  ) => void = () => {};

  let minzoom = 13;
  // TODO Runes would make this so nicer. The > 0 part is a hack...
  $: gj =
    $mutationCounter > 0 ? $backend!.renderModalFilters() : emptyGeojson();
</script>

<GeoJSON data={gj} generateId>
  <SymbolLayer
    {...layerId("modal-filters")}
    filter={["!=", ["get", "filter_kind"], "diagonal_filter"]}
    {minzoom}
    layout={{
      "icon-image": ["get", "filter_kind"],
      "icon-rotate": ["get", "angle"],
      "icon-allow-overlap": true,
      "icon-size": 0.1,
    }}
    paint={{
      "icon-opacity": ["case", ["get", "edited"], 1.0, 0.5],
    }}
    on:click={onClickModalFilter}
  >
    <slot name="modal-filter" />
  </SymbolLayer>
  <SymbolLayer
    {...layerId("intersection-filters")}
    filter={["==", ["get", "filter_kind"], "diagonal_filter"]}
    {minzoom}
    layout={{
      "icon-image": "diagonal_filter",
      "icon-rotate": ["get", "angle", ["get", "filter"]],
      "icon-allow-overlap": true,
      "icon-size": 0.07,
    }}
    interactive={false}
  />
</GeoJSON>

<TurnRestrictionLayer {onClickTurnRestriction}>
  <slot name="turn-restriction" />
</TurnRestrictionLayer>
