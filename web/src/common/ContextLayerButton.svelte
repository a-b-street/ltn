<script lang="ts">
  import type { Snippet } from "svelte";
  import { HelpButton } from "../common";

  export let label: string;
  export let show = false;
  export let onChange: () => void = () => {};
  export let help: Snippet | undefined = undefined;
  export let legend: Snippet | undefined = undefined;
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
    onchange={onChange}
  />
  {label}
  {#if help}
    <span style="margin-left: auto"
      ><HelpButton>
        <div class="context-layer-help-content">
          {@render help()}
        </div>
      </HelpButton></span
    >
  {/if}
</button>

{#if show && legend}
  <div class="legend">
    {@render legend()}
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
