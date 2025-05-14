<script lang="ts">
  import { type Map, type MapMouseEvent } from "maplibre-gl";
  import { onDestroy, onMount } from "svelte";
  import { GeoJSON, LineLayer, MapEvents, Marker } from "svelte-maplibre";
  import { layerId } from "../common";
  import { backend } from "../stores";
  import type { SnappedRoute, Waypoint2 as Waypoint } from "../wasm";

  export let map: Map;
  export let finish: (roads: number[]) => void;
  export let cancel: () => void;

  let waypoints: Waypoint[] = [];
  let undoStates: Waypoint[][] = [];

  // TODO Fix svelte-maplibre -- this isn't just a layer event
  onMount(() => {
    map.on("mouseout", onMouseOut);
  });

  onDestroy(() => {
    map.off("mouseout", onMouseOut);
    map.getCanvas().style.cursor = "inherit";
  });

  interface ExtraNode {
    point: [number, number];
    insertIdx: number;
  }

  let extraNodes: ExtraNode[] = [];
  $: updateExtraNodes(waypoints, draggingExtraNode);

  let cursor: Waypoint | null = null;
  let hoveringOnWaypoint = false;
  let hoveringOnExtraNode = false;
  let draggingWaypoint = false;
  let draggingExtraNode = false;

  function emptyRoute(): SnappedRoute {
    return { type: "FeatureCollection", features: [], roads: [] };
  }

  let confirmedRouteGj: SnappedRoute = emptyRoute();
  $: updateConfirmedRoute(waypoints);

  let previewGj = emptyRoute();
  $: updatePreview(
    waypoints,
    cursor,
    hoveringOnWaypoint ||
      hoveringOnExtraNode ||
      draggingWaypoint ||
      draggingExtraNode,
  );

  $: updateCursorStyle(waypoints);
  function updateCursorStyle(waypoints: Waypoint[]) {
    let cursorStyle = waypoints.length == 0 ? "crosshair" : "inherit";
    map.getCanvas().style.cursor = cursorStyle;
  }

  export function undo() {
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

  function onMapClick(e: CustomEvent<MapMouseEvent>) {
    captureUndoState();
    waypoints.push(
      $backend!.snapPointInNeighbourhood(e.detail.lngLat.toArray()),
    );
    waypoints = waypoints;
  }

  function onMouseMove(e: CustomEvent<MapMouseEvent>) {
    cursor = e.detail.lngLat.toArray();
  }

  function onMouseOut() {
    cursor = null;
    previewGj = emptyRoute();
  }

  function removeWaypoint(idx: number) {
    captureUndoState();
    waypoints.splice(idx, 1);
    waypoints = waypoints;
    hoveringOnWaypoint = false;
  }

  function updatePreview(
    waypoints: Waypoint[],
    cursor: Waypoint | null,
    suppress: boolean,
  ) {
    if (suppress || !cursor) {
      previewGj = emptyRoute();
      return;
    }

    try {
      previewGj = $backend!.snapRouteInNeighbourhood([
        waypoints[waypoints.length - 1],
        cursor,
      ]);
    } catch (err) {}
  }

  function updateConfirmedRoute(waypoints: Waypoint[]) {
    try {
      confirmedRouteGj = $backend!.snapRouteInNeighbourhood(waypoints);
    } catch (err) {}
  }

  function updateExtraNodes(waypoints: Waypoint[], draggingExtraNode: boolean) {
    if (draggingExtraNode) {
      return;
    }

    let nodes: ExtraNode[] = [];
    let insertIdx = 1;

    for (let i = 0; i < waypoints.length - 1; i++) {
      let extra = $backend!.getExtraNodes(waypoints[i], waypoints[i + 1]);
      for (let [x, y] of extra) {
        nodes.push({ point: [x, y], insertIdx });
      }
      insertIdx++;
    }

    extraNodes = nodes;
  }

  function addNode(node: ExtraNode) {
    // Turn an extra node into a waypoint. Capture state before the user drags
    // around the new waypoint.
    captureUndoState();
    waypoints.splice(node.insertIdx, 0, node.point);
    waypoints = waypoints;
    hoveringOnExtraNode = false;
    draggingExtraNode = true;
  }

  function updateDrag(node: ExtraNode) {
    // Don't constantly update undoStates
    waypoints[node.insertIdx] = node.point;
    waypoints = waypoints;
  }

  function finalizeDrag(node: ExtraNode) {
    draggingExtraNode = false;
    waypoints[node.insertIdx] = node.point;
    waypoints = waypoints;
  }

  function keyDown(e: KeyboardEvent) {
    let tag = (e.target as HTMLElement).tagName;
    let formFocused = tag == "INPUT" || tag == "TEXTAREA" || tag == "SELECT";

    if (e.key === "Enter" && !formFocused) {
      e.stopPropagation();
      if (confirmedRouteGj.roads.length > 0) {
        finish(confirmedRouteGj.roads);
      }
    } else if (e.key === "Escape") {
      e.stopPropagation();
      cancel();
    } else if (e.key == "z" && e.ctrlKey && !formFocused) {
      e.stopPropagation();
      undo();
    }
  }

  function startDraggingWaypoint() {
    captureUndoState();
    draggingWaypoint = true;
  }

  function stopDraggingWaypoint(idx: number) {
    draggingWaypoint = false;

    let point = $backend!.snapPointInNeighbourhood(waypoints[idx]);
    waypoints[idx] = point;
    waypoints = waypoints;
  }

  function onClickWaypoint(idx: number) {
    if (waypoints.length < 2) {
      return;
    }
    // Click the end to finish
    if (idx == waypoints.length - 1) {
      finish(confirmedRouteGj.roads);
    }
  }
</script>

<svelte:window on:keydown={keyDown} />

<MapEvents on:click={onMapClick} on:mousemove={onMouseMove} />

{#each extraNodes as node}
  <Marker
    draggable
    bind:lngLat={node.point}
    on:mouseenter={() => (hoveringOnExtraNode = true)}
    on:mouseleave={() => (hoveringOnExtraNode = false)}
    on:dragstart={() => addNode(node)}
    on:drag={() => updateDrag(node)}
    on:dragend={() => finalizeDrag(node)}
    zIndex={0}
  >
    <span class="extra-node-clickable" class:hide={draggingExtraNode}>
      <span class="extra-node-display" class:hide={draggingExtraNode} />
    </span>
  </Marker>
{/each}

{#each waypoints as waypt, idx}
  <Marker
    draggable
    bind:lngLat={waypt}
    on:click={() => onClickWaypoint(idx)}
    on:contextmenu={() => removeWaypoint(idx)}
    on:mouseenter={() => (hoveringOnWaypoint = true)}
    on:mouseleave={() => (hoveringOnWaypoint = false)}
    on:dragstart={startDraggingWaypoint}
    on:dragend={() => stopDraggingWaypoint(idx)}
    zIndex={1}
  >
    <span class="waypoint">{idx + 1}</span>
  </Marker>
{/each}

<GeoJSON data={confirmedRouteGj}>
  <LineLayer
    {...layerId("snapper-confirmed")}
    paint={{
      "line-color": "blue",
      "line-width": 5,
    }}
  />
</GeoJSON>

<GeoJSON data={previewGj}>
  <LineLayer
    {...layerId("snapper-preview")}
    paint={{
      "line-color": "blue",
      "line-width": 3,
      "line-dasharray": [3, 2],
    }}
  />
</GeoJSON>

<style>
  /** Styling on the map **/
  .waypoint {
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

  .waypoint:hover {
    border: 1px solid black;
    cursor: pointer;
  }

  .extra-node-clickable {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    display: flex;
    position: relative;
  }

  .extra-node-clickable:hover {
    background-color: blue;
    width: 20px;
    height: 20px;
  }

  .extra-node-display {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    display: flex;
    background-color: white;

    /* Center the small displayed circle inside the larger invisible hitbox circle */
    position: absolute;
    top: 50%;
    left: 50%;
    margin: -2.5px 0px 0px -2.5px;
  }

  .hide {
    visibility: hidden;
  }
</style>
