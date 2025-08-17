<script lang="ts">
  import borderEntryArrorUrl from "../assets/arrow-big-up.png?url";
  import onewayArrowUrl from "../assets/arrow.png?url";
  import favicon from "../assets/favicon.ico?url";
  import diagonalUrl from "../assets/filters/diagonal_filter_icon.png?url";
  import noLeftUrl from "../assets/filters/no_left_turn.png?url";
  import noRightUrl from "../assets/filters/no_right_turn.png?url";
  import noStraightUrl from "../assets/filters/no_straight_turn.png?url";
  import noUTurnRtlUrl from "../assets/filters/no_u_left_to_right_turn.png?url";
  import noUTurnLtrUrl from "../assets/filters/no_u_right_to_left_turn.png?url";
  import logo from "../assets/logo.svg?url";
  import nationalRailUrl from "../assets/national_rail.png?url";
  import About from "./About.svelte";
  import ContextualLayers from "./context/ContextualLayers.svelte";
  import "@picocss/pico/css/pico.conditional.jade.min.css";
  import { CircleHelp, House } from "lucide-svelte";
  import type { LngLatBoundsLike, Map } from "maplibre-gl";
  import * as routeSnapperPkg from "route-snapper";
  import { onMount } from "svelte";
  import {
    Control,
    ControlButton,
    ControlGroup,
    FillLayer,
    GeoJSON,
    MapLibre,
    NavigationControl,
    ScaleControl,
  } from "svelte-maplibre";
  import { Modal, notNull } from "svelte-utils";
  import { Geocoder } from "svelte-utils/map";
  import {
    Layout,
    mapContents,
    sidebarContents,
    topContents,
  } from "svelte-utils/top_bar_layout";
  import * as backendPkg from "../../backend/pkg";
  import streetsMapStyleUrl from "../assets/map-styles/streets-v2-style.json?url";
  import AddNeighbourhoodMode from "./AddNeighbourhoodMode.svelte";
  import { DisableInteractiveLayers, layerId, StreetView } from "./common";
  import { ModalFilterType } from "./common/ModalFilterType";
  import DebugDemandMode from "./DebugDemandMode.svelte";
  import DebugIntersectionsMode from "./DebugIntersectionsMode.svelte";
  import DebugNeighbourhoodMode from "./DebugNeighbourhoodMode.svelte";
  import NeighbourhoodMode from "./edit/NeighbourhoodMode.svelte";
  import ImpactDetailMode from "./ImpactDetailMode.svelte";
  import ImpactOneDestinationMode from "./ImpactOneDestinationMode.svelte";
  import PickNeighbourhoodMode from "./pick_neighbourhood/PickNeighbourhoodMode.svelte";
  import PredictImpactMode from "./PredictImpactMode.svelte";
  import RouteMode from "./RouteMode.svelte";
  import SetBoundaryMode from "./SetBoundaryMode.svelte";
  import {
    appFocus as appFocusStore,
    backend,
    map as mapStore,
    maptilerApiKey,
    maptilerBasemap,
    mode,
    useLocalVite,
    type AppFocus,
  } from "./stores";
  import NewProjectMode from "./title/NewProjectMode.svelte";
  import TitleMode from "./title/TitleMode.svelte";
  import ViewShortcutsMode from "./ViewShortcutsMode.svelte";
  import "tippy.js/dist/tippy.css";

  export let appFocus: AppFocus;
  appFocusStore.set(appFocus);

  let wasmReady = false;
  onMount(async () => {
    await backendPkg.default();
    await routeSnapperPkg.default();
    wasmReady = true;

    // When running locally if a vite public/ directory is set up, load from that for speed
    try {
      let resp = await fetch("/severance_pbfs/areas.json");
      if (resp.ok) {
        $useLocalVite = true;
        console.log("Using local asset files");
      }
    } catch (err) {}
  });

  let map: Map | null = null;
  $: if (map) {
    map.keyboard.disableRotation();
    map.dragRotate.disable();
    map.touchZoomRotate.disableRotation();
    mapStore.set(map);
  }

  function zoomToFit(animate: boolean) {
    $mapStore!.fitBounds($backend!.getBounds(), { animate });
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

  let style: string | null = null;
  $: updateStyle($maptilerBasemap);
  async function updateStyle(basemap: string) {
    // streets-v2 uses a fill-extrusion layer for 3D buildings that's very distracting, so we have a custom version
    // NOTE: our maptiler apiKey is baked into the downloaded style, so if we rotate keys, we'll need to regenerate this file.
    if (basemap == "streets-v2") {
      style = streetsMapStyleUrl;
    } else {
      style = `https://api.maptiler.com/maps/${basemap}/style.json?key=${maptilerApiKey}`;
    }
  }

  let showAbout = false;
</script>

<svelte:head>
  <link rel="icon" type="image/x-icon" href={favicon} />
</svelte:head>

<div class="pico">
  <Modal bind:show={showAbout}>
    {#if appFocus == "cnt"}
      <h1>The Connected Neighbourhoods Tool</h1>
    {:else}
      <h1>The Low-Traffic Neighbourhood (LTN) tool, v2</h1>
    {/if}

    <About />
  </Modal>
</div>
<div class="app-focus-{$appFocusStore}">
  <Layout>
    <div
      slot="top"
      class="pico"
      style="display: flex; align-items: center; gap: 8px;"
    >
      <button class="outline" onclick={() => (showAbout = true)}>
        <img src={logo} style="height: 32px;" alt="A/B Street logo" />
      </button>
      <span bind:this={topDiv} style="width: 100%"></span>
      <button
        class="icon-btn"
        title="User guide"
        onclick={() => window.open("user_guide.html", "_blank")}
      >
        <CircleHelp color="black" />
      </button>
    </div>
    <div class="pico" slot="left">
      <div bind:this={sidebarDiv}></div>
    </div>
    <div slot="main" style="position: relative; width: 100%; height: 100%;">
      {#if style}
        <MapLibre
          {style}
          hash
          bind:map
          bounds={initialBounds}
          maxZoom={19}
          onerror={(e) => {
            console.log(e.error);
          }}
          images={[
            {
              id: ModalFilterType.walkCycleOnly.filterType,
              url: ModalFilterType.walkCycleOnly.iconURL,
            },
            {
              id: ModalFilterType.noEntry.filterType,
              url: ModalFilterType.noEntry.iconURL,
            },
            {
              id: ModalFilterType.busGate.filterType,
              url: ModalFilterType.busGate.iconURL,
            },
            {
              id: ModalFilterType.schoolStreet.filterType,
              url: ModalFilterType.schoolStreet.iconURL,
            },
            {
              id: "diagonal_filter",
              url: diagonalUrl,
            },
            {
              id: "no_straight_turn",
              url: noStraightUrl,
            },
            {
              id: "no_left_turn",
              url: noLeftUrl,
            },
            {
              id: "no_right_turn",
              url: noRightUrl,
            },
            {
              id: "no_u_left_to_right_turn",
              url: noUTurnLtrUrl,
            },
            {
              id: "no_u_right_to_left_turn",
              url: noUTurnRtlUrl,
            },
            {
              id: "oneway_arrow",
              url: onewayArrowUrl,
            },
            {
              id: "border_entry_arrow",
              url: borderEntryArrorUrl,
              options: { sdf: true },
            },
            {
              id: "national_rail",
              url: nationalRailUrl,
            },
          ]}
        >
          <NavigationControl showCompass={false} />

          {#if $backend}
            <Control position="top-left">
              <ControlGroup>
                <ControlButton
                  title="Zoom to fit study area"
                  onclick={() => zoomToFit(true)}
                >
                  <div class="ltn-map-btn zoom-to-fit-btn">
                    <House />
                  </div>
                </ControlButton>
              </ControlGroup>
            </Control>
            <Control position="top-left">
              <ControlGroup>
                <StreetView
                  map={notNull($mapStore)}
                  maptilerBasemap={$maptilerBasemap}
                />
              </ControlGroup>
            </Control>
          {/if}

          <Geocoder {map} apiKey={maptilerApiKey} country={undefined} />

          <ScaleControl />

          <div bind:this={mapDiv}></div>

          {#if $mode.mode == "title"}
            <TitleMode {wasmReady} />
          {:else if $mode.mode == "new-project"}
            <NewProjectMode />
          {/if}

          <ContextualLayers />
          {#if $backend}
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
              <PredictImpactMode prevMode={$mode.prevMode} />
            {:else if $mode.mode == "impact-detail"}
              <ImpactDetailMode
                road={$mode.road}
                prevPrevMode={$mode.prevPrevMode}
              />
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
      {/if}
    </div>
  </Layout>
</div>

<style>
  :global(.top) {
    box-shadow: 0 2px 5px rgba(0, 0, 0, 0.4);
    /* so box-shadow falls onto .main (the map) and .left */
    z-index: 2;
  }

  :global(#app .left) {
    width: 30%;
    min-width: 300px;
    max-width: 450px;
    box-shadow: 2px 0 5px rgba(0, 0, 0, 0.4);
    /* so box-shadow falls onto .main (the map) */
    z-index: 1;
  }

  :global(#app .main) {
    width: auto;
    flex: 1;
  }

  :global(#app .pico button) {
    padding: 6px 14px;
  }

  :global(.pico button.icon-btn.destructive),
  :global(button.icon-btn.destructive),
  :global(.pico button.destructive) {
    background-color: #dc2626;
  }

  :global(#app .pico button.icon-btn),
  :global(#app button.icon-btn) {
    margin: 0;
    padding: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  :global(#app .pico button.outline) {
    color: black;
    border-color: black;
  }

  :global(#app .pico button.icon-btn):hover,
  :global(#app button.icon-btn):hover,
  :global(#app .pico .tool-palette button:hover) {
    background-color: #ddd;
  }

  :global(#app.pico button.icon-btn.destructive):hover,
  :global(#app button.icon-btn.destructive):hover {
    background-color: #891717;
  }

  :global(#app .pico button.icon-btn svg),
  :global(#app button.icon-btn svg) {
    height: 100%;
    width: auto;
    aspect-ratio: 1;
    object-fit: contain;
  }

  :global(#app .pico .tool-palette button) {
    margin: 0;
    background: none;
    color: black;
  }

  :global(#app .pico .tool-palette button.icon-btn) {
    padding: 6px;
    height: 100%;
    aspect-ratio: 1;
  }

  :global(#app .pico .tool-palette button.icon-btn img) {
    aspect-ratio: 1;
    height: 100%;
    width: auto;
    object-fit: contain;
  }

  :global(#app .pico .tool-palette button.active) {
    /* slightly increased border */
    border: 2px solid black;
  }

  :global(#app .pico .tool-palette button.icon-btn.active) {
    /* Slightly decreased padding to account for the slightly increased border */
    padding: 5px;
  }

  :global(#app .pico .tool-palette button.active),
  :global(#app .pico .tool-palette button.active:hover) {
    /* picocss default color is very dark */
    background: rgb(124, 190, 146);
  }

  /* Form Controls */
  :global(#app .pico [type="checkbox"] :not([role="switch"])),
  :global(#app .pico [type="radio"]) {
    width: 1em;
    height: 1em;
    margin-inline-end: 4px;
  }

  :global(#app .pico select) {
    padding: 4px 40px 4px 8px;
    margin: 8px;
  }

  :global(#app .pico nav[aria-label="breadcrumb"] ul) {
    margin-left: 4px;
  }

  :global(#app .pico nav[aria-label="breadcrumb"] ul > li) {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  :global(#app .pico nav[aria-label="breadcrumb"] ul li:before) {
    /* pico overrides to reconcile breadcrumb li becoming `flex` */
    display: none;
  }

  :global(
    #app .pico nav[aria-label="breadcrumb"] ul li:not(:last-child)::after
  ) {
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
    margin: 10px 0;
  }

  :global(.left .pico h2) {
    font-size: 28px;
    margin: 8px 0;
  }

  :global(.left .pico h3) {
    font-size: 22px;
    margin: 6px 0;
  }

  :global(.left .pico h4) {
    font-size: 18px;
    margin: 4px 0;
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
    height: 36px;
  }

  :global(.top .pico button:hover) {
    background: #ddd;
  }

  :global(#app .top .pico button) {
    background: white;
    height: 40px;
    width: 40px;
    padding: 4px;
    margin: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    border: none;
  }

  :global(.top .pico button img) {
    height: 100%;
  }
  :global(.top .pico nav ul:last-child) {
    /* PICO override - the right aligned nav was causing a little bit of a scroll past the page width */
    margin-right: 0;
  }
  :global(.top .pico nav ul li) {
    padding: 8px 8px;
  }

  :global(.ltn-map-btn svg) {
    height: 20px;
    width: auto;
    margin-top: 2px;
  }
</style>
