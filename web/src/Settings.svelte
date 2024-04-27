<script lang="ts">
  import { BasemapPicker } from "./common";
  import { Modal } from "svelte-utils";
  import { lightMode } from "./stores";
  import settingsLight from "../assets/settings_light.svg?url";
  import settingsDark from "../assets/settings_dark.svg?url";

  let show = false;

  $: document.documentElement.setAttribute(
    "data-theme",
    $lightMode ? "light" : "dark",
  );
</script>

<button class="outline" on:click={() => (show = true)}>
  <img src={$lightMode ? settingsLight : settingsDark} alt="Settings" />
</button>

{#if show}
  <Modal on:close={() => (show = false)}>
    <h1>Settings</h1>
    <BasemapPicker />
    <label>
      <input type="checkbox" role="switch" bind:checked={$lightMode} />
      Light mode
    </label>
    <center><button on:click={() => (show = false)}>Confirm</button></center>
  </Modal>
{/if}
