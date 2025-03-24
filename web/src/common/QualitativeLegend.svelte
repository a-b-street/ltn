<script lang="ts">
  export let labelColors: Record<string, string>;
  export let swatchClass: "rectangle" | "circle" = "rectangle";
  export let itemsPerRow: 1 | 2 | 3 | 4 = 2;
  let gridItemWidth = `${100 / itemsPerRow}%`;
</script>

<div
  style="display: grid; grid-template-columns: repeat(auto-fill, calc({gridItemWidth} - 10px)); gap: 10px;"
>
  {#each Object.entries(labelColors) as [label, color]}
    <div style="display: flex; align-items: center; gap: 6px;">
      <span class="color-swatch {swatchClass}" style:background={color} />
      {label}
    </div>
  {/each}
</div>

<style>
  .color-swatch.circle {
    height: 20px;
    width: 20px;
    border-radius: 10px;
    border: 1px solid black;
  }
  .color-swatch.rectangle {
    height: 16px;
    width: 24px;
    border: 1px solid #888;
  }
</style>
