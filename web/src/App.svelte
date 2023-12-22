<script lang="ts">
  import turfBbox from "@turf/bbox";
  import { LTN } from "backend";
  import type { Feature, Polygon } from "geojson";
  import type { Map } from "maplibre-gl";
  import { MapLibre } from "svelte-maplibre";
  import { Layout } from "./common";
  import { RouteTool } from "./common/route_tool";
  import RouteSnapperLayer from "./common/RouteSnapperLayer.svelte";
  import MapLoader from "./MapLoader.svelte";
  import NeighbourhoodMode from "./NeighbourhoodMode.svelte";
  import NetworkMode from "./NetworkMode.svelte";
  import { mapContents, sidebarContents } from "./stores";
  import ViewShortcutsMode from "./ViewShortcutsMode.svelte";

  let offlineMode = true;
  let mapStyle = offlineMode
    ? {
        version: 8,
        sources: {},
        layers: [],
      }
    : "https://api.maptiler.com/maps/dataviz/style.json?key=MZEJTanw3WpxRvt7qDfo";

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
      }
    | {
        mode: "view-shortcuts";
        prevMode: Mode;
      };

  let mode = {
    mode: "network",
  };
  let app: LTN | undefined = undefined;
  let route_tool: RouteTool | undefined = undefined;
  let map: Map;

  function zoomToFit() {
    if (map && app) {
      // TODO wasteful
      let bbox = turfBbox(JSON.parse(app.render()));
      map.fitBounds(bbox, { animate: false });
    }
  }

  function gotApp(_x: LTN) {
    if (!app) {
      return;
    }
    console.log("New map model loaded");
    zoomToFit();
    mode = {
      mode: "network",
    };
    route_tool = new RouteTool(map, app.toRouteSnapper());
  }
  $: gotApp(app);

  // TODO Move this somewhere else
  function setBoundaryMode() {
    if (mode.mode == "network") {
      route_tool.startArea();
    } else if (mode.mode == "neighbourhood") {
      route_tool.editExistingArea(mode.boundary);
    }

    mode = {
      mode: "set-boundary",
    };
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

  let sidebarDiv;
  let mapDiv;
  $: if (sidebarDiv && $sidebarContents) {
    sidebarDiv.innerHTML = "";
    sidebarDiv.appendChild($sidebarContents);
  }
  $: if (mapDiv && $mapContents) {
    mapDiv.innerHTML = "";
    mapDiv.appendChild($mapContents);
  }
</script>

<Layout>
  <div slot="left">
    {#if map}
      <MapLoader {map} bind:app />
    {/if}
    <div><button on:click={zoomToFit}>Zoom to fit</button></div>

    {#if mode.mode == "set-boundary"}
      <p>Draw the boundary...</p>
    {/if}

    <div bind:this={sidebarDiv} />
  </div>
  <div slot="main" style="position:relative; width: 100%; height: 100vh;">
    <MapLibre style={mapStyle} standardControls hash bind:map>
      {#if app}
        {#if mode.mode == "set-boundary"}
          <RouteSnapperLayer />
        {/if}
      {/if}

      <div bind:this={mapDiv} />
      {#if app}
        {#if mode.mode == "network"}
          <NetworkMode {app} bind:mode {setBoundaryMode} />
        {:else if mode.mode == "neighbourhood"}
          <NeighbourhoodMode
            {map}
            {app}
            boundary={mode.boundary}
            {offlineMode}
            bind:mode
            {setBoundaryMode}
          />
        {:else if mode.mode == "view-shortcuts"}
          <ViewShortcutsMode bind:mode {app} prevMode={mode.prevMode} {map} />
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
