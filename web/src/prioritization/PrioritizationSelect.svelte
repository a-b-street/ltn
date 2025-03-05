<script lang="ts">
  import { SequentialLegend } from "svelte-utils";
  import {
    areaLimits,
    bucketize,
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

    if (currentURLParam == "area") {
      selectedPrioritization = "area";
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
    <option value="area">Area (km²)</option>
    <option value="simd">SIMD</option>
    <option value="stats19">Collisions</option>
  </select>
</div>

{#if selectedPrioritization == "simd"}
  <SequentialLegendBucketed
    colorScale={simdColorScale}
    buckets={bucketize(simdLimits)}
  />
  <div style="display: flex; justify-content: space-between;">
    <span>More deprived</span>
    <span>Less deprived</span>
  </div>
{:else if selectedPrioritization == "area"}
  <SequentialLegend colorScale={simdColorScale} limits={areaLimits} />
{:else if selectedPrioritization == "stats19"}
  <SequentialLegend colorScale={stats19ColorScale} limits={stats19Limits} />
{/if}
