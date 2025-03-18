<script lang="ts">
  import { CircleX } from "lucide-svelte";
  import { prettyPrintPercent } from "../common";
  import { appFocus } from "../stores";
  import type { GeneratedBoundaryFeature } from "../wasm";

  export let neighbourhoodBoundary: GeneratedBoundaryFeature;
</script>

<div class="container">
  <table>
    <tr>
      <th>Area</th>
      <td>
        {neighbourhoodBoundary.properties.area_km2.toFixed(1)} km²
      </td>
    </tr>
    {#if $appFocus == "cnt"}
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
        <th>SIMD</th>
        <td>{neighbourhoodBoundary.properties.simd.toFixed(1)}%</td>
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
    {/if}
  </table>
</div>

<style>
  .container {
    border: dashed black 2px;
    border-radius: 8px;
    margin-bottom: 16px;
  }
</style>
