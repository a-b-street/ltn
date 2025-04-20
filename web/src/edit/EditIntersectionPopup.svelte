<script lang="ts">
  import icon from "../../assets/filters/diagonal_filter_icon.png";
  import { Intersection } from "../common/Intersection";

  export let intersection: Intersection;
</script>

<div style="display: flex; align-items: center;">
  {#if intersection.filter}
    {#if intersection.hasRotatedFilter}
      <div class="icon">❌</div>
      <img
        src={icon}
        alt="Remove diagonal filter"
        style="transform: rotate(45deg);"
      />
      Delete diagonal filter
    {:else}
      <div style="font-size: 200%; padding-bottom: 4px;" class="icon">↻</div>
      <img
        src={icon}
        alt="Rotate diagonal filter"
        style="transform: rotate(45deg);"
      />
      Rotate diagonal filter
    {/if}
  {:else}
    <div style="font-size: 200%; margin-top: -12px;" class="icon">+</div>
    <img
      src={icon}
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
