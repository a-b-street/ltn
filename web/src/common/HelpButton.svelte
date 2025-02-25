<script lang="ts">
  // This launches a modal when clicked. The user should put the modal contents
  // as a slot beneath this component.
  import { CircleHelp } from "lucide-svelte";
  import NewModal from "./NewModal.svelte";

  let show = false;
  export let color = "black";
</script>

<button class="icon-btn help" aria-label="Help" on:click|stopPropagation={() => (show = true)}>
  <CircleHelp {color} />
</button>

<NewModal bind:show={show}>
  <h2>Help</h2>
  <slot />
</NewModal>

<style>
  button.help {
    background: none;
    border: none;
  }

  /* 
  special considerations so that the help button doesn't exapnd the
  height of the breadcrums
  */
  :global(.pico nav[aria-label="breadcrumb"] ul > li) button.help {
    height: 30px;
    padding: 2px;
    width: auto;
  }
</style>
