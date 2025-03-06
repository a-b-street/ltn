<script lang="ts">
  import { SequentialLegend } from "svelte-utils";
  import {
    bucketize,
    densityColorScale,
    densityLimits,
    simdColorScale,
    simdLimits,
    stats19ColorScale,
    stats19Limits,
  } from "../common/colors";
  import SequentialLegendBucketed from "../common/SequentialLegendBucketed.svelte";
  import { projectName } from "../stores";
  import type { Prioritization } from "./index";

  export let selectedPrioritization: Prioritization;

  function setSelectedPrioritizationFromURL() {
    let currentURLParam = new URL(window.location.href).searchParams.get(
      "prioritizationMetric",
    );

    if (currentURLParam == "density") {
      selectedPrioritization = "density";
    } else if (currentURLParam == "simd") {
      selectedPrioritization = "simd";
    } else if (currentURLParam == "stats19") {
      selectedPrioritization = "stats19";
    }
  }

  if ($projectName.startsWith("ltn_cnt/")) {
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

<div style="display: flex; gap: 16px; align-items: center;">
  <label for="prioritization-selection">Metric</label>
  <select id="prioritization-selection" bind:value={selectedPrioritization}>
    <option value="none">None</option>
    <option value="density">Density</option>
    <option value="stats19">Collisions</option>
    <option value="simd">SIMD</option>
  </select>
</div>

{#if selectedPrioritization == "density"}
  <SequentialLegend colorScale={densityColorScale} limits={densityLimits} />
  <div class="sub-labels">
    <span>Less less</span>
    <span>More dense</span>
  </div>
{:else if selectedPrioritization == "simd"}
  <SequentialLegendBucketed
    colorScale={simdColorScale}
    buckets={bucketize(simdLimits)}
  />
  <div class="sub-labels">
    <span>More deprived</span>
    <span>Less deprived</span>
  </div>
{:else if selectedPrioritization == "stats19"}
  <SequentialLegend colorScale={stats19ColorScale} limits={stats19Limits} />
  <div style="text-align: center;">collisions / km²</div>
{/if}

<style>
  .sub-labels {
    display: flex;
    justify-content: space-between;
  }
</style>
