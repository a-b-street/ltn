<script lang="ts">
  import { onMount } from "svelte";
  import { GeoJSON, LineLayer } from "svelte-maplibre";
  import { constructMatchExpression } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import BackButton from "./BackButton.svelte";
  import {
    DotMarker,
    layerId,
    ModeLink,
    pageTitle,
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
    routePtA,
    routePtB,
    zoomToDefault,
  } from "./stores";

  export let prevMode:
    | "pick-neighbourhood"
    | "neighbourhood"
    | "impact-one-destination";

  $: gj = $backend!.compareRoute($routePtA, $routePtB, $mainRoadPenalty);
  $: routeBefore = gj.features.find((f) => f.properties.kind == "before");
  $: routeAfter = gj.features.find((f) => f.properties.kind == "after");

  onMount(() => {
    // There seems to be a race with the Marker component, so we wait just a bit before updating.
    setTimeout(() => {
      ensurePointInVisibleBounds(routePtA);
      ensurePointInVisibleBounds(routePtB);
    }, 10);
  });
</script>

<SplitComponent>
  <div slot="top">
    <nav aria-label="breadcrumb">
      <ul>
        <li>
          <ModeLink mode={{ mode: "title" }} afterLink={zoomToDefault} />
        </li>
        <li>
          <ModeLink mode={{ mode: "pick-neighbourhood" }} />
        </li>
        {#if prevMode == "neighbourhood"}
          <li>
            <ModeLink mode={{ mode: "neighbourhood" }} />
          </li>
        {/if}
        <li>{pageTitle($mode.mode)}</li>
      </ul>
    </nav>
  </div>
  <div slot="sidebar">
    <BackButton mode={{ mode: prevMode }} />

    <p>Drag markers for a route</p>

    <u style:color="red">Route before changes</u>
    {#if routeBefore}
      <p>
        {prettyPrintDistance(routeBefore.properties.distance)}, {prettyPrintTime(
          routeBefore.properties.time,
        )}
      </p>
    {:else}
      <p>
        No possible route (
        <i>This is usually a known software bug</i>
        )
      </p>
    {/if}

    <u style:color="blue">Route after changes</u>
    {#if routeAfter}
      <p>
        {prettyPrintDistance(routeAfter.properties.distance)}, {prettyPrintTime(
          routeAfter.properties.time,
        )}
      </p>
    {:else}
      <p>
        No possible route (
        <i>This is usually a known software bug</i>
        )
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

    <ModalFilterLayer interactive={false} />

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
