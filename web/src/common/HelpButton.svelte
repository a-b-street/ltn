<script lang="ts">
  // This launches a modal when clicked. The user should put the modal contents
  // as a slot beneath this component.
  import { CircleHelp } from "lucide-svelte";
  import { Modal } from "svelte-utils";

  let show = false;
  export let color = "black";
</script>

<button
  class="icon-btn help"
  title="Help"
  on:click|stopPropagation={() => (show = true)}
>
  <CircleHelp {color} />
</button>

<Modal bind:show>
  <h2>Help</h2>
  <slot />
</Modal>

<style>
  button.help {
    background: none;
    border: none;
  }

  /* 
  special considerations so that the help button doesn't expand the
  height of the breadcrumbs
  */
  :global(.pico nav[aria-label="breadcrumb"] ul > li) button.help {
    height: 30px;
    padding: 2px;
    width: auto;
  }
</style>
