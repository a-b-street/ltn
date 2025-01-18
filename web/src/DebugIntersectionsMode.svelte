<script lang="ts">
  import {
    CircleLayer,
    GeoJSON,
    LineLayer,
    type LayerClickInfo,
  } from "svelte-maplibre";
  import { notNull } from "svelte-utils";
  import { emptyGeojson } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import BackButton from "./BackButton.svelte";
  import { layerId, Link } from "./common";
  import { backend, mode } from "./stores";

  let movements = emptyGeojson();

  function pickIntersection(e: CustomEvent<LayerClickInfo>) {
    movements = $backend!.getMovements(e.detail.features[0].id as number);
  }
</script>

<SplitComponent>
  <div slot="top">
    <nav aria-label="breadcrumb">
      <ul>
        <li>
          <Link on:click={() => ($mode = { mode: "title", firstLoad: false })}>
            Choose project
          </Link>
        </li>
        <li>Debug intersections</li>
      </ul>
    </nav>
  </div>

  <div slot="sidebar">
    <BackButton on:click={() => ($mode = { mode: "network" })} />

    {#if movements.features.length > 0}
      <button class="secondary" on:click={() => (movements = emptyGeojson())}>
        Pick another intersection
      </button>
      <p>{movements.features.length} movements</p>
    {/if}
  </div>

  <div slot="map">
    <GeoJSON data={notNull($backend).getAllIntersections()} generateId>
      <CircleLayer
        {...layerId("debug-intersections")}
        paint={{
          "circle-radius": 15,
          "circle-color": "black",
        }}
        manageHoverState
        hoverCursor="pointer"
        on:click={pickIntersection}
      />
    </GeoJSON>

    <GeoJSON data={movements} generateId>
      <LineLayer
        {...layerId("debug-movements")}
        paint={{
          "line-width": 2,
          "line-color": "red",
        }}
      />
    </GeoJSON>
  </div>
</SplitComponent>
