<script lang="ts">
  import { Modal } from "svelte-utils";
  import icon from "../assets/settings.svg?url";
  import { BasemapPicker } from "./common";
  import { thickRoadsForShortcuts } from "./stores";

  let show = false;
</script>

<button class="outline" on:click={() => (show = true)}>
  <img src={icon} alt="Settings" />
</button>

<Modal bind:show>
  <h1>Settings</h1>

  <BasemapPicker />

  <label>
    <input type="checkbox" bind:checked={$thickRoadsForShortcuts} />
    Road thickness depends on shortcuts
  </label>

  <center><button on:click={() => (show = false)}>Confirm</button></center>
</Modal>
