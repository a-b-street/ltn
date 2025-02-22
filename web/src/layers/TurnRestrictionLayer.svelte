<script lang="ts">
  import type { Feature, Geometry } from "geojson";
  import { GeoJSON, LineLayer, SymbolLayer } from "svelte-maplibre";
  import { emptyGeojson } from "svelte-utils/map";
  import { layerId } from "../common";
  import { backend, mutationCounter } from "../stores";

  let hoveredIcon: Feature | null = null;
  $: showArrow =
    hoveredIcon == null
      ? emptyGeojson()
      : {
          type: "Feature" as const,
          geometry: JSON.parse(hoveredIcon.properties!.arrow) as Geometry,
          properties: {},
        };

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
    bind:hovered={hoveredIcon}
  />
</GeoJSON>

<GeoJSON data={showArrow}>
  <LineLayer
    {...layerId("turn-restrictions-debug-arrows")}
    paint={{
      "line-width": 2,
      "line-color": "red",
    }}
  />
</GeoJSON>
