<script lang="ts">
  export let colorScale;
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
    <span class="bucket" class:fits={idx == bucketIdx} style:background={color}
      >&nbsp;</span
    >
  {/each}
</div>

<style>
  .colors {
    display: flex;
    justify-content: space-around;
    height: 20px;
  }

  .colors .bucket {
    flex: 1;
  }

  .fits {
    border: 3px solid red;
  }
</style>
