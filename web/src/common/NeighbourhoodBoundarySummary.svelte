<script lang="ts">
  import { prettyPrintPercent } from "../common";
  import {
    carOwnershipColorScale,
    carOwnershipLimits,
    combinedColorScale,
    combinedLimits,
    poiColorScale,
    populationDensityColorScale,
    simdColorScale,
    simdLimits,
    stats19ColorScale,
  } from "../common/colors";
  import { appFocus, metricBuckets } from "../stores";
  import type { GeneratedBoundaryFeature } from "../wasm";
  import MetricProgress from "./MetricProgress.svelte";

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
      <th
        >SIMD
        <MetricProgress
          colorScale={simdColorScale}
          limits={simdLimits}
          value={neighbourhoodBoundary.properties.simd}
        />
      </th>
      <td
        >quintile {1 +
          Math.floor(neighbourhoodBoundary.properties.simd / 20)}</td
      >
    </tr>

    <tr>
      <th
        >Population density
        <MetricProgress
          colorScale={populationDensityColorScale}
          limits={$metricBuckets.population_density}
          value={neighbourhoodBoundary.properties.population /
            neighbourhoodBoundary.properties.area_km2}
        />
      </th>
      <td>
        {Math.round(
          neighbourhoodBoundary.properties.population /
            neighbourhoodBoundary.properties.area_km2,
        ).toLocaleString()} people / km²
      </td>
    </tr>

    <tr>
      <th
        >Car ownership
        <MetricProgress
          colorScale={carOwnershipColorScale.toReversed()}
          limits={carOwnershipLimits}
          value={100 *
            (1 -
              neighbourhoodBoundary.properties.households_with_cars_or_vans /
                neighbourhoodBoundary.properties.total_households)}
        />
      </th>
      <td>
        {prettyPrintPercent(
          neighbourhoodBoundary.properties.households_with_cars_or_vans,
          neighbourhoodBoundary.properties.total_households,
        )} of households
      </td>
    </tr>

    <tr>
      <th
        >POI density
        <MetricProgress
          colorScale={poiColorScale}
          limits={$metricBuckets.poi_density}
          value={neighbourhoodBoundary.properties.number_pois /
            neighbourhoodBoundary.properties.area_km2}
        />
      </th>
      <td>
        {(
          neighbourhoodBoundary.properties.number_pois /
          neighbourhoodBoundary.properties.area_km2
        ).toFixed(1)} / km²
      </td>
    </tr>

    <tr>
      <th
        >Collision density
        <MetricProgress
          colorScale={stats19ColorScale}
          limits={$metricBuckets.collision_density}
          value={neighbourhoodBoundary.properties.number_stats19_collisions /
            neighbourhoodBoundary.properties.area_km2}
        />
      </th>
      <td>
        {(
          neighbourhoodBoundary.properties.number_stats19_collisions /
          neighbourhoodBoundary.properties.area_km2
        ).toFixed(1)} / km²
      </td>
    </tr>

    <tr>
      <th
        >Overall prioritisation score
        <MetricProgress
          colorScale={combinedColorScale}
          limits={combinedLimits}
          value={neighbourhoodBoundary.properties.combined_score}
        />
      </th>
      <td>
        {neighbourhoodBoundary.properties.combined_score} / 5
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
