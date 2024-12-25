<script lang="ts">
  // This launches a modal when clicked. The user should put the modal contents
  // as a slot beneath this component.
  import { Modal } from "svelte-utils";
  import icon from "../../assets/help.svg?url";

  let show = false;
</script>

<button on:click={() => (show = true)}>
  <img src={icon} title="Help" alt="Help" />
</button>

{#if show}
  <Modal on:close={() => (show = false)}>
    <h2>Help</h2>
    <slot />
  </Modal>
{/if}

<style>
  button {
    background: none;
    border: 1px solid rgba(0, 0, 0, 0);
  }

  button:hover {
    border: 1px solid black;
  }
</style>
