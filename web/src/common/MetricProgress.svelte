<script lang="ts">
  import { ArrowUp } from "lucide-svelte";

  export let colorScale: string[];
  export let limits: number[];
  export let value: number;

  $: bucketIdx = calculateBucket(limits, value);

  function calculateBucket(limits: number[], value: number): number {
    // Note the value might exceed the highest limit; it winds up in the last bucket if so
    let idx = 0;
    for (let limit of limits.slice(1, -1)) {
      if (value < limit) {
        break;
      }
      idx++;
    }
    return idx;
  }
</script>

<div class="colors">
  {#each colorScale as color, idx}
    <div class="bucket" class:selected={idx == bucketIdx}>
      <div class="bucket-color" style:background={color}></div>
      <div class="bucket-indicator">
        <ArrowUp strokeWidth="4px" size="14px" />
      </div>
    </div>
  {/each}
</div>

<style>
  .colors {
    display: flex;
  }

  .bucket {
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .bucket .bucket-color {
    height: 20px;
  }
  .bucket .bucket-indicator {
    display: none;
    justify-content: center;
  }
  .bucket.selected .bucket-indicator {
    display: flex;
  }
</style>
