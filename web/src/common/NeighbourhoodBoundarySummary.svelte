<script lang="ts">
  import { prettyPrintPercent } from "../common";
  import { appFocus } from "../stores";
  import type { GeneratedBoundaryFeature } from "../wasm";

  export let neighbourhoodBoundary: GeneratedBoundaryFeature;
</script>

<table>
  <tr>
    <th>Area</th>
    <td>
      {neighbourhoodBoundary.properties.area_km2.toFixed(1)} km²
    </td>
  </tr>

  {#if $appFocus == "cnt"}
    <tr>
      <th>SIMD</th>
      <td
        >quintile {1 +
          Math.floor(neighbourhoodBoundary.properties.simd / 20)}</td
      >
    </tr>

    <tr>
      <th>Population density</th>
      <td>
        {Math.round(
          neighbourhoodBoundary.properties.population /
            neighbourhoodBoundary.properties.area_km2,
        ).toLocaleString()} people / km²
      </td>
    </tr>

    <tr>
      <th>Car ownership</th>
      <td>
        {prettyPrintPercent(
          neighbourhoodBoundary.properties.households_with_cars_or_vans,
          neighbourhoodBoundary.properties.total_households,
        )} of households
      </td>
    </tr>

    <tr>
      <th>POI density</th>
      <td>
        {(
          neighbourhoodBoundary.properties.number_pois /
          neighbourhoodBoundary.properties.area_km2
        ).toFixed(1)} / km²
      </td>
    </tr>

    <tr>
      <th>Collision density</th>
      <td>
        {(
          neighbourhoodBoundary.properties.number_stats19_collisions /
          neighbourhoodBoundary.properties.area_km2
        ).toFixed(1)} / km²
      </td>
    </tr>
  {/if}
</table>

<style>
  table {
    margin: 0;
    padding: 0;
  }
</style>
