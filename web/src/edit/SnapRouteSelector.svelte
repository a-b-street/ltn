<script lang="ts">
  import type { Feature, FeatureCollection, LineString } from "geojson";
  import type { Map, MapMouseEvent } from "maplibre-gl";
  import { RouteTool } from "route-snapper-ts";
  import { onDestroy } from "svelte";
  import { GeoJSON, LineLayer, MapEvents, Marker } from "svelte-maplibre";
  import { emptyGeojson } from "svelte-utils/map";
  import { layerId } from "../common";
  import { routeTool, type Waypoint } from "../common/draw_area/stores";

  export let map: Map;
  export let waypoints: Waypoint[];
  export let finish: (roads: number[]) => void;
  export let cancel: () => void;

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
  let extraNodes: ExtraNode[] = [];
  $: updateExtraNodes($routeTool, waypoints, draggingExtraNode);

  let cursor: Waypoint | null = null;
  let hoveringOnMarker = false;
  let draggingMarker = false;
  let draggingExtraNode = false;
  $: previewGj = getPreview(
    $routeTool,
    waypoints,
    cursor,
    hoveringOnMarker || draggingMarker,
  );

  $: updateCursor(waypoints);
  function updateCursor(waypoints: Waypoint[]) {
    let cursor = waypoints.length == 0 ? "crosshair" : "inherit";
    map.getCanvas().style.cursor = cursor;
  }

  function onMapClick(e: CustomEvent<MapMouseEvent>) {
    waypoints.push({
      point: e.detail.lngLat.toArray(),
      snapped: snapMode,
    });
    waypoints = waypoints;
  }

  function onMouseMove(e: CustomEvent<MapMouseEvent>) {
    cursor = {
      point: e.detail.lngLat.toArray(),
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

  function updateExtraNodes(
    routeTool: RouteTool | null,
    waypoints: Waypoint[],
    draggingExtraNode: boolean,
  ) {
    if (draggingExtraNode) {
      return;
    }
    if (!routeTool) {
      extraNodes = [];
      return;
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

    extraNodes = nodes;
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
      let gj = calculateRoute($routeTool, waypoints);
      if (gj) {
        finish(gj.properties.full_path.map((step) => step.snapped));
      } else {
        cancel();
      }
    }
  }
</script>

<svelte:window on:keydown={onKeyDown} />

<MapEvents on:click={onMapClick} on:mousemove={onMouseMove} />

{#each extraNodes as node}
  <Marker
    draggable
    bind:lngLat={node.point}
    on:dragstart={() => addNode(node)}
    on:drag={() => updateDrag(node)}
    on:dragend={finalizeDrag}
    zIndex={0}
  >
    <span
      class="dot"
      class:snapped-node={node.snapped}
      class:free-node={!node.snapped}
      class:hide={draggingExtraNode}
    />
  </Marker>
{/each}

{#each waypoints as waypt, idx}
  <Marker
    draggable
    bind:lngLat={waypt.point}
    on:contextmenu={() => removeWaypoint(idx)}
    on:mouseenter={() => (hoveringOnMarker = true)}
    on:mouseleave={() => (hoveringOnMarker = false)}
    on:dragstart={startDraggingWaypoint}
    on:dragend={() => (draggingMarker = false)}
    zIndex={1}
  >
    <span class="dot" class:snapped={waypt.snapped}>{idx + 1}</span>
  </Marker>
{/each}

<GeoJSON
  data={calculateRoute($routeTool, waypoints) || emptyGeojson()}
  generateId
>
  <LineLayer
    {...layerId("draw-route-lines")}
    paint={{
      "line-color": "black",
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
</style>
