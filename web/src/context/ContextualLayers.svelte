<script lang="ts">
  import { Layers } from "lucide-svelte";
  import { Control } from "svelte-maplibre";
  import { ContextLayerButton } from "../common";
  import {
    appFocus,
    backend,
    devMode,
    maptilerBasemap,
    showExistingFiltersAndTRs,
  } from "../stores";
  import BusRoutes from "./BusRoutes.svelte";
  import CBD from "./CBD.svelte";
  import POIs from "./POIs.svelte";
  import Population from "./Population.svelte";
  import RailwayStations from "./RailwayStations.svelte";
  import Stats19 from "./Stats19.svelte";

  let expand = false;
</script>

<Control defaultStyling>
  <div
    class="pico contextual-layers"
    style="display: flex; flex-direction: column; border-radius: 5px;"
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
      class="contextual-layer-controls"
      style="flex-direction: column;"
      style:display={expand ? "flex" : "none"}
    >
      {#if $backend}
        <ContextLayerButton
          label="Existing modal filters and turn restrictions"
          bind:show={$showExistingFiltersAndTRs}
        />
        {#if $appFocus == "cnt"}
          <div class="layer-group">Metrics</div>
          <Population />
          <POIs />
          <Stats19 />

          <div class="layer-group">Public transport integration</div>
          <RailwayStations />
          <BusRoutes />

          <div class="layer-group">Active travel</div>
          <CBD />
        {/if}
      {/if}

      <div class="context-control">
        <span style="font-size: 20px; margin-left: 8px; margin-top: 4px;">
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

        <ContextLayerButton label="Debugging tools" bind:show={$devMode} />
      </div>
    </div>
  </div>
</Control>

<style>
  .contextual-layers {
    background: white;
    box-shadow: 0 0 10px rgba(0, 0, 0, 0.2);
    border-radius: 5px;
  }
  button.show-layers-button {
    background-color: #fff;
    border-radius: 5px;
    border-color: #999;
    width: 44px;
    height: 44px;
    box-shadow: 0 2px 5px rgba(0, 0, 0, 0.2);
  }
  button.show-layers-button:hover {
    background-color: #eee;
  }
  button.show-layers-button.expanded {
    width: 100%;
    background-color: #f2f2f2;
    border-bottom: none;
    border-bottom-right-radius: 0;
    border-bottom-left-radius: 0;
  }
  .contextual-layer-controls {
    max-height: calc(100vh - 160px);
    overflow: auto;
  }
  :global(.pico.contextual-layers .context-control) {
    border: none;
    border-radius: 0;
    color: black;
    background: none;
    padding: 8px 8px;
  }

  :global(.pico.contextual-layers .context-control:not(:first-child)) {
    border-top: solid #ddd 1px;
  }

  :global(.layer-group) {
    font-size: 1rem;
    font-weight: bold;
    padding: 4px;
  }
</style>
