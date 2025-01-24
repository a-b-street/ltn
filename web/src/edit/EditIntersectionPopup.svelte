<script lang="ts">
  import { Intersection } from "../common/Intersection";

  export let intersection: Intersection;
</script>

<div style="display: flex; align-items: center;">
  {#if intersection.filter}
    {#if intersection.hasAlreadyRotatedFilter}
      <div class="icon">❌</div>
      <img
        src={`${import.meta.env.BASE_URL}/filters/diagonal_filter_icon.png`}
        alt="Remove diagonal filter"
        style="transform: rotate(45deg);"
      />
      Delete diagonal filter
    {:else}
      <div style="font-size: 200%; padding-bottom: 4px;" class="icon">↻</div>
      <img
        src={`${import.meta.env.BASE_URL}/filters/diagonal_filter_icon.png`}
        alt="Rotate diagonal filter"
        style="transform: rotate(45deg);"
      />
      Rotate diagonal filter
    {/if}
  {:else}
    <div style="font-size: 200%; margin-top: -12px;" class="icon">+</div>
    <img
      src={`${import.meta.env.BASE_URL}/filters/diagonal_filter_icon.png`}
      alt="Add diagonal filter"
      style="transform: rotate(135deg);"
    />
    Add diagonal filter
  {/if}
</div>

<style>
  img {
    width: 40px;
  }
</style>
