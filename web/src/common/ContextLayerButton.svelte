<script lang="ts">
  import { HelpButton } from "../common";

  export let label: string;
  export let show = false;
</script>

<button
  class="context-control"
  style="display: flex; align-items: center; justify-content: leading; gap: 8px; text-align: left;"
  on:click={() => (show = !show)}
>
  <input style="aspect-ratio: 1.0" type="checkbox" bind:checked={show} />
  {label}
  {#if $$slots.help}
    <HelpButton color="white">
      <div class="context-layer-help-content">
        <slot name="help" />
      </div>
    </HelpButton>
  {/if}
</button>
{#if show && $$slots.legend}
  <div class="legend">
    <slot name="legend" />
  </div>
{/if}

<style>
  :global(.pico .context-layer-help-content p),
  :global(.pico .context-layer-help-content ul) {
    color: black;
  }
  :global(.pico .context-layer-help-content ul) {
    padding-left: 4px;
  }
  .legend {
    background-color: white;
    color: black;
    padding: 8px;
  }
</style>
