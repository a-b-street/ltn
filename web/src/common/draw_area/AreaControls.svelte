<script lang="ts">
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { emptyGeojson } from "svelte-utils/map";
  import { routeTool, waypoints, calculateArea, type Waypoint } from "./stores";
  import { layerId, HelpButton } from "../";
  import {
    Marker,
    MapEvents,
    GeoJSON,
    LineLayer,
    FillLayer,
  } from "svelte-maplibre";
  import type { Map, MapMouseEvent } from "maplibre-gl";
  import type { FeatureCollection, GeoJSON as GeoJSONType } from "geojson";
  import { RouteTool } from "route-snapper-ts";
  import { onDestroy } from "svelte";

  export let map: Map;
  export let finish: () => void;
  export let cancel: () => void;

  onDestroy(() => {
    $waypoints = [];
    $routeTool?.stop();
    map.getCanvas().style.cursor = "inherit";
  });

  let snapMode: "snap" | "free" = "snap";
  let undoStates: Waypoint[][] = [];

  interface ExtraNode {
    point: [number, number];
    insertIdx: number;
    snapped: boolean;
  }
  let extraNodes: ExtraNode[] = [];
  $: updateExtraNodes($routeTool, $waypoints, draggingExtraNode);

  let cursor: Waypoint | null = null;
  let hoveringOnMarker = false;
  let draggingMarker = false;
  let draggingExtraNode = false;
  $: previewGj = getPreview(
    $routeTool,
    $waypoints,
    cursor,
    hoveringOnMarker || draggingMarker,
  );

  $: updateCursor($waypoints);
  function updateCursor(waypoints: Waypoint[]) {
    let cursor = waypoints.length == 0 ? "crosshair" : "inherit";
    map.getCanvas().style.cursor = cursor;
  }

  function undo() {
    let state = undoStates.pop();
    undoStates = undoStates;
    if (state) {
      $waypoints = state;
    }
  }

  function captureUndoState() {
    if (undoStates.length == 100) {
      undoStates.shift();
    }
    undoStates = [...undoStates, JSON.parse(JSON.stringify($waypoints))];
  }

  function toggleSnap() {
    snapMode = snapMode == "snap" ? "free" : "snap";
    if (cursor) {
      cursor.snapped = snapMode == "snap";
    }
  }

  function onMapClick(e: CustomEvent<MapMouseEvent>) {
    if ($waypoints.length >= 3) {
      return;
    }
    captureUndoState();
    waypoints.update((w) => {
      w.push({
        point: e.detail.lngLat.toArray(),
        snapped: snapMode == "snap",
      });
      return w;
    });
  }

  function onMouseMove(e: CustomEvent<MapMouseEvent>) {
    cursor = {
      point: e.detail.lngLat.toArray(),
      snapped: snapMode == "snap",
    };
  }

  function toggleSnapped(idx: number) {
    captureUndoState();
    waypoints.update((w) => {
      w[idx].snapped = !w[idx].snapped;
      return w;
    });
  }

  function removeWaypoint(idx: number) {
    // TODO Context menu sometimes appears briefly; need to eat the event?
    captureUndoState();
    waypoints.update((w) => {
      w.splice(idx, 1);
      return w;
    });
    hoveringOnMarker = false;
  }

  function calculateGj(
    routeTool: RouteTool | null,
    waypoints: Waypoint[],
  ): GeoJSONType {
    try {
      if (routeTool) {
        return calculateArea(waypoints);
      }
    } catch (err) {}
    return emptyGeojson();
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
    waypoints.update((w) => {
      w.splice(node.insertIdx, 0, {
        point: node.point,
        snapped: node.snapped,
      });
      return w;
    });
    draggingExtraNode = true;
  }

  function updateDrag(node: ExtraNode) {
    // Don't constantly update undoStates
    waypoints.update((w) => {
      w[node.insertIdx].point = node.point;
      return w;
    });
  }

  function finalizeDrag() {
    draggingExtraNode = false;
  }

  function keyDown(e: KeyboardEvent) {
    let tag = (e.target as HTMLElement).tagName;
    let formFocused = tag == "INPUT" || tag == "TEXTAREA";

    if (e.key == "Enter" && !formFocused) {
      e.stopPropagation();
      if ($waypoints.length >= 3) {
        finish();
      } else {
        window.alert(
          "You can't save this area unless it has at least three points. Press 'Cancel' to discard these changes.",
        );
      }
    } else if (e.key == "Escape") {
      e.stopPropagation();
      if ($waypoints.length >= 3) {
        finish();
      } else {
        window.alert(
          "You can't save this area unless it has at least three points. Press 'Cancel' to discard these changes.",
        );
      }
    } else if (e.key == "s" && !formFocused) {
      toggleSnap();
    } else if (e.key == "z" && e.ctrlKey) {
      e.stopPropagation();
      undo();
    }
  }

  function startDraggingWaypoint() {
    captureUndoState();
    draggingMarker = true;
  }
</script>

<svelte:window on:keydown={keyDown} />

<SplitComponent>
  <div slot="top">
    <slot name="extra-top" />
  </div>

  <div slot="sidebar">
    <slot name="extra-sidebar" />

    <div style="display: flex">
      <button on:click={finish} disabled={$waypoints.length < 3}>Finish</button>
      <button class="secondary" on:click={cancel}>Cancel</button>
      <HelpButton>
        <ul>
          <li>
            <b>Click</b>
            the map to add new points, until there are at least 3 points
          </li>
          <li>
            <b>Click and drag</b>
            any point to move it
          </li>
          <li>
            <b>Click</b>
            a waypoint to toggle snapping
          </li>
          <li>
            <b>Right click</b>
            a waypoint to delete it
          </li>
        </ul>

        <p>Keyboard shortcuts:</p>
        <ul>
          <li>
            <b>s</b>
            to switch between snapping to roads and drawing anywhere
          </li>
          <li>
            <b>Control+Z</b>
            to undo your last change
          </li>
          <li>
            <b>Enter</b>
            or
            <b>Escape</b>
            to finish
          </li>
          <li>
            From the main mode, <b>3</b>
            to draw a new area
          </li>
        </ul>
      </HelpButton>
    </div>
    <button class="secondary" disabled={undoStates.length == 0} on:click={undo}>
      {#if undoStates.length == 0}
        Undo
      {:else}
        Undo ({undoStates.length})
      {/if}
    </button>

    {#if $waypoints.length < 3}
      <fieldset>
        <label>
          <input type="radio" value="snap" bind:group={snapMode} />
          Snap to roads
        </label>
        <label>
          <input type="radio" value="free" bind:group={snapMode} />
          Draw anywhere
        </label>
      </fieldset>

      <p>Click to add at least 3 points</p>
    {:else}
      <p>Drag to adjust. Click to toggle snapped. Right click to delete.</p>
    {/if}
  </div>

  <div slot="map">
    <MapEvents on:click={onMapClick} on:mousemove={onMouseMove} />

    {#each extraNodes as node}
      <Marker
        draggable
        bind:lngLat={node.point}
        on:dragstart={() => addNode(node)}
        on:drag={() => updateDrag(node)}
        on:dragend={finalizeDrag}
        on:click={() => {
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
        />
      </Marker>
    {/each}

    {#each $waypoints as waypt, idx}
      <Marker
        draggable
        bind:lngLat={waypt.point}
        on:click={() => toggleSnapped(idx)}
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

    <GeoJSON data={calculateGj($routeTool, $waypoints)} generateId>
      <LineLayer
        {...layerId("draw-area-lines")}
        paint={{
          "line-color": "black",
          "line-width": 10,
        }}
      />

      <FillLayer
        {...layerId("draw-area-preview")}
        paint={{ "fill-color": "grey", "fill-opacity": 0.5 }}
      />
    </GeoJSON>

    <GeoJSON data={previewGj}>
      <LineLayer
        paint={{
          "line-color": "black",
          "line-width": 3,
        }}
      />
    </GeoJSON>
  </div>
</SplitComponent>

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
