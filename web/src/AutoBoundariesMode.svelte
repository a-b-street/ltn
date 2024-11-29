<script lang="ts">
  import BackButton from "./BackButton.svelte";
  import {
    FillLayer,
    GeoJSON,
    LineLayer,
    hoverStateFilter,
  } from "svelte-maplibre";
  import { Link, layerId } from "./common";
  import { notNull } from "svelte-utils";
  import { isLine } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import { app, mode } from "./stores";
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
        <li>
          <Link on:click={() => ($mode = { mode: "network" })}>
            Pick neighbourhood
          </Link>
        </li>
        <li>Generated boundaries (experiment)</li>
      </ul>
    </nav>
  </div>

  <div slot="sidebar">
    <BackButton on:click={() => ($mode = { mode: "network" })} />
  </div>

  <div slot="map">
    <GeoJSON data={JSON.parse(notNull($app).renderAutoBoundaries())} generateId>
      <FillLayer
        {...layerId("auto-boundaries-areas")}
        manageHoverState
        paint={{
          "fill-color": "cyan",
          "fill-opacity": hoverStateFilter(0.3, 0.5),
        }}
      />

      <LineLayer
        {...layerId("auto-boundaries-severances")}
        filter={isLine}
        manageHoverState
        paint={{
          "line-color": hoverStateFilter("black", "red"),
          "line-width": 3,
        }}
      />
    </GeoJSON>
  </div>
</SplitComponent>
