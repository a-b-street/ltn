<script lang="ts">
  import { Modal } from "svelte-utils";
  import icon from "../assets/settings.svg?url";
  import { BasemapPicker } from "./common";

  let show = false;
</script>

<button class="outline" on:click={() => (show = true)}>
  <img src={icon} alt="Settings" />
</button>

{#if show}
  <Modal on:close={() => (show = false)}>
    <h1>Settings</h1>
    <BasemapPicker />
    <center><button on:click={() => (show = false)}>Confirm</button></center>
  </Modal>
{/if}
