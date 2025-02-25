<script lang="ts">
  import NewModal from "./common/NewModal.svelte";
  import icon from "../assets/settings.svg?url";
  import { BasemapPicker } from "./common";
  import { thickRoadsForShortcuts } from "./stores";

  let show = false;
</script>

<button class="outline" on:click={() => (show = true)}>
  <img src={icon} alt="Settings" />
</button>

  <NewModal bind:show={show}>
    <h1>Settings</h1>

    <BasemapPicker />

    <label>
      <input type="checkbox" bind:checked={$thickRoadsForShortcuts} />
      Road thickness depends on shortcuts
    </label>

    <center><button on:click={() => (show = false)}>Confirm</button></center>
  </NewModal>
