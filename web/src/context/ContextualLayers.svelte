<script lang="ts">
  import { Layers } from "lucide-svelte";
  import { Control } from "svelte-maplibre";
  import { ContextLayerButton } from "../common";
  import {
    appFocus,
    backend,
    maptilerBasemap,
    showExistingFiltersAndTRs,
  } from "../stores";
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
    style="display: flex; flex-direction: column; max-height: 80vh; overflow: auto; border-radius: 5px;"
    style:width={expand ? "300px" : "auto"}
  >
    <button
      class="show-layers-button icon-btn {expand ? 'expanded' : ''}"
      aria-label="Layers"
      on:click={() => (expand = !expand)}
    >
      <div
        style="display: flex; gap: 12px; color: #333; align-items: center; justify-content: center;"
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
      {#if $backend}
        <ContextLayerButton
          label="Existing modal filters and turn restrictions"
          bind:show={$showExistingFiltersAndTRs}
        />
        {#if $appFocus == "cnt"}
          <POIs />
          <Population />
          <RailwayStations />
          <BusRoutes />
          <CBD />
          <RouteNetwork />
          <Stats19 />
        {/if}
      {/if}
      <span
        style="font-size: 20px; color: var(--pico-secondary-inverse); margin-left: 8px; margin-top: 4px;"
      >
        Basemap
      </span>
      <select
        style="width: auto; font-size: 20px; margin: 8px; padding: 8px; text-overflow: ellipsis;"
        bind:value={$maptilerBasemap}
      >
        <option value="dataviz">MapTiler Dataviz</option>
        <option value="streets-v2">MapTiler Streets</option>
        <option value="hybrid">MapTiler Satellite</option>
        <option value="uk-openzoomstack-light">OS Open Zoomstack</option>
      </select>
    </div>
  </div>
</Control>

<style>
  .pico button.show-layers-button {
    background-color: #fff;
    border-color: #999;
    width: 44px;
    height: 44px;
  }
  .pico button.show-layers-button.expanded {
    width: 100%;
    border-bottom-left-radius: 0;
    border-bottom-right-radius: 0;
  }
  .pico button.show-layers-button:hover {
    background-color: #f2f2f2;
  }
  :global(.pico.contextual-layers button) {
    padding: 8px 8px;
  }
</style>
