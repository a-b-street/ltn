<script lang="ts">
  import { SequentialLegend } from "../common";
  import {
    bucketize,
    carOwnershipColorScale,
    carOwnershipLimits,
    combinedColorScale,
    poiColorScale,
    populationDensityColorScale,
    simdColorScale,
    simdLimits,
    stats19ColorScale,
  } from "../common/colors";
  import { appFocus, metricBuckets } from "../stores";
  import type { Prioritization } from "./index";

  export let selectedPrioritization: Prioritization;

  function setSelectedPrioritizationFromURL() {
    let currentURLParam = new URL(window.location.href).searchParams.get(
      "prioritizationMetric",
    );

    if (currentURLParam == "population_density") {
      selectedPrioritization = "population_density";
    } else if (currentURLParam == "simd") {
      selectedPrioritization = "simd";
    } else if (currentURLParam == "stats19") {
      selectedPrioritization = "stats19";
    } else if (currentURLParam == "pois") {
      selectedPrioritization = "pois";
    } else if (currentURLParam == "car_ownership") {
      selectedPrioritization = "car_ownership";
    } else if (currentURLParam == "combined") {
      selectedPrioritization = "combined";
    }
  }

  if ($appFocus == "cnt") {
    setSelectedPrioritizationFromURL();
  } else {
    selectedPrioritization = "none";
  }
  $: {
    const url = new URL(window.location.href);
    if (selectedPrioritization == "none") {
      url.searchParams.delete("prioritizationMetric");
    } else {
      url.searchParams.set("prioritizationMetric", selectedPrioritization);
    }
    history.replaceState({}, "", url);
  }
</script>

<div style="display: flex; gap: 16px; align-items: center; width: fit-content;">
  <label style="margin: 0; padding: 0;" for="prioritization-selection"
    >Metric</label
  >
  <select id="prioritization-selection" bind:value={selectedPrioritization}>
    <option value="none">None</option>
    <option value="simd">SIMD</option>
    <option value="population_density">Population density</option>
    <option value="car_ownership">Car ownership</option>
    <option value="pois">Points of interest</option>
    <option value="stats19">Collisions</option>
    <option value="combined">Combined</option>
  </select>
</div>

{#if selectedPrioritization == "population_density"}
  <SequentialLegend
    colorScale={populationDensityColorScale}
    labels={{ limits: $metricBuckets.population_density }}
  />
  <div class="sub-labels">
    <span>Less dense</span>
    <span>people / km²</span>
    <span>More dense</span>
  </div>
{:else if selectedPrioritization == "car_ownership"}
  <SequentialLegend
    colorScale={carOwnershipColorScale}
    labels={{ limits: carOwnershipLimits.map((number) => `${number}%`) }}
  />
  <div style="display: flex; justify-content: space-between;">
    <span style="text-align: center; width: 100%">
      Households with at least one car or van
    </span>
  </div>
{:else if selectedPrioritization == "simd"}
  <SequentialLegend
    colorScale={simdColorScale}
    labels={{ buckets: bucketize(simdLimits) }}
  />
  <div class="sub-labels">
    <span>More deprived</span>
    <span>Less deprived</span>
  </div>
{:else if selectedPrioritization == "stats19"}
  <SequentialLegend
    colorScale={stats19ColorScale}
    labels={{ limits: $metricBuckets.collision_density }}
  />
  <div style="text-align: center;">collisions / km²</div>
{:else if selectedPrioritization == "pois"}
  <SequentialLegend
    colorScale={poiColorScale}
    labels={{ limits: $metricBuckets.poi_density }}
  />
  <div style="text-align: center;">POIs / km²</div>
{:else if selectedPrioritization == "combined"}
  <SequentialLegend
    colorScale={combinedColorScale}
    labels={{ limits: [1, 2, 3, 4, 5] }}
  />
  <div class="sub-labels">
    <span>Least important</span>
    <span>Most important</span>
  </div>
  <br />
  <p>
    This metric combines the other five by averaging each of the metrics, on the
    1-5 scale.
  </p>
{/if}

<style>
  .sub-labels {
    display: flex;
    justify-content: space-between;
  }
</style>
