<script lang="ts">
  import { Link } from "../";
  import { PolygonTool } from "./polygon_tool";
  import { undoLength } from "./stores";

  export let polygonTool: PolygonTool;
</script>

<div style="display: flex; justify-content: space-between;">
  <Link on:click={() => polygonTool.finish()}>Finish</Link>
  <Link on:click={() => polygonTool.cancel()}>Cancel</Link>
  <button disabled={$undoLength == 0} on:click={() => polygonTool.undo()}>
    {#if $undoLength == 0}
      Undo
    {:else}
      Undo ({$undoLength})
    {/if}
  </button>
</div>

<ul>
  <li>
    <b>Click</b>
    the map to add a vertex
  </li>
  <li>
    <b>Click</b>
    a vertex to delete it
  </li>
  <li>
    <b>Drag</b>
    a vertex or the polygon to move it
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
