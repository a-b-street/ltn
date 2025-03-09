<script lang="ts">
  export let colorScale: string[];
  export let buckets: any[];
  export let decimalPlaces = 0;
</script>

<div style="display: flex">
  {#each colorScale as color}
    <span style="background: {color}; width: 100%; border: 1px solid black;">
      &nbsp;
    </span>
  {/each}
</div>
<div style="display: flex; justify-content: space-around;">
  {#each buckets as bucket}
    {#if decimalPlaces > 0}
      <span>{bucket.toFixed(decimalPlaces)}</span>
    {:else}
      <span>{bucket}</span>
    {/if}
  {/each}
</div>
