<script lang="ts">
  import {
    CircleLayer,
    FillLayer,
    GeoJSON,
    LineLayer,
    type LayerClickInfo,
  } from "svelte-maplibre";
  import { notNull } from "svelte-utils";
  import { emptyGeojson } from "svelte-utils/map";
  import { SplitComponent } from "svelte-utils/top_bar_layout";
  import BackButton from "./BackButton.svelte";
  import { layerId, Link, PrevNext } from "./common";
  import { backend, mode } from "./stores";

  let movements = emptyGeojson();
  let idx = 0;

  function pickIntersection(e: CustomEvent<LayerClickInfo>) {
    movements = $backend!.getMovements(e.detail.features[0].id as number);
    idx = 0;
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
        <li>
          <Link on:click={() => ($mode = { mode: "pick-neighbourhood" })}>
            Pick neighbourhood
          </Link>
        </li>
        <li>Debug intersections</li>
      </ul>
    </nav>
  </div>

  <div slot="sidebar">
    <BackButton on:click={() => ($mode = { mode: "pick-neighbourhood" })} />

    <p>Purple intersections have some kind of turn restriction.</p>

    {#if movements.features.length > 0}
      <button class="secondary" on:click={() => (movements = emptyGeojson())}>
        Pick another intersection
      </button>

      <PrevNext list={movements.features} bind:idx />
    {/if}
  </div>

  <div slot="map">
    <GeoJSON data={notNull($backend).getAllIntersections()} generateId>
      <CircleLayer
        {...layerId("debug-intersections")}
        paint={{
          "circle-radius": 15,
          "circle-color": [
            "case",
            ["get", "has_turn_restrictions"],
            "purple",
            "black",
          ],
        }}
        manageHoverState
        hoverCursor="pointer"
        on:click={pickIntersection}
      />
    </GeoJSON>

    <GeoJSON data={movements} generateId>
      <LineLayer
        {...layerId("debug-movements-outline")}
        paint={{
          "line-width": 2,
          "line-color": "red",
        }}
      />

      <FillLayer
        {...layerId("debug-movements-fill")}
        filter={["==", ["id"], idx]}
        paint={{
          "fill-color": "cyan",
        }}
      />
    </GeoJSON>
  </div>
</SplitComponent>
