<script lang="ts">
  import type { Feature, FeatureCollection, LineString } from "geojson";
  import type { Map, MapMouseEvent } from "maplibre-gl";
  import { RouteTool } from "route-snapper-ts";
  import { onDestroy } from "svelte";
  import {
    GeoJSON,
    LineLayer,
    MapEvents,
    Marker,
    Popup,
    type MapMoveEvent,
  } from "svelte-maplibre";
  import { emptyGeojson } from "svelte-utils/map";
  import { layerId } from "../common";
  import { routeTool, type Waypoint } from "../common/draw_area/stores";

  interface Props {
    map: Map;
    waypoints: Waypoint[];
    finish: (roads: number[]) => void;
    cancel: () => void;
  }

  let { map, waypoints = $bindable(), finish, cancel }: Props = $props();

  onDestroy(() => {
    waypoints = [];
    $routeTool?.stop();
    map.getCanvas().style.cursor = "inherit";
  });

  let snapMode = true;

  interface ExtraNode {
    point: [number, number];
    insertIdx: number;
    snapped: boolean;
  }
  let extraNodes: ExtraNode[] = $derived.by(() =>
    getExtraNodes($routeTool, waypoints, draggingExtraNode),
  );

  let cursor: Waypoint | null = $state(null);
  let hoveringOnMarker = $state(false);
  let draggingMarker = $state(false);
  let draggingExtraNode = $state(false);

  function onMapClick(e: MapMouseEvent) {
    waypoints.push({
      point: e.lngLat.toArray(),
      snapped: snapMode,
    });
    waypoints = waypoints;
  }

  function onMouseMove(e: MapMoveEvent) {
    cursor = {
      // @ts-expect-error TODO fix upstream types
      point: e.lngLat.toArray(),
      snapped: snapMode,
    };
  }

  function removeWaypoint(idx: number) {
    waypoints.splice(idx, 1);
    waypoints = waypoints;
    hoveringOnMarker = false;
  }

  function calculateRoute(
    routeTool: RouteTool | null,
    waypoints: Waypoint[],
  ): Feature<LineString, { full_path: Array<{ snapped: number }> }> | null {
    try {
      if (routeTool) {
        return JSON.parse(routeTool.inner.calculateRoute(waypoints));
      }
    } catch (err) {}
    return null;
  }

  function getPreview(
    routeTool: RouteTool | null,
    waypoints: Waypoint[],
    cursor: Waypoint | null,
    suppress: boolean,
  ): FeatureCollection {
    if (suppress) {
      return emptyGeojson();
    }
    try {
      if (routeTool && waypoints.length > 0 && cursor) {
        return JSON.parse(
          routeTool.inner.calculateRoute([
            waypoints[waypoints.length - 1],
            cursor,
          ]),
        );
      }
    } catch (err) {}
    return emptyGeojson();
  }

  function getExtraNodes(
    routeTool: RouteTool | null,
    waypoints: Waypoint[],
    draggingExtraNode: boolean,
  ): ExtraNode[] {
    if (draggingExtraNode) {
      return extraNodes;
    }
    if (!routeTool) {
      return [];
    }

    let nodes: ExtraNode[] = [];
    let insertIdx = 1;

    for (let i = 0; i < waypoints.length - 1; i++) {
      let extra = JSON.parse(
        routeTool.inner.getExtraNodes(waypoints[i], waypoints[i + 1]),
      );
      for (let [x, y, snapped] of extra) {
        nodes.push({ point: [x, y], snapped, insertIdx });
      }
      insertIdx++;
    }

    return nodes;
  }

  function addNode(node: ExtraNode) {
    // Turn an extra node into a waypoint.
    waypoints.splice(node.insertIdx, 0, {
      point: node.point,
      snapped: node.snapped,
    });
    waypoints = waypoints;
    draggingExtraNode = true;
  }

  function updateDrag(node: ExtraNode) {
    waypoints[node.insertIdx].point = node.point;
    waypoints = waypoints;
  }

  function finalizeDrag() {
    draggingExtraNode = false;
  }

  function startDraggingWaypoint() {
    draggingMarker = true;
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.key == "Escape") {
      cancel();
    }
    if (e.key == "Enter" && waypoints.length > 1) {
      finishDrawing();
    }
  }

  function clickWaypoint(idx: number) {
    if (waypoints.length > 1 && idx == waypoints.length - 1) {
      finishDrawing();
    }
  }

  function finishDrawing() {
    let gj = calculateRoute($routeTool, waypoints);
    if (gj) {
      finish(gj.properties.full_path.map((step) => step.snapped));
      waypoints = [];
    } else {
      cancel();
    }
  }
  let previewGj = $derived(
    getPreview(
      $routeTool,
      waypoints,
      cursor,
      hoveringOnMarker || draggingMarker,
    ),
  );
  $effect(() => {
    let cursor = waypoints.length == 0 ? "crosshair" : "inherit";
    map.getCanvas().style.cursor = cursor;
  });
</script>

<svelte:window onkeydown={onKeyDown} />

<MapEvents onclick={onMapClick} onmousemove={onMouseMove} />

{#each extraNodes as node}
  <Marker
    draggable
    bind:lngLat={node.point}
    ondragstart={() => addNode(node)}
    ondrag={() => updateDrag(node)}
    ondragend={finalizeDrag}
    zIndex={0}
  >
    <span
      class="dot"
      class:snapped-node={node.snapped}
      class:free-node={!node.snapped}
      class:hide={draggingExtraNode}
    ></span>
  </Marker>
{/each}

{#each waypoints as waypt, idx}
  <Marker
    draggable
    bind:lngLat={waypt.point}
    onclick={() => clickWaypoint(idx)}
    oncontextmenu={() => removeWaypoint(idx)}
    onmouseenter={() => (hoveringOnMarker = true)}
    onmouseleave={() => (hoveringOnMarker = false)}
    ondragstart={startDraggingWaypoint}
    ondragend={() => (draggingMarker = false)}
    zIndex={1}
  >
    <span class="dot" class:snapped={waypt.snapped}>{idx + 1}</span>
    <Popup openOn="hover" popupClass="waypoint-popup">
      <ul style="padding-right: 0; padding-left: 20px; margin: 0;">
        <li>
          <b>Click and drag</b>
          to move
        </li>
        <li>
          <b>Right click</b>
          to delete
        </li>
        {#if waypoints.length > 1 && idx == waypoints.length - 1}
          <li><b>Click</b> to reclassify everything highlighted</li>
        {/if}
      </ul>
    </Popup>
  </Marker>
{/each}

<GeoJSON
  data={calculateRoute($routeTool, waypoints) || emptyGeojson()}
  generateId
>
  <LineLayer
    {...layerId("draw-route-lines")}
    paint={{
      "line-color": "purple",
      "line-width": 10,
    }}
  />
</GeoJSON>

<GeoJSON data={previewGj}>
  <LineLayer
    {...layerId("draw-route-preview")}
    paint={{
      "line-color": "black",
      "line-width": 3,
    }}
  />
</GeoJSON>

<style>
  .dot {
    width: 30px;
    height: 30px;
    border-radius: 50%;
    display: flex;
    justify-content: center;
    align-items: center;

    color: white;
    background-color: blue;
    font-weight: bold;
  }

  .dot:hover {
    border: 1px solid black;
    cursor: pointer;
  }

  .snapped {
    background-color: red;
  }

  .free-node,
  .snapped-node {
    width: 10px;
    height: 10px;
    background-color: white;
  }

  .snapped-node:hover {
    border: 3px solid red;
  }

  .free-node:hover {
    border: 3px solid blue;
  }

  .hide {
    visibility: hidden;
  }

  /* See comments in AreaControls for rationale */
  :global(.waypoint-popup) {
    padding-bottom: 16px;
    z-index: 2;
  }
</style>
