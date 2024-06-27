<script lang="ts">
  import { RouteTool } from "route-snapper-ts";
  import { undoLength, showAllNodes, showAllNodesGj } from "./stores";

  export let route_tool: RouteTool;

  function loadNodes(show: boolean) {
    if (show && $showAllNodesGj.features.length == 0) {
      $showAllNodesGj = JSON.parse(route_tool.inner.debugSnappableNodes());
    }
  }
  $: loadNodes($showAllNodes);
</script>

<button
  disabled={$undoLength == 0}
  on:click={() => route_tool.undo()}
  data-tooltip="Ctrl+Z"
>
  {#if $undoLength == 0}
    Undo
  {:else}
    Undo ({$undoLength})
  {/if}
</button>

<label>
  <input type="checkbox" bind:checked={$showAllNodes} />
  Show all snappable points
</label>

<ul>
  <li>
    <b>Click</b>
    the map to add points
  </li>
  <li>
    <b>Click and drag</b>
    any point to move it
  </li>
  <li>
    <b>Click</b>
    a waypoint to delete it
  </li>
  <li>
    Press <b>Control+Z</b>
    to undo your last change
  </li>
  <li>
    Press <b>Enter</b>
    or
    <b>double click</b>
    to finish
  </li>
  <li>
    Press <b>Escape</b>
    to cancel
  </li>
</ul>
