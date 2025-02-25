<script lang="ts">
  import { HelpButton } from "../common";

  export let label: string;
  export let show = false;
</script>

<button
  style="display: flex; align-items: center; justify-content: leading; gap: 16px;"
  class="secondary"
  on:click={() => (show = !show)}
>
  <input type="checkbox" bind:checked={show} />
  {label}
  {#if $$slots.help}
    <HelpButton color="var(--pico-secondary-inverse)">
      <div class="context-layer-help-content">
        <slot name="help" />
      </div>
    </HelpButton>
  {/if}
</button>

<style>
  :global(.pico .context-layer-help-content p) {
    color: black;
  }
</style>
