<script lang="ts">
  export let colorScale;
  type IntoLabel = string | number;
  export let labels: { limits: IntoLabel[] } | { buckets: IntoLabel[] };
  export let decimalPlaces = 0;

  let intoLabels: IntoLabel[] = [];
  if ("limits" in labels) {
    intoLabels = labels.limits;
  } else {
    intoLabels = labels.buckets;
  }

  let labelTexts: string[] = intoLabels.map((label) => {
    if (typeof label === "number") {
      if (decimalPlaces > 0) {
        return label.toFixed(decimalPlaces);
      } else {
        return label.toLocaleString();
      }
    } else {
      return label;
    }
  });
</script>

<div class="colors">
  {#each colorScale as color}
    <span class="bucket" style="background: {color};">&nbsp;</span>
  {/each}
</div>

<div
  class="labels"
  class:bucketed={labels.hasOwnProperty("buckets")}
  class:limits={labels.hasOwnProperty("limits")}
>
  {#each labelTexts as labelText}
    <span>{labelText}</span>
  {/each}
</div>

<style>
  .colors {
    display: flex;
    justify-content: space-around;
  }

  .colors .bucket {
    flex: 1;
    border: 1px solid black;
    /* "collapse" the double border between the inner buckets */
    border-left: 0;
  }
  .colors .bucket:first-child {
    border-left: 1px solid black;
  }

  /* align the colors so that the limits labels fall between color bucket */
  .colors::before,
  .colors::after {
    content: "";
    flex: 0.5;
  }

  .labels {
    display: flex;
    justify-content: space-around;
    text-align: center;
  }

  .labels * {
    flex: 1;
  }
  .labels.bucketed::before,
  .labels.bucketed::after {
    content: "";
    flex: 0.5;
  }
</style>
