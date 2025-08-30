<script lang="ts">
  import type { Feature, FeatureCollection, Geometry } from "geojson";
  import {
    GeoJSON,
    LineLayer,
    SymbolLayer,
    type LayerClickInfo,
  } from "svelte-maplibre";
  import { emptyGeojson } from "svelte-utils/map";
  import { layerId } from "../common";
  import {
    backend,
    mutationCounter,
    showExistingFiltersAndTRs,
  } from "../stores";

  export let turnRestrictionGj: FeatureCollection | null = null;
  export let show = true;
  export let prefix = "";
  export let onClickTurnRestriction: (
    e: CustomEvent<LayerClickInfo>,
  ) => void = () => {};

  let hoveredIcon: Feature | null = null;
  $: showArrow =
    hoveredIcon == null
      ? emptyGeojson()
      : {
          type: "FeatureCollection" as const,
          features: [
            {
              type: "Feature" as const,
              geometry: JSON.parse(
                hoveredIcon.properties!.from_geometry,
              ) as Geometry,
              properties: {},
            },
            {
              type: "Feature" as const,
              geometry: JSON.parse(
                hoveredIcon.properties!.to_geometry,
              ) as Geometry,
              properties: {},
            },
          ],
        };

  // TODO Runes would make this so nicer. The > 0 part is a hack...
  $: gj =
    $mutationCounter > 0 && turnRestrictionGj == null
      ? $backend!.renderTurnRestrictions()
      : emptyGeojson();
</script>

<GeoJSON data={turnRestrictionGj || gj} generateId>
  <SymbolLayer
    {...layerId(prefix + "turn-restrictions")}
    minzoom={13}
    filter={$showExistingFiltersAndTRs ? undefined : ["get", "edited"]}
    layout={{
      "icon-image": ["concat", "no_", ["get", "kind"], "_turn"],
      "icon-rotate": ["get", "icon_angle"],
      "icon-allow-overlap": true,
      "icon-size": 0.05,
      visibility: show ? "visible" : "none",
    }}
    paint={{
      "icon-opacity": ["case", ["get", "edited"], 1.0, 0.5],
    }}
    bind:hovered={hoveredIcon}
    on:click={onClickTurnRestriction}
  >
    <slot />
  </SymbolLayer>
</GeoJSON>

<GeoJSON data={showArrow}>
  <LineLayer
    {...layerId(prefix + "turn-restrictions-debug-arrows")}
    interactive={false}
    paint={{
      "line-width": 4,
      "line-color": "red",
    }}
    layout={{
      visibility: show ? "visible" : "none",
    }}
  />
</GeoJSON>
