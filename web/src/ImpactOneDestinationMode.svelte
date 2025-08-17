<script lang="ts">
  import type { Feature } from "geojson";
  import { LngLat, type MapMouseEvent } from "maplibre-gl";
  import { onMount } from "svelte";
  import { FillLayer, GeoJSON, LineLayer, MapEvents } from "svelte-maplibre";
  import {
    constructMatchExpression,
    emptyGeojson,
    Popup,
  } from "svelte-utils/map";
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
  import { ModalFilterLayer, RenderNeighbourhood } from "./layers";
  import {
    backend,
    ensurePointInVisibleBounds,
    mode,
    oneDestination,
    routePtA,
    routePtB,
  } from "./stores";
  import type { CompareRoute } from "./wasm";

  $: perRoadGj = $backend!.impactToOneDestination($oneDestination);

  let hovered: Feature | null = null;

  $: routeGj = previewRoutes(hovered);
  $: routeBefore = routeGj.features.find((f) => f.properties.kind == "before");
  $: routeAfter = routeGj.features.find((f) => f.properties.kind == "after");

  onMount(() => {
    // There seems to be a race with the Marker component, so we wait just a bit before updating.
    setTimeout(() => {
      if ($oneDestination) {
        ensurePointInVisibleBounds(oneDestination);
      }
    }, 10);
  });

  function previewRoutes(hovered: Feature | null): CompareRoute {
    if (!hovered) {
      return emptyGeojson() as CompareRoute;
    }
    return $backend!.compareRoute(
      new LngLat(hovered.properties!.pt1_x, hovered.properties!.pt1_y),
      $oneDestination,
      1.0,
    );
  }

  function compareRoute(f: Feature) {
    $routePtA = new LngLat(f.properties!.pt1_x, f.properties!.pt1_y);
    $routePtB = $oneDestination;
    $mode = { mode: "route", prevMode: "impact-one-destination" };
  }

  function onRightClick(e: MapMouseEvent) {
    // Move the first marker, for convenience
    $oneDestination = e.lngLat;
  }
</script>

<MapEvents oncontextmenu={onRightClick} />

<SplitComponent>
  <div slot="top">
    <nav aria-label="breadcrumb">
      <ul>
        <li>
          <ModeLink mode={{ mode: "title" }} />
        </li>
        <li>
          <ModeLink mode={{ mode: "pick-neighbourhood" }} />
        </li>
        <li>
          <ModeLink mode={{ mode: "neighbourhood" }} />
        </li>
        <li>{pageTitle($mode.mode)}</li>
      </ul>
    </nav>
  </div>

  <div slot="sidebar">
    <BackButton mode={{ mode: "neighbourhood" }} />

    <p>
      This shows the change in driving time to one destination from everywhere
      within the neighbourhood. Drag the pin around to change that destination.
    </p>

    <u style:color="red">Route before changes</u>
    {#if routeBefore}
      <p>
        {prettyPrintDistance(routeBefore.properties.distance)}, {prettyPrintTime(
          routeBefore.properties.time,
        )}
      </p>
    {:else if routeAfter}
      <p>
        No possible route (
        <i>This is usually a known software bug</i>
        )
      </p>
    {:else}
      <p>Hover on a road to compare</p>
    {/if}

    <u style:color="blue">Route after changes</u>
    {#if routeAfter}
      <p>
        {prettyPrintDistance(routeAfter.properties.distance)}, {prettyPrintTime(
          routeAfter.properties.time,
        )}
      </p>
    {:else if routeBefore}
      <p>
        No possible route (
        <i>This is usually a known software bug</i>
        )
      </p>
    {:else}
      <p>Hover on a road to compare</p>
    {/if}
  </div>

  <div slot="map">
    <RenderNeighbourhood>
      <FillLayer
        {...layerId("cells")}
        filter={["==", ["get", "kind"], "cell"]}
        paint={{
          "fill-color": ["get", "color"],
          "fill-opacity": 0.6,
        }}
      />
    </RenderNeighbourhood>

    <GeoJSON data={perRoadGj} generateId>
      <LineLayer
        {...layerId("interior-roads")}
        paint={{
          "line-color": [
            "interpolate-hcl",
            ["linear"],
            ["/", ["get", "time_after"], ["get", "time_before"]],
            1,
            "white",
            Math.max(perRoadGj.highest_time_ratio, 1.1),
            "red",
          ],
          "line-width": 5,
        }}
        manageHoverState
        onclick={(e) => compareRoute(e.features[0])}
        bind:hovered
      >
        <Popup openOn="hover" let:props>
          <p>Time ratio: {(props.time_after / props.time_before).toFixed(1)}</p>
        </Popup>
      </LineLayer>
    </GeoJSON>

    <GeoJSON data={routeGj}>
      <LineLayer
        {...layerId("compare-route")}
        interactive={false}
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

    <ModalFilterLayer interactive={false} />

    <DotMarker bind:lngLat={$oneDestination} draggable>X</DotMarker>
  </div>
</SplitComponent>
