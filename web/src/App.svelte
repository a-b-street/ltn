<script lang="ts">
  import initLtn, { LTN } from "backend";
  import type { Map } from "maplibre-gl";
  import initRouteSnapper from "route-snapper";
  import { onMount } from "svelte";
  import { FillLayer, GeoJSON, MapLibre } from "svelte-maplibre";
  import { Layout } from "./common";
  import { RouteTool } from "./common/route_tool";
  import NeighbourhoodMode from "./NeighbourhoodMode.svelte";
  import NetworkMode from "./NetworkMode.svelte";
  import RouteMode from "./RouteMode.svelte";
  import SetBoundaryMode from "./SetBoundaryMode.svelte";
  import {
    app,
    mapContents,
    map as mapStore,
    mode,
    showBasemap,
    sidebarContents,
  } from "./stores";
  import TitleMode from "./TitleMode.svelte";
  import ViewShortcutsMode from "./ViewShortcutsMode.svelte";

  let wasmReady = false;
  onMount(async () => {
    await initLtn();
    await initRouteSnapper();
    wasmReady = true;
  });

  $: mapStyle = $showBasemap
    ? "https://api.maptiler.com/maps/dataviz/style.json?key=MZEJTanw3WpxRvt7qDfo"
    : {
        version: 8 as const,
        sources: {},
        layers: [],
      };

  let map: Map;
  $: if (map) {
    mapStore.set(map);
  }

  // TODO Move stuff like this out...
  let route_tool: RouteTool | undefined = undefined;
  function zoomToFit() {
    $mapStore!.fitBounds(
      Array.from($app!.getBounds()) as [number, number, number, number],
      { animate: false }
    );
  }

  // TODO Can we make the title screen mode do this?
  function gotApp(_x: LTN | null) {
    if (!$app) {
      return;
    }
    console.log("New map model loaded");
    zoomToFit();
    $mode = {
      mode: "network",
    };
    route_tool = new RouteTool($mapStore!, $app.toRouteSnapper());
  }
  $: gotApp($app);

  let sidebarDiv: HTMLDivElement;
  let mapDiv: HTMLDivElement;
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
    <div bind:this={sidebarDiv} />

    <hr />

    {#if $app}
      <div><button on:click={zoomToFit}>Zoom to fit study area</button></div>
    {/if}
    <div>
      <label
        ><input type="checkbox" bind:checked={$showBasemap} />Show basemap</label
      >
    </div>
  </div>
  <div slot="main" style="position:relative; width: 100%; height: 100vh;">
    <MapLibre
      style={mapStyle}
      standardControls
      hash
      bind:map
      on:error={(e) => {
        // @ts-ignore ErrorEvent isn't exported
        console.log(e.detail.error);
      }}
    >
      <div bind:this={mapDiv} />
      {#if $mode.mode == "title"}
        <TitleMode {wasmReady} />
      {/if}
      {#if $app}
        <GeoJSON data={JSON.parse($app.getInvertedBoundary())}>
          <FillLayer paint={{ "fill-color": "black", "fill-opacity": 0.3 }} />
        </GeoJSON>
        {#if $mode.mode == "network"}
          <NetworkMode />
        {:else if $mode.mode == "set-boundary"}
          <SetBoundaryMode
            {route_tool}
            name={$mode.name}
            existing={$mode.existing}
          />
        {:else if $mode.mode == "neighbourhood"}
          <NeighbourhoodMode />
        {:else if $mode.mode == "view-shortcuts"}
          <ViewShortcutsMode />
        {:else if $mode.mode == "route"}
          <RouteMode />
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
