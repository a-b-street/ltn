<script lang="ts">
  import { onMount } from "svelte";
  import { GeoJSON, LineLayer } from "svelte-maplibre";
  import { constructMatchExpression } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import BackButton from "./BackButton.svelte";
  import {
    DotMarker,
    layerId,
    Link,
    prettyPrintDistance,
    prettyPrintTime,
  } from "./common";
  import {
    CellLayer,
    HighlightBoundaryLayer,
    ModalFilterLayer,
    NeighbourhoodRoadLayer,
    OneWayLayer,
    RenderNeighbourhood,
  } from "./layers";
  import {
    backend,
    ensurePointInVisibleBounds,
    mainRoadPenalty,
    mode,
    returnToChooseProject,
    routePtA,
    routePtB,
  } from "./stores";

  export let prevMode:
    | "pick-neighbourhood"
    | "neighbourhood"
    | "impact-one-destination";

  $: gj = $backend!.compareRoute($routePtA, $routePtB, $mainRoadPenalty);

  onMount(() => {
    // There seems to be a race with the Marker component, so we wait just a bit before updating.
    setTimeout(() => {
      ensurePointInVisibleBounds(routePtA);
      ensurePointInVisibleBounds(routePtB);
    }, 10);
  });

  function back() {
    $mode = { mode: prevMode };
  }
</script>

<SplitComponent>
  <div slot="top">
    <nav aria-label="breadcrumb">
      <ul>
        <li>
          <Link on:click={returnToChooseProject}>Choose project</Link>
        </li>
        <li>
          <Link on:click={() => ($mode = { mode: "pick-neighbourhood" })}>
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
    {#if gj.features.length == 2}
      <p>
        <span style="color: red">Route before</span>
        : {prettyPrintDistance(gj.features[0].properties.distance)}, {prettyPrintTime(
          gj.features[0].properties.time,
        )}
      </p>
      <p>
        <span style="color: blue">Route after</span>
        : {prettyPrintDistance(gj.features[1].properties.distance)}, {prettyPrintTime(
          gj.features[1].properties.time,
        )}
      </p>
    {/if}

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
        <NeighbourhoodRoadLayer interactive={false} />
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

    <DotMarker bind:lngLat={$routePtA} draggable>A</DotMarker>
    <DotMarker bind:lngLat={$routePtB} draggable>B</DotMarker>
  </div>
</SplitComponent>
