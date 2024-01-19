<script lang="ts">
  import { Modal, notNull } from "./common";
  import PolygonToolLayer from "./common/draw_polygon/PolygonToolLayer.svelte";
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
        <h1>The Low-traffic neighbourhood (LTN) tool, v2</h1>
        <p>
          This is an <b>experimental</b> version of the
          <a
            href="https://a-b-street.github.io/docs/software/ltn/index.html"
            target="_blank">A/B Street LTN tool</a
          >. Most parts of it do not work yet, and you should probably use the
          other version of the tool instead.
        </p>
        <p>To use this tool, you need to:</p>
        <ol>
          <li>Choose your study area to analyze</li>
          <li>Define one or more neighbourhood boundaries</li>
          <li>
            Optionally, to fix the basemap data on existing modal filters, turn
            restrictions, one-ways, etc
          </li>
          <li>
            To create one or more proposals with new modal filters, and explore
            their effects
          </li>
        </ol>
        <p>
          This tool is created by <a
            href="https://github.com/dabreegster/"
            target="_blank">Dustin Carlino</a
          >
          and relies heavily on
          <a href="https://www.openstreetmap.org/about" target="_blank"
            >OpenStreetMap</a
          > data.
        </p>
        <center
          ><button on:click={() => notNull(dialog).close()}>Start!</button
          ></center
        >
      </Modal>
    {/if}

    <h1>Choose your study area</h1>
    <button on:click={() => (showModal = true)}>About the LTN tool</button>
    <hr />

    {#if $map && wasmReady}
      <MapLoader />
    {:else}
      <p>Waiting for MapLibre and WASM to load...</p>
    {/if}
  </div>

  <div slot="map">
    <PolygonToolLayer />
  </div>
</SplitComponent>
