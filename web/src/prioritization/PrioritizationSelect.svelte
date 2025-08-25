<script lang="ts">
  import { run } from 'svelte/legacy';

  import { SequentialLegend } from "svelte-utils";
  import {
    areaColorScale,
    areaLimits,
    bucketize,
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
  import type { Prioritization } from "./index";

  interface Props {
    selectedPrioritization: Prioritization;
  }

  let { selectedPrioritization = $bindable() }: Props = $props();

  function setSelectedPrioritizationFromURL() {
    let currentURLParam = new URL(window.location.href).searchParams.get(
      "prioritisationMetric",
    );

    if (currentURLParam == "area") {
      selectedPrioritization = "area";
    } else if (currentURLParam == "population_density") {
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
  run(() => {
    const url = new URL(window.location.href);
    if (selectedPrioritization == "none") {
      url.searchParams.delete("prioritisationMetric");
    } else {
      url.searchParams.set("prioritisationMetric", selectedPrioritization);
    }
    history.replaceState({}, "", url);
  });
</script>

<div style="display: flex; gap: 16px; align-items: center; width: fit-content;">
  <label style="margin: 0; padding: 0;" for="prioritization-selection"
    >Metric</label
  >
  <select id="prioritization-selection" bind:value={selectedPrioritization}>
    <option value="none">None</option>
    <option value="area">Area</option>
    <option value="simd">SIMD</option>
    <option value="population_density">Population density</option>
    <option value="car_ownership">Car ownership</option>
    <option value="pois">Points of interest</option>
    <option value="stats19">Collisions</option>
    <option value="combined">Overall prioritisation score</option>
  </select>
</div>

{#if selectedPrioritization == "area"}
  <SequentialLegend
    colorScale={areaColorScale}
    labels={{ limits: areaLimits }}
  />
  <div class="sub-labels">
    <span></span>
    <span>km²</span>
    <span></span>
  </div>
{:else if selectedPrioritization == "population_density"}
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
    colorScale={carOwnershipColorScale.toReversed()}
    labels={{
      limits: carOwnershipLimits.toReversed().map((number) => `${number}%`),
    }}
  />
  <div style="display: flex; justify-content: space-between;">
    <span style="text-align: center; width: 100%">
      Households with at least one car or van
    </span>
  </div>
{:else if selectedPrioritization == "simd"}
  <SequentialLegend
    colorScale={simdColorScale.toReversed()}
    labels={{ buckets: bucketize(simdLimits).toReversed() }}
  />
  <div class="sub-labels">
    <span>Less deprived</span>
    <span>More deprived</span>
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
    labels={{ buckets: combinedLimits }}
  />
  <div class="sub-labels">
    <span>Least important</span>
    <span>Most important</span>
  </div>
  <br />
  <p>This score averages the other five metrics, on the 1-5 scale.</p>
{/if}

<style>
  .sub-labels {
    display: flex;
    justify-content: space-between;
  }
</style>
