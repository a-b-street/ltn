<script lang="ts">
  import type { Feature, FeatureCollection, Polygon } from "geojson";
  import type { Map, MapMouseEvent } from "maplibre-gl";
  import { RouteTool } from "route-snapper-ts";
  import { onDestroy } from "svelte";
  import {
    FillLayer,
    GeoJSON,
    LineLayer,
    MapEvents,
    Marker,
    Popup,
    type MapMoveEvent,
  } from "svelte-maplibre";
  import { emptyGeojson } from "svelte-utils/map";
  import { run } from "svelte/legacy";
  import { layerId } from "../";
  import { routeTool, type Waypoint } from "./stores";

  interface Props {
    map: Map;
    waypoints: Waypoint[];
    drawnShapeOut?: Feature<Polygon> | undefined;
  }

  let {
    map,
    waypoints = $bindable(),
    drawnShapeOut = $bindable(undefined),
  }: Props = $props();

  function calculateArea(
    routeTool: RouteTool,
    waypoints: Waypoint[],
  ): Feature<Polygon> {
    if (waypoints.length < 3) {
      return JSON.parse(routeTool.inner.calculateRoute(waypoints));
    }

    // Glue the end to the start
    let copy = JSON.parse(JSON.stringify(waypoints));
    copy.push(copy[0]);
    let out = JSON.parse(routeTool.inner.calculateRoute(copy));
    out.properties.waypoints.pop();
    out.geometry.type = "Polygon";
    out.geometry.coordinates = [out.geometry.coordinates];
    return out;
  }

  onDestroy(() => {
    $routeTool?.stop();
    map.getCanvas().style.cursor = "inherit";
  });

  let snapMode: "snap" | "free" = $state("snap");
  let undoStates: Waypoint[][] = $state([]);

  interface ExtraNode {
    point: [number, number];
    insertIdx: number;
    snapped: boolean;
  }
  let extraNodes: ExtraNode[] = $state([]);

  let cursor: Waypoint | null = $state(null);
  let hoveringOnMarker = $state(false);
  let draggingMarker = $state(false);
  let draggingExtraNode = $state(false);

  function updateCursor(waypoints: Waypoint[]) {
    let cursor = waypoints.length == 0 ? "crosshair" : "inherit";
    map.getCanvas().style.cursor = cursor;
  }

  function undo() {
    let state = undoStates.pop();
    undoStates = undoStates;
    if (state) {
      waypoints = state;
    }
  }

  function captureUndoState() {
    if (undoStates.length == 100) {
      undoStates.shift();
    }
    undoStates = [...undoStates, JSON.parse(JSON.stringify(waypoints))];
  }

  function onMapClick(e: MapMouseEvent) {
    if (waypoints.length >= 3) {
      return;
    }
    captureUndoState();
    waypoints.push({
      point: e.lngLat.toArray(),
      snapped: snapMode == "snap",
    });
    waypoints = waypoints;
  }

  function onMouseMove(e: MapMoveEvent) {
    cursor = {
      // @ts-expect-error TODO fix upstream types
      point: e.lngLat.toArray(),
      snapped: snapMode == "snap",
    };
  }

  function toggleSnapped(idx: number) {
    captureUndoState();
    waypoints[idx].snapped = !waypoints[idx].snapped;
    waypoints = waypoints;
  }

  function removeWaypoint(idx: number) {
    // TODO Context menu sometimes appears briefly; need to eat the event?
    captureUndoState();
    waypoints.splice(idx, 1);
    waypoints = waypoints;
    hoveringOnMarker = false;
  }

  function getPreview(
    routeTool: RouteTool | null,
    waypoints: Waypoint[],
    cursor: Waypoint | null,
    suppress: boolean,
  ): FeatureCollection {
    if (suppress || waypoints.length >= 3) {
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

  function updateExtraNodes(
    routeTool: RouteTool | null,
    waypoints: Waypoint[],
    draggingExtraNode: boolean,
  ) {
    if (draggingExtraNode) {
      return;
    }
    if (!routeTool || waypoints.length < 3) {
      extraNodes = [];
      return;
    }

    let nodes: ExtraNode[] = [];
    let insertIdx = 1;

    let copy = JSON.parse(JSON.stringify(waypoints));
    copy.push(copy[0]);

    for (let i = 0; i < copy.length - 1; i++) {
      let extra = JSON.parse(
        routeTool.inner.getExtraNodes(copy[i], copy[i + 1]),
      );
      for (let [x, y, snapped] of extra) {
        nodes.push({ point: [x, y], snapped, insertIdx });
      }
      insertIdx++;
    }

    extraNodes = nodes;
  }

  function addNode(node: ExtraNode) {
    // Turn an extra node into a waypoint. Capture state before the user drags
    // around the new waypoint.
    captureUndoState();
    waypoints.splice(node.insertIdx, 0, {
      point: node.point,
      snapped: node.snapped,
    });
    waypoints = waypoints;
    draggingExtraNode = true;
  }

  function updateDrag(node: ExtraNode) {
    // Don't constantly update undoStates
    waypoints[node.insertIdx].point = node.point;
    waypoints = waypoints;
  }

  function finalizeDrag() {
    draggingExtraNode = false;
  }

  function startDraggingWaypoint() {
    captureUndoState();
    draggingMarker = true;
  }
  run(() => {
    try {
      if ($routeTool) {
        drawnShapeOut = calculateArea($routeTool, waypoints);
      }
    } catch (err) {
      console.log("error drawing shape", err);
      drawnShapeOut = undefined;
    }
  });
  run(() => {
    updateExtraNodes($routeTool, waypoints, draggingExtraNode);
  });
  let previewGj = $derived(
    getPreview(
      $routeTool,
      waypoints,
      cursor,
      hoveringOnMarker || draggingMarker,
    ),
  );
  run(() => {
    updateCursor(waypoints);
  });
</script>

<p>Click the map to add three points. Then adjust the points or add more.</p>
<div style="display: flex; gap: 16px;">
  <label>
    <!-- REVIEW: Currently, after three points, route-snapper will try to draw an area, 
       and whether any subsequent points should snap is based on that new points neighbors.
       Instead, I think it would be simpler to always honor the snap mode set here.
    -->
    <input
      disabled={waypoints.length >= 3}
      type="checkbox"
      value="snap"
      bind:group={snapMode}
    />
    Snap boundary to roads
  </label>
  <button class="secondary" disabled={undoStates.length == 0} onclick={undo}>
    {#if undoStates.length == 0}
      Undo
    {:else}
      Undo ({undoStates.length})
    {/if}
  </button>
</div>

<MapEvents onclick={onMapClick} onmousemove={onMouseMove} />

{#each extraNodes as node}
  <Marker
    draggable
    bind:lngLat={node.point}
    ondragstart={() => addNode(node)}
    ondrag={() => updateDrag(node)}
    ondragend={finalizeDrag}
    onclick={() => {
      addNode(node);
      draggingExtraNode = false;
    }}
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
    onclick={() => toggleSnapped(idx)}
    oncontextmenu={() => removeWaypoint(idx)}
    onmouseenter={() => (hoveringOnMarker = true)}
    onmouseleave={() => (hoveringOnMarker = false)}
    ondragstart={startDraggingWaypoint}
    ondragend={() => (draggingMarker = false)}
    zIndex={1}
  >
    <span class="dot" class:snapped={waypt.snapped}>{idx + 1}</span>
    <Popup openOn="hover" popupClass="edit-waypoint-popup">
      {#snippet children({ data })}
        <ul style="padding-right: 0; padding-left: 20px; margin: 0;">
          <li>
            <b>Click and drag</b>
            to move
          </li>
          <li>
            <b>Click</b>
            to toggle snapping
          </li>
          <li>
            <b>Right click</b>
            to delete
          </li>
        </ul>
      {/snippet}
    </Popup>
  </Marker>
{/each}

<GeoJSON data={drawnShapeOut || emptyGeojson()} generateId>
  <LineLayer
    {...layerId("draw-area-lines")}
    paint={{
      "line-color": "black",
      "line-width": 10,
    }}
  />

  <FillLayer
    {...layerId("draw-area-area")}
    paint={{ "fill-color": "grey", "fill-opacity": 0.5 }}
  />
</GeoJSON>

<GeoJSON data={previewGj}>
  <LineLayer
    {...layerId("draw-area-lines-preview")}
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
    width: 20px;
    height: 20px;
    background-color: grey;
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

  :global(.edit-waypoint-popup) {
    /* This solves two problems: 
    * 1. The popup is too close to the marker
    * 2. The popup obscures other points, making it hard to click on them.
    *    To explain further: The popup remains visible when hovering over the marker **or**, once visible, the popup itself. 
    *    As generic behavior, that makes sense to facilitate clicking on any interactive content within the popup.
    *    However, the .edit-waypoint-popup doesn't have any interactive content in it, so we don't need that.
    *    More important, say you have two nearby points positioned vertically, you start with the bottom point, presenting it's popup which covers the top point.
    *    By adding this padding, there's a little gap between the marker and the popup, so the bottom popup clears while panning to the top point.
    */
    padding-bottom: 16px;

    /* The markers have z-index 1, so the popup needs to be above them */
    z-index: 2;
  }
</style>
