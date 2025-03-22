<script lang="ts">
  import onewayArrowUrl from "../assets/arrow.png?url";
  import logo from "../assets/logo.svg?url";
  import nationalRailUrl from "../assets/national_rail.png?url";
  import About from "./About.svelte";
  import ContextualLayers from "./context/ContextualLayers.svelte";
  import "@picocss/pico/css/pico.conditional.jade.min.css";
  import initLtn from "backend";
  import type { LngLatBoundsLike, Map, StyleSpecification } from "maplibre-gl";
  import { init as initRouteSnapper } from "route-snapper-ts";
  import { onMount } from "svelte";
  import {
    FillLayer,
    GeoJSON,
    MapLibre,
    NavigationControl,
    ScaleControl,
  } from "svelte-maplibre";
  import { notNull } from "svelte-utils";
  import { Geocoder } from "svelte-utils/map";
  import {
    Layout,
    mapContents,
    sidebarContents,
    topContents,
  } from "svelte-utils/top_bar_layout";
  import streetsMapStyleUrl from "../assets/map-styles/streets-v2-style.json?url";
  import AddNeighbourhoodMode from "./AddNeighbourhoodMode.svelte";
  import { DisableInteractiveLayers, layerId, StreetView } from "./common";
  import DebugDemandMode from "./DebugDemandMode.svelte";
  import DebugIntersectionsMode from "./DebugIntersectionsMode.svelte";
  import DebugNeighbourhoodMode from "./DebugNeighbourhoodMode.svelte";
  import NeighbourhoodMode from "./edit/NeighbourhoodMode.svelte";
  import ImpactDetailMode from "./ImpactDetailMode.svelte";
  import ImpactOneDestinationMode from "./ImpactOneDestinationMode.svelte";
  import PickNeighbourhoodMode from "./PickNeighbourhoodMode.svelte";
  import PredictImpactMode from "./PredictImpactMode.svelte";
  import RouteMode from "./RouteMode.svelte";
  import SetBoundaryMode from "./SetBoundaryMode.svelte";
  import Settings from "./Settings.svelte";
  import {
    appFocus as appFocusStore,
    backend,
    map as mapStore,
    maptilerApiKey,
    maptilerBasemap,
    mode,
    showAbout,
    useLocalVite,
    type AppFocus,
  } from "./stores";
  import NewProjectMode from "./title/NewProjectMode.svelte";
  import TitleMode from "./title/TitleMode.svelte";
  import ViewShortcutsMode from "./ViewShortcutsMode.svelte";

  export let appFocus: AppFocus = "global";
  appFocusStore.set(appFocus);

  let wasmReady = false;
  onMount(async () => {
    await initLtn();
    await initRouteSnapper();
    wasmReady = true;

    // When running locally if a vite public/ directory is set up, load from that for speed
    try {
      let resp = await fetch("/severance_pbfs/areas.json");
      if (resp.ok) {
        $useLocalVite = true;
        console.log("Using local cache, not od2net.org");
      }
    } catch (err) {}
  });

  let map: Map | null = null;
  $: if (map) {
    mapStore.set(map);
  }

  function zoomToFit() {
    $mapStore!.fitBounds($backend!.getBounds(), { animate: false });
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

  let initialBounds: LngLatBoundsLike | undefined = undefined;
  if (appFocus == "cnt") {
    initialBounds = [-8.943, 54.631, -0.901, 59.489];
  }
  async function getStyle(
    basemap: string,
  ): Promise<StyleSpecification | string> {
    // streets-v2 uses a fill-extrusion layer for 3D buildings that's very distracting, so we have a custom version
    // NOTE: our maptiler apiKey is baked into the downloaded style, so if we rotate keys, we'll need to regenerate this file.
    if (basemap == "streets-v2") {
      return streetsMapStyleUrl;
    } else {
      return `https://api.maptiler.com/maps/${basemap}/style.json?key=${maptilerApiKey}`;
    }
  }
</script>

<div class="pico">
  <About />
</div>
<div class="app-focus-{$appFocusStore}">
  <Layout>
    <div slot="top" class="pico" style="display: flex">
      <button class="outline" on:click={() => ($showAbout = true)}>
        <img src={logo} style="height: 6vh;" alt="A/B Street logo" />
      </button>
      <Settings />
      <span bind:this={topDiv} style="width: 100%" />
    </div>
    <div class="pico" slot="left">
      <div bind:this={sidebarDiv} />

      <hr />

      {#if $backend}
        <button class="secondary" on:click={zoomToFit}>
          Zoom to fit study area
        </button>
        <StreetView
          map={notNull($mapStore)}
          maptilerBasemap={$maptilerBasemap}
        />
      {/if}
    </div>
    <div slot="main" style="position: relative; width: 100%; height: 100%;">
      {#await getStyle($maptilerBasemap) then style}
        <MapLibre
          {style}
          hash
          bind:map
          bounds={initialBounds}
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
              id: "diagonal_filter",
              url: `${import.meta.env.BASE_URL}/filters/diagonal_filter_icon.png`,
            },
            {
              id: "no_straight_turn",
              url: `${import.meta.env.BASE_URL}/filters/no_straight_turn.png`,
            },
            {
              id: "no_left_turn",
              url: `${import.meta.env.BASE_URL}/filters/no_left_turn.png`,
            },
            {
              id: "no_right_turn",
              url: `${import.meta.env.BASE_URL}/filters/no_right_turn.png`,
            },
            {
              id: "no_u_left_to_right_turn",
              url: `${import.meta.env.BASE_URL}/filters/no_u_left_to_right_turn.png`,
            },
            {
              id: "no_u_right_to_left_turn",
              url: `${import.meta.env.BASE_URL}/filters/no_u_right_to_left_turn.png`,
            },
            {
              id: "oneway_arrow",
              url: onewayArrowUrl,
            },
            {
              id: "national_rail",
              url: nationalRailUrl,
            },
          ]}
        >
          <NavigationControl />
          <ScaleControl />
          <Geocoder {map} apiKey={maptilerApiKey} country={undefined} />

          <div bind:this={mapDiv} />

          {#if $mode.mode == "title"}
            <TitleMode {wasmReady} firstLoad={$mode.firstLoad} />
          {:else if $mode.mode == "new-project"}
            <NewProjectMode />
          {/if}

          {#if $backend}
            {#if $appFocusStore == "cnt"}
              <ContextualLayers />
            {/if}

            <GeoJSON data={$backend.getInvertedBoundary()}>
              <FillLayer
                {...layerId("boundary")}
                paint={{ "fill-color": "black", "fill-opacity": 0.3 }}
              />
            </GeoJSON>
            {#if $mode.mode == "pick-neighbourhood"}
              <PickNeighbourhoodMode />
            {:else if $mode.mode == "set-boundary"}
              <SetBoundaryMode name={$mode.name} existing={$mode.existing} />
            {:else if $mode.mode == "add-neighbourhood"}
              <AddNeighbourhoodMode />
            {:else if $mode.mode == "neighbourhood"}
              <NeighbourhoodMode />
            {:else if $mode.mode == "view-shortcuts"}
              <ViewShortcutsMode />
            {:else if $mode.mode == "impact-one-destination"}
              <ImpactOneDestinationMode />
            {:else if $mode.mode == "route"}
              <RouteMode prevMode={$mode.prevMode} />
            {:else if $mode.mode == "predict-impact"}
              <PredictImpactMode />
            {:else if $mode.mode == "impact-detail"}
              <ImpactDetailMode road={$mode.road} />
            {:else if $mode.mode == "debug-neighbourhood"}
              <DebugNeighbourhoodMode />
            {:else if $mode.mode == "debug-intersections"}
              <DebugIntersectionsMode />
            {:else if $mode.mode == "debug-demand"}
              <DebugDemandMode />
            {/if}
          {/if}
          <DisableInteractiveLayers />
        </MapLibre>
      {/await}
    </div>
  </Layout>
</div>

<style>
  :global(#app .left) {
    width: 35%;
    min-width: 400px;
    max-width: 700px;
  }
  :global(#app .main) {
    width: 65%;
  }

  :global(.pico .icon-btn.destructive),
  :global(.icon-btn.destructive),
  :global(.pico button.destructive) {
    background-color: #dc2626;
  }

  :global(.pico .icon-btn),
  :global(.icon-btn) {
    margin: 0;
    height: 36px;
    width: 36px;
    aspect-ratio: 1;
    padding: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  :global(.pico .icon-btn):hover,
  :global(.icon-btn):hover {
    background-color: #ddd;
  }

  :global(.pico .icon-btn.destructive):hover,
  :global(.icon-btn.destructive):hover {
    background-color: #891717;
  }

  :global(.pico .icon-btn svg),
  :global(.icon-btn svg) {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  :global(.pico nav[aria-label="breadcrumb"] ul) {
    margin-left: 4px;
  }

  :global(.pico nav[aria-label="breadcrumb"] ul > li) {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  :global(.pico nav[aria-label="breadcrumb"] ul li:before) {
    /* pico overrides to reconcile breadcrumb li becoming `flex` */
    display: none;
  }

  :global(.pico nav[aria-label="breadcrumb"] ul li:not(:last-child)::after) {
    /* pico overrides to reconcile breadcrumb li becoming `flex` */
    position: static;
    margin-right: -32px;
    margin-left: -16px;
  }
  :global(.pico nav[aria-label="breadcrumb"] ul li:last-child) {
    font-size: 110%;
    font-weight: bold;
    text-decoration: underline;
  }

  /**
  * Gloal Typography
  */

  :global(.left .pico h1) {
    font-size: 36px;
    margin: 0 0 10px 0;
  }

  :global(.left .pico h2) {
    font-size: 28px;
    margin: 0 0 8px 0;
  }

  :global(.left .pico h3) {
    font-size: 22px;
    margin: 0 0 6px 0;
  }

  :global(.left .pico h4) {
    font-size: 18px;
    margin: 0 0 4px 0;
  }

  /**
  * List navigation re-used for neighbourhood and project list
  */
  :global(.left .pico .navigable-list h1),
  :global(.left .pico .navigable-list h2),
  :global(.left .pico .navigable-list h3),
  :global(.left .pico .navigable-list h4) {
    padding: 0;
    margin: 0;
  }

  :global(ul.navigable-list) {
    padding: 0;
    margin-bottom: 16px;
  }

  :global(.navigable-list > li) {
    list-style: none;
    margin: 0;
    padding: 8px 8px;
  }

  :global(.navigable-list > li:not(:last-child)) {
    border-bottom: solid #ddd 1px;
  }

  :global(.left .pico .navigable-list .actionable-cell) {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
  }

  :global(.left .pico .navigable-list .actionable-cell .actions) {
    display: flex;
    gap: 16px;
  }
</style>
