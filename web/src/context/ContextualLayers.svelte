<script lang="ts">
  import { Layers } from "lucide-svelte";
  import { Control } from "svelte-maplibre";
  import BusRoutes from "./BusRoutes.svelte";
  import CBD from "./CBD.svelte";
  import POIs from "./POIs.svelte";
  import Population from "./Population.svelte";
  import RailwayStations from "./RailwayStations.svelte";
  import RouteNetwork from "./RouteNetwork.svelte";
  import Stats19 from "./Stats19.svelte";

  let expand = false;
</script>

<Control defaultStyling={true}>
  <div
    class="pico contextual-layers"
    style="display: flex; flex-direction: column; max-height: 80vh; max-width: 300px; overflow: auto; border-radius: 5px;"
  >
    <button
      class="show-layers-button icon-btn {expand ? 'expanded' : ''}"
      aria-label="Layers"
      on:click={() => (expand = !expand)}
    >
      <div
        style="display: flex; gap: 8px; color: #333; align-items: center; justify-content: center;"
      >
        <Layers color="#333" />
        {#if expand}
          Layers
        {/if}
      </div>
    </button>

    <div
      style="flex-direction: column"
      style:display={expand ? "flex" : "none"}
      style:background-color="#515f7A"
    >
      <POIs />
      <Population />
      <RailwayStations />
      <BusRoutes />
      <CBD />
      <RouteNetwork />
      <Stats19 />
    </div>
  </div>
</Control>

<style>
  .pico .show-layers-button {
    background-color: #fff;
    border-color: #999;
    width: 44px;
    height: 44px;
  }
  .pico .show-layers-button.expanded {
    width: 100%;
    border-bottom-left-radius: 0;
    border-bottom-right-radius: 0;
  }
  .pico .show-layers-button:hover {
    background-color: #f2f2f2;
  }
</style>
