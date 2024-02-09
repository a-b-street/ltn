<script lang="ts">
  import { BasemapPicker, Modal } from "./common";
  import settings from "../assets/settings.svg?url";

  let show = false;
</script>

<button class="outline" on:click={() => (show = true)}>
  <img src={settings} alt="Settings" />
</button>

{#if show}
  <Modal on:close={() => (show = false)}>
    <h1>Settings</h1>
    <BasemapPicker />
    <center><button on:click={() => (show = false)}>Confirm</button></center>
  </Modal>
{/if}
