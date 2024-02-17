<script lang="ts">
  import { BasemapPicker, Modal } from "./common";
  import settings from "../assets/settings.svg?url";

  let show = false;

  let light = !window.matchMedia("(prefers-color-scheme: dark)").matches;
  $: document.documentElement.setAttribute(
    "data-theme",
    light ? "light" : "dark",
  );
</script>

<button class="outline" on:click={() => (show = true)}>
  <img src={settings} alt="Settings" />
</button>

{#if show}
  <Modal on:close={() => (show = false)}>
    <h1>Settings</h1>
    <BasemapPicker />
    <label>
      <input type="checkbox" role="switch" bind:checked={light} />
      Light mode
    </label>
    <center><button on:click={() => (show = false)}>Confirm</button></center>
  </Modal>
{/if}
