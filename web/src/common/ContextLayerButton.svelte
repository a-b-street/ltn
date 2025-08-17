<script lang="ts">
  import { HelpButton } from "../common";

  export let label: string;
  export let show = false;
  export let onChange: () => void = () => {};
</script>

<button
  class="context-control"
  onclick={() => {
    show = !show;
    onChange();
  }}
>
  <input
    style="aspect-ratio: 1.0"
    type="checkbox"
    bind:checked={show}
    on:change={onChange}
  />
  {label}
  {#if $$slots.help}
    <span style="margin-left: auto"
      ><HelpButton>
        <div class="context-layer-help-content">
          <slot name="help" />
        </div>
      </HelpButton></span
    >
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
    color: black;
    padding: 8px;
  }

  .context-control {
    display: flex;
    align-items: center;
    justify-content: leading;
    gap: 8px;
    text-align: left;
  }
</style>
