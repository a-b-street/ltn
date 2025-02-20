<script lang="ts">
  import { SequentialLegend } from "svelte-utils";
  import { areaLimits, simdColorScale, simdLimits } from "../common/colors";

  export let selectedPrioritization: "none" | "area" | "simd";

  let currentURLParam = new URL(window.location.href).searchParams.get(
    "prioritizationMetric",
  );
  if (currentURLParam == "area") {
    selectedPrioritization = "area";
  } else if (currentURLParam == "simd") {
    selectedPrioritization = "simd";
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
    <option value="simd">Fake SIMD (percentile)</option>
  </select>
</div>

{#if selectedPrioritization == "simd"}
  <SequentialLegend colorScale={simdColorScale} limits={simdLimits} />
{:else if selectedPrioritization == "area"}
  <SequentialLegend colorScale={simdColorScale} limits={areaLimits} />
{/if}
