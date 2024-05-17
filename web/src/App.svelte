<script lang="ts">
  import About from "./About.svelte";
  import onewayArrowUrl from "../assets/arrow.png?url";
  import logoLight from "../assets/logo_light.svg?url";
  import logoDark from "../assets/logo_dark.svg?url";
  import "@picocss/pico/css/pico.jade.min.css";
  import initLtn from "backend";
  import type { Map } from "maplibre-gl";
  import { init as initRouteSnapper } from "route-snapper-ts";
  import { onMount } from "svelte";
  import { FillLayer, GeoJSON, MapLibre } from "svelte-maplibre";
  import { notNull } from "svelte-utils";
  import { Geocoder } from "svelte-utils/map";
  import {
    DisableInteractiveLayers,
    Layout,
    layerId,
    StreetView,
  } from "./common";
  import DebugMode from "./DebugMode.svelte";
  import NeighbourhoodMode from "./edit/NeighbourhoodMode.svelte";
  import NetworkMode from "./NetworkMode.svelte";
  import RouteMode from "./RouteMode.svelte";
  import Settings from "./Settings.svelte";
  import SetBoundaryMode from "./SetBoundaryMode.svelte";
  import {
    app,
    mapContents,
    topContents,
    map as mapStore,
    showAbout,
    lightMode,
    mode,
    sidebarContents,
    maptilerBasemap,
    maptilerApiKey,
    useLocalVite,
  } from "./stores";
  import TitleMode from "./title/TitleMode.svelte";
  import NewProjectMode from "./title/NewProjectMode.svelte";
  import ViewShortcutsMode from "./ViewShortcutsMode.svelte";

  let wasmReady = false;
  onMount(async () => {
    await initLtn();
    await initRouteSnapper();
    wasmReady = true;

    // When running locally if a vite public/ directory is set up, load from that for speed
    try {
      let resp = await fetch("/osm/areas.json");
      if (resp.ok) {
        $useLocalVite = true;
        console.log("Using local cache, not od2net.org");
      }
    } catch (err) {}
  });

  let map: Map;
  $: if (map) {
    mapStore.set(map);
  }

  function zoomToFit() {
    $mapStore!.fitBounds(
      Array.from($app!.getBounds()) as [number, number, number, number],
      { animate: false },
    );
  }

  let topDiv: HTMLSpanElement;
  let sidebarDiv: HTMLDivElement;
  let mapDiv: HTMLDivElement;
  $: if (topDiv && $topContents) {
    topDiv.innerHTML = "";
    topDiv.appendChild($topContents);
  }
  $: if (sidebarDiv && $sidebarContents) {
    sidebarDiv.innerHTML = "";
    sidebarDiv.appendChild($sidebarContents);
  }
  $: if (mapDiv && $mapContents) {
    mapDiv.innerHTML = "";
    mapDiv.appendChild($mapContents);
  }
</script>

<About />
<Layout>
  <div slot="top" style="display: flex">
    <button class="outline" on:click={() => ($showAbout = true)}>
      <img
        src={$lightMode ? logoLight : logoDark}
        style="height: 6vh;"
        alt="A/B Street logo"
      />
    </button>
    <Settings />
    <span bind:this={topDiv} style="width: 100%" />
  </div>
  <div slot="left">
    <div bind:this={sidebarDiv} />

    <hr />

    {#if $app}
      <button class="secondary" on:click={zoomToFit}>
        Zoom to fit study area
      </button>
      <StreetView map={notNull($mapStore)} maptilerBasemap={$maptilerBasemap} />
    {/if}
  </div>
  <div slot="main" style="position: relative; width: 100%; height: 100%;">
    <MapLibre
      style={`https://api.maptiler.com/maps/${$maptilerBasemap}/style.json?key=${maptilerApiKey}`}
      standardControls
      hash
      bind:map
      on:error={(e) => {
        // @ts-expect-error ErrorEvent isn't exported
        console.log(e.detail.error);
      }}
      images={[
        {
          id: "walk_cycle_only",
          url: `${import.meta.env.BASE_URL}/filters/walk_cycle_only_icon.gif`,
        },
        {
          id: "no_entry",
          url: `${import.meta.env.BASE_URL}/filters/no_entry_icon.gif`,
        },
        {
          id: "bus_gate",
          url: `${import.meta.env.BASE_URL}/filters/bus_gate_icon.gif`,
        },
        {
          id: "school_street",
          url: `${import.meta.env.BASE_URL}/filters/school_street_icon.gif`,
        },
        {
          id: "oneway-arrow",
          url: onewayArrowUrl,
        },
      ]}
    >
      <Geocoder {map} apiKey={maptilerApiKey} />
      <div bind:this={mapDiv} />
      {#if $mode.mode == "title"}
        <TitleMode {wasmReady} />
      {:else if $mode.mode == "new-project"}
        <NewProjectMode />
      {/if}
      {#if $app}
        <GeoJSON data={JSON.parse($app.getInvertedBoundary())}>
          <FillLayer
            {...layerId("boundary")}
            paint={{ "fill-color": "black", "fill-opacity": 0.3 }}
          />
        </GeoJSON>
        {#if $mode.mode == "network"}
          <NetworkMode />
        {:else if $mode.mode == "set-boundary"}
          <SetBoundaryMode name={$mode.name} existing={$mode.existing} />
        {:else if $mode.mode == "neighbourhood"}
          <NeighbourhoodMode />
        {:else if $mode.mode == "view-shortcuts"}
          <ViewShortcutsMode />
        {:else if $mode.mode == "route"}
          <RouteMode prevMode={$mode.prevMode} />
        {:else if $mode.mode == "debug"}
          <DebugMode />
        {/if}
      {/if}
      <DisableInteractiveLayers />
    </MapLibre>
  </div>
</Layout>
