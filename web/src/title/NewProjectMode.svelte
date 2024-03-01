<script lang="ts">
  import type { Feature } from "geojson";
  import { overpassQueryForPolygon, Link } from "../common";
  import PolygonToolLayer from "../common/draw_polygon/PolygonToolLayer.svelte";
  import SplitComponent from "../SplitComponent.svelte";
  import { projectName, example, map, mode } from "../stores";
  import MapLoader from "./MapLoader.svelte";
</script>

<SplitComponent>
  <div slot="top">
    <nav aria-label="breadcrumb">
      <ul>
        <li>
          <Link on:click={() => ($mode = { mode: "title" })}>
            Choose project
          </Link>
        </li>
        <li>New project</li>
      </ul>
    </nav>
  </div>

  <div slot="sidebar">
    <div>
      <label>
        Project name:
        <input type="text" bind:value={$projectName} />
      </label>
    </div>
  </div>

  <div slot="map">
    <PolygonToolLayer />
  </div>
</SplitComponent>
