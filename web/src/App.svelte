<script lang="ts">
  import turfBbox from "@turf/bbox";
  import { MapModel } from "backend";
  import type { Feature, Polygon } from "geojson";
  import type { Map } from "maplibre-gl";
  import { MapLibre } from "svelte-maplibre";
  import { Layout } from "./common";
  import { RouteTool } from "./common/route_tool";
  import RouteSnapperLayer from "./common/RouteSnapperLayer.svelte";
  import MapLoader from "./MapLoader.svelte";
  import NeighbourhoodLayer from "./NeighbourhoodLayer.svelte";
  import NetworkLayer from "./NetworkLayer.svelte";

  type Mode =
    | {
        mode: "network";
      }
    | {
        mode: "set-boundary";
      }
    | {
        mode: "neighbourhood";
        boundary: Feature<Polygon>;
      };

  let mode = {
    mode: "network",
  };
  let model: MapModel | undefined = undefined;
  let route_tool: RouteTool | undefined = undefined;
  let map: Map;

  function zoomToFit() {
    if (map && model) {
      // TODO wasteful
      let bbox = turfBbox(JSON.parse(model.render()));
      map.fitBounds(bbox, { animate: false });
    }
  }

  function gotModel(_m: MapModel) {
    if (!model) {
      return;
    }
    console.log("New map model loaded");
    zoomToFit();
    mode = {
      mode: "network",
    };
    route_tool = new RouteTool(map, model.toRouteSnapper());
  }
  $: gotModel(model);

  function setBoundaryMode() {
    mode = {
      mode: "set-boundary",
    };
    route_tool.startArea();
    route_tool.addEventListenerSuccess((feature) => {
      mode = {
        mode: "neighbourhood",
        boundary: feature,
      };
      route_tool.clearEventListeners();
    });
    route_tool.addEventListenerFailure(() => {
      mode = {
        mode: "network",
      };
      route_tool.clearEventListeners();
    });
  }

  function reset() {
    mode = {
      mode: "network",
    };
  }
</script>

<Layout>
  <div slot="left">
    {#if map}
      <MapLoader {map} bind:model />
    {/if}
    <div><button on:click={zoomToFit}>Zoom to fit</button></div>

    {#if mode.mode == "network" && model}
      <button on:click={setBoundaryMode}>Set boundary</button>
    {:else if mode.mode == "set-boundary"}
      <p>Draw the boundary...</p>
    {:else if mode.mode == "neighbourhood"}
      <button on:click={reset}>Reset</button>
      <p>Analyze and edit now</p>
    {/if}
  </div>
  <div slot="main" style="position:relative; width: 100%; height: 100vh;">
    <MapLibre
      style="https://api.maptiler.com/maps/dataviz/style.json?key=MZEJTanw3WpxRvt7qDfo"
      standardControls
      hash
      bind:map
    >
      {#if model}
        {#if mode.mode == "network"}
          <NetworkLayer {model} />
        {:else if mode.mode == "set-boundary"}
          <RouteSnapperLayer />
        {:else if mode.mode == "neighbourhood"}
          <NeighbourhoodLayer {model} boundary={mode.boundary} />
        {/if}
      {/if}
    </MapLibre>
  </div>
</Layout>

<style>
  :global(body, button, input) {
    font-size: 26px;
  }
</style>
