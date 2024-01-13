<script lang="ts">
  import { Modal, notNull } from "./common";
  import MapLoader from "./MapLoader.svelte";
  import SplitComponent from "./SplitComponent.svelte";
  import { map } from "./stores";

  export let wasmReady: boolean;

  // TODO Once per session
  let showModal = true;
</script>

<SplitComponent>
  <div slot="sidebar">
    {#if showModal}
      <Modal on:close={() => (showModal = false)} let:dialog>
        <h1>The LTN tool</h1>
        <p>...</p>
        <center
          ><button on:click={() => notNull(dialog).close()}>Start!</button
          ></center
        >
      </Modal>
    {/if}

    <button on:click={() => (showModal = true)}>About the LTN tool</button>
    {#if $map && wasmReady}
      <MapLoader />
    {:else}
      <p>Waiting for MapLibre and WASM to load...</p>
    {/if}
  </div>

  <div slot="map" />
</SplitComponent>
