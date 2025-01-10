<script lang="ts">
  import { GeoJSON, LineLayer } from "svelte-maplibre";
  import { constructMatchExpression } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import BackButton from "./BackButton.svelte";
  import { DotMarker, layerId, Link } from "./common";
  import {
    CellLayer,
    HighlightBoundaryLayer,
    InteriorRoadLayer,
    ModalFilterLayer,
    OneWayLayer,
    RenderNeighbourhood,
  } from "./layers";
  import {
    backend,
    mainRoadPenalty,
    mode,
    route_pt_a,
    route_pt_b,
  } from "./stores";

  export let prevMode: "network" | "neighbourhood" | "impact-one-destination";

  $: gj = $backend!.compareRoute($route_pt_a, $route_pt_b, $mainRoadPenalty);

  function back() {
    $mode = { mode: prevMode };
  }
</script>

<SplitComponent>
  <div slot="top">
    <nav aria-label="breadcrumb">
      <ul>
        <li>
          <Link on:click={() => ($mode = { mode: "title", firstLoad: false })}>
            Choose project
          </Link>
        </li>
        <li>
          <Link on:click={() => ($mode = { mode: "network" })}>
            Pick neighbourhood
          </Link>
        </li>
        {#if prevMode == "neighbourhood"}
          <li>
            <Link on:click={() => ($mode = { mode: "neighbourhood" })}>
              Editing
            </Link>
          </li>
        {/if}
        <li>Routing</li>
      </ul>
    </nav>
  </div>
  <div slot="sidebar">
    <BackButton on:click={back} />

    <p>Drag markers for a route</p>
    <p>
      <span style="color: red">Route before</span>
      ,
      <span style="color: blue">route after</span>
    </p>

    <label>
      Slow-down factor for main roads: {$mainRoadPenalty}
      <input
        type="range"
        bind:value={$mainRoadPenalty}
        min="1.0"
        max="5.0"
        step="0.1"
      />
    </label>
    <i>
      Increase to see how drivers may detour in heavy traffic. 1 means
      free-flow.
    </i>
  </div>

  <div slot="map">
    {#if prevMode == "neighbourhood"}
      <RenderNeighbourhood>
        <HighlightBoundaryLayer />
        <CellLayer />
        <OneWayLayer />
        <InteriorRoadLayer interactive={false} />
      </RenderNeighbourhood>
    {/if}

    <ModalFilterLayer />

    <GeoJSON data={gj}>
      <LineLayer
        {...layerId("compare-route")}
        paint={{
          "line-width": 10,
          "line-color": constructMatchExpression(
            ["get", "kind"],
            {
              before: "red",
              after: "blue",
            },
            "red",
          ),
        }}
      />
    </GeoJSON>

    <DotMarker bind:lngLat={$route_pt_a} draggable>A</DotMarker>
    <DotMarker bind:lngLat={$route_pt_b} draggable>B</DotMarker>
  </div>
</SplitComponent>
