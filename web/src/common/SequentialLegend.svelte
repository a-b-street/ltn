<script lang="ts">
  export let colorScale;
  type IntoLabel = string | number;
  export let labels: { limits: IntoLabel[] } | { buckets: IntoLabel[] };
  export let decimalPlaces = 0;
  export let fullWidthBucketLegend = false;

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

<div
  class:full-width-bucket-legend={fullWidthBucketLegend}
  class:bucketed={labels.hasOwnProperty("buckets")}
  class:limits={labels.hasOwnProperty("limits")}
>
  <div class="colors">
    {#each colorScale as color}
      <span class="bucket" style="background: {color};">&nbsp;</span>
    {/each}
  </div>

  <div class="labels">
    {#each labelTexts as labelText}
      <span>{labelText}</span>
    {/each}
  </div>
</div>

<style>
  .colors {
    display: flex;
    justify-content: space-around;
    height: 20px;
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
  :not(.full-width-bucket-legend) .colors::before,
  :not(.full-width-bucket-legend) .colors::after,
  .bucketed:not(.full-width-bucket-legend) .labels::before,
  .bucketed:not(.full-width-bucket-legend) .labels::after {
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
</style>
