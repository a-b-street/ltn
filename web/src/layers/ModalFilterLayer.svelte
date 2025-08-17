<script lang="ts">
  import type { FeatureCollection } from "geojson";
  import type { Snippet } from "svelte";
  import { GeoJSON, SymbolLayer, type LayerClickInfo } from "svelte-maplibre";
  import { emptyGeojson } from "svelte-utils/map";
  import { layerId } from "../common";
  import {
    backend,
    mutationCounter,
    showExistingFiltersAndTRs,
  } from "../stores";
  // TODO Maybe make another component wrapping both modal filters and turn
  // restrictions, since all callers want both
  import TurnRestrictionLayer from "./TurnRestrictionLayer.svelte";

  export let modalFilterGj: FeatureCollection | null = null;
  export let turnRestrictionGj: FeatureCollection | null = null;
  export let onClickModalFilter: (e: LayerClickInfo) => void = () => {};
  export let onClickTurnRestriction: (e: LayerClickInfo) => void = () => {};
  export let modalFilterPopup: Snippet | undefined = undefined;
  export let turnRestrictionPopup: Snippet | undefined = undefined;

  export let interactive: boolean;
  export let show = true;
  export let prefix = "";

  let minzoom = 13;
  // TODO Runes would make this so nicer. The > 0 part is a hack...
  $: gj =
    $mutationCounter > 0 && modalFilterGj == null
      ? $backend!.renderModalFilters()
      : emptyGeojson();
</script>

<GeoJSON data={modalFilterGj || gj} generateId>
  <SymbolLayer
    {...layerId(prefix + "modal-filters")}
    {interactive}
    filter={[
      "all",
      ["!=", ["get", "filter_kind"], "diagonal_filter"],
      $showExistingFiltersAndTRs ? ["literal", true] : ["get", "edited"],
    ]}
    {minzoom}
    layout={{
      "icon-image": ["get", "filter_kind"],
      "icon-rotate": ["get", "angle"],
      "icon-allow-overlap": true,
      "icon-size": 0.1,
      visibility: show ? "visible" : "none",
    }}
    paint={{
      "icon-opacity": ["case", ["get", "edited"], 1.0, 0.5],
    }}
    onclick={onClickModalFilter}
  >
    {@render modalFilterPopup?.()}
  </SymbolLayer>
  <SymbolLayer
    {...layerId(prefix + "intersection-filters")}
    filter={["==", ["get", "filter_kind"], "diagonal_filter"]}
    {minzoom}
    layout={{
      "icon-image": "diagonal_filter",
      "icon-rotate": ["get", "angle", ["get", "filter"]],
      "icon-allow-overlap": true,
      "icon-size": 0.07,
      visibility: show ? "visible" : "none",
    }}
    interactive={false}
  />
</GeoJSON>

<TurnRestrictionLayer
  {show}
  {prefix}
  {turnRestrictionGj}
  {onClickTurnRestriction}
>
  {@render turnRestrictionPopup?.()}
</TurnRestrictionLayer>
