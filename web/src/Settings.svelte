<script lang="ts">
  import { Modal } from "svelte-utils";
  import icon from "../assets/settings.svg?url";
  import { BasemapPicker } from "./common";
  import { roadStyle } from "./stores";

  let show = false;
</script>

<button class="outline" on:click={() => (show = true)}>
  <img src={icon} alt="Settings" />
</button>

{#if show}
  <Modal on:close={() => (show = false)}>
    <h1>Settings</h1>

    <BasemapPicker />

    <label>
      Style for roads in a neighbourhood:
      <select bind:value={$roadStyle}>
        <option value="shortcuts">Show worst shortcuts</option>
        <option value="cells">Show the cell</option>
      </select>
    </label>

    <center><button on:click={() => (show = false)}>Confirm</button></center>
  </Modal>
{/if}
